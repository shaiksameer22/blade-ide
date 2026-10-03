use anyhow::{bail, Context, Result};
use log::{debug, error};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, oneshot, Mutex};

#[derive(Serialize, Deserialize, Debug)]
pub struct DapRequest {
    pub seq: usize,
    #[serde(rename = "type")]
    pub type_: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DapResponse {
    pub seq: usize,
    #[serde(rename = "type")]
    pub type_: String,
    pub request_seq: usize,
    pub success: bool,
    pub command: String,
    pub message: Option<String>,
    pub body: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DapEvent {
    pub seq: usize,
    #[serde(rename = "type")]
    pub type_: String,
    pub event: String,
    pub body: Option<Value>,
}

#[derive(Clone)]
pub struct DapClient {
    tx: mpsc::Sender<String>,
    next_seq: Arc<AtomicUsize>,
    pending_requests: Arc<Mutex<HashMap<usize, oneshot::Sender<Result<DapResponse>>>>>,
}

impl DapClient {
    pub fn new(mut cmd: Command) -> Result<Self> {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn debug adapter")?;

        let mut stdin = child.stdin.take().context("Failed to get stdin")?;
        let stdout = child.stdout.take().context("Failed to get stdout")?;

        let (tx, mut rx) = mpsc::channel::<String>(32);
        let pending_requests: Arc<Mutex<HashMap<usize, oneshot::Sender<Result<DapResponse>>>>> = Arc::new(Mutex::new(HashMap::new()));
        let pending_requests_clone = pending_requests.clone();

        // Write task
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let content_length = msg.len();
                let payload = format!("Content-Length: {}\r\n\r\n{}", content_length, msg);
                if let Err(e) = stdin.write_all(payload.as_bytes()).await {
                    error!("Failed to write to debug adapter: {}", e);
                    break;
                }
                if let Err(e) = stdin.flush().await {
                    error!("Failed to flush debug adapter stdin: {}", e);
                    break;
                }
            }
        });

        // Read task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut content_length = 0;
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line).await {
                        Ok(0) => return, // EOF
                        Ok(_) => {
                            let line = line.trim();
                            if line.is_empty() {
                                break;
                            }
                            if let Some(stripped) = line.strip_prefix("Content-Length: ") {
                                if let Ok(len) = stripped.parse::<usize>() {
                                    content_length = len;
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error reading from debug adapter: {}", e);
                            return;
                        }
                    }
                }

                if content_length > 0 {
                    let mut buf = vec![0u8; content_length];
                    if let Err(e) = reader.read_exact(&mut buf).await {
                        error!("Failed to read message body: {}", e);
                        return;
                    }

                    if let Ok(msg) = String::from_utf8(buf) {
                        if let Ok(response) = serde_json::from_str::<DapResponse>(&msg) {
                            if response.type_ == "response" {
                                let mut pending = pending_requests_clone.lock().await;
                                if let Some(sender) = pending.remove(&response.request_seq) {
                                    let _ = sender.send(Ok(response));
                                }
                            }
                        }
                        // Handle events here as well if needed
                    }
                }
            }
        });

        Ok(Self {
            tx,
            next_seq: Arc::new(AtomicUsize::new(1)),
            pending_requests,
        })
    }

    pub async fn send_request<T: Serialize, R: DeserializeOwned>(
        &self,
        command: &str,
        arguments: Option<T>,
    ) -> Result<R> {
        let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
        let request = DapRequest {
            seq,
            type_: "request".to_string(),
            command: command.to_string(),
            arguments: arguments.map(|a| serde_json::to_value(a).unwrap()),
        };

        let msg = serde_json::to_string(&request)?;
        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().await.insert(seq, tx);

        self.tx.send(msg).await?;

        let response = rx.await??;
        if response.success {
            if let Some(body) = response.body {
                Ok(serde_json::from_value(body)?)
            } else {
                // Return generic empty type or fail if body is expected.
                // In DAP, some successful responses have no body.
                // This generic parsing might fail if R isn't Option or Unit.
                // For simplicity, we just use from_value with null.
                Ok(serde_json::from_value(Value::Null)?)
            }
        } else {
            bail!("DAP request failed: {:?}", response.message)
        }
    }
}
