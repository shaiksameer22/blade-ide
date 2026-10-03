use std::process::Stdio;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use anyhow::{bail, Context, Result};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::process::Command;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

pub struct LspClient {
    next_id: AtomicI32,
    sender: mpsc::Sender<Message>,
    pub diagnostics_rx: Option<mpsc::Receiver<lsp_types::PublishDiagnosticsParams>>,
}

enum Message {
    Request(Request, oneshot::Sender<Response>),
    Notification(Notification),
}

impl LspClient {
    pub async fn new(cmd: &str) -> Result<Self> {
        let mut child = Command::new(cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("Failed to spawn LSP server")?;

        let stdin = child.stdin.take().context("No stdin")?;
        let stdout = child.stdout.take().context("No stdout")?;

        let (tx, rx) = mpsc::channel(32);
        let (diag_tx, diag_rx) = mpsc::channel(32);
        
        let pending_requests = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::<i32, oneshot::Sender<Response>>::new()));

        // Writer task
        let pending_reqs_clone = Arc::clone(&pending_requests);
        tokio::spawn(async move {
            let mut rx = rx;
            let mut writer = BufWriter::new(stdin);
            
            while let Some(msg) = rx.recv().await {
                let json_bytes = match msg {
                    Message::Request(req, reply_tx) => {
                        let id = req.id;
                        pending_reqs_clone.lock().await.insert(id, reply_tx);
                        serde_json::to_vec(&req).unwrap()
                    }
                    Message::Notification(notif) => {
                        serde_json::to_vec(&notif).unwrap()
                    }
                };

                let content_length = json_bytes.len();
                let header = format!("Content-Length: {}\r\n\r\n", content_length);
                
                if writer.write_all(header.as_bytes()).await.is_err() {
                    break;
                }
                if writer.write_all(&json_bytes).await.is_err() {
                    break;
                }
                if writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Reader task
        let pending_reqs_clone = Arc::clone(&pending_requests);
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut buf = Vec::new();
            
            loop {
                let mut content_length: Option<usize> = None;
                loop {
                    let mut line = String::new();
                    let mut b = [0; 1];
                    while reader.read_exact(&mut b).await.is_ok() {
                        line.push(b[0] as char);
                        if line.ends_with("\r\n") {
                            break;
                        }
                    }
                    
                    if line.is_empty() {
                        return; // EOF
                    }
                    
                    if line == "\r\n" {
                        break;
                    }
                    
                    if line.starts_with("Content-Length:") {
                        if let Some(val) = line.strip_prefix("Content-Length:").and_then(|s| s.trim().parse::<usize>().ok()) {
                            content_length = Some(val);
                        }
                    }
                }
                
                if let Some(len) = content_length {
                    buf.resize(len, 0);
                    if reader.read_exact(&mut buf).await.is_ok() {
                        if let Ok(value) = serde_json::from_slice::<Value>(&buf) {
                            if let Ok(resp) = serde_json::from_value::<Response>(value.clone()) {
                                if let Some(tx) = pending_reqs_clone.lock().await.remove(&resp.id) {
                                    let _ = tx.send(resp);
                                }
                            } else if let Ok(notif) = serde_json::from_value::<Notification>(value) {
                                if notif.method == "textDocument/publishDiagnostics" {
                                    if let Some(params) = notif.params {
                                        if let Ok(diag_params) = serde_json::from_value::<lsp_types::PublishDiagnosticsParams>(params) {
                                            let _ = diag_tx.send(diag_params).await;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            next_id: AtomicI32::new(1),
            sender: tx,
            diagnostics_rx: Some(diag_rx),
        })
    }
    
    pub async fn initialize(&self, root_uri: Option<lsp_types::Uri>) -> Result<lsp_types::InitializeResult> {
        #[allow(deprecated)]
        let params = lsp_types::InitializeParams {
            process_id: Some(std::process::id()),
            root_uri,
            capabilities: lsp_types::ClientCapabilities::default(),
            ..Default::default()
        };
        
        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            method: "initialize".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Request(req, tx)).await.context("Failed to send initialize request")?;
        
        let resp = rx.await.context("Failed to receive initialize response")?;
        if let Some(err) = resp.error {
            bail!("LSP Error: {:?}", err);
        }
        
        let result = resp.result.context("No result in response")?;
        let init_result: lsp_types::InitializeResult = serde_json::from_value(result)?;
        
        Ok(init_result)
    }

    pub async fn text_document_did_open(&self, params: lsp_types::DidOpenTextDocumentParams) -> Result<()> {
        let notif = Notification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/didOpen".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        self.sender.send(Message::Notification(notif)).await.context("Failed to send notification")?;
        Ok(())
    }

    pub async fn text_document_did_change(&self, params: lsp_types::DidChangeTextDocumentParams) -> Result<()> {
        let notif = Notification {
            jsonrpc: "2.0".to_string(),
            method: "textDocument/didChange".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        self.sender.send(Message::Notification(notif)).await.context("Failed to send notification")?;
        Ok(())
    }

    pub async fn text_document_completion(&self, params: lsp_types::CompletionParams) -> Result<lsp_types::CompletionResponse> {
        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            method: "textDocument/completion".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Request(req, tx)).await.context("Failed to send request")?;
        
        let resp = rx.await.context("Failed to receive response")?;
        if let Some(err) = resp.error {
            bail!("LSP Error: {:?}", err);
        }
        
        let result = resp.result.context("No result in response")?;
        let comp_result: lsp_types::CompletionResponse = serde_json::from_value(result)?;
        Ok(comp_result)
    }

    pub async fn text_document_semantic_tokens_full(&self, params: lsp_types::SemanticTokensParams) -> Result<Option<lsp_types::SemanticTokensResult>> {
        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            method: "textDocument/semanticTokens/full".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Request(req, tx)).await.context("Failed to send request")?;
        let resp = rx.await.context("Failed to receive response")?;
        if let Some(err) = resp.error { bail!("LSP Error: {:?}", err); }
        let result = resp.result.context("No result in response")?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn text_document_signature_help(&self, params: lsp_types::SignatureHelpParams) -> Result<Option<lsp_types::SignatureHelp>> {
        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            method: "textDocument/signatureHelp".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Request(req, tx)).await.context("Failed to send request")?;
        let resp = rx.await.context("Failed to receive response")?;
        if let Some(err) = resp.error { bail!("LSP Error: {:?}", err); }
        let result = resp.result.context("No result in response")?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn text_document_definition(&self, params: lsp_types::GotoDefinitionParams) -> Result<Option<lsp_types::GotoDefinitionResponse>> {
        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            method: "textDocument/definition".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Request(req, tx)).await.context("Failed to send request")?;
        let resp = rx.await.context("Failed to receive response")?;
        if let Some(err) = resp.error { bail!("LSP Error: {:?}", err); }
        let result = resp.result.context("No result in response")?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn text_document_rename(&self, params: lsp_types::RenameParams) -> Result<Option<lsp_types::WorkspaceEdit>> {
        let req = Request {
            jsonrpc: "2.0".to_string(),
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            method: "textDocument/rename".to_string(),
            params: Some(serde_json::to_value(params)?),
        };
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Request(req, tx)).await.context("Failed to send request")?;
        let resp = rx.await.context("Failed to receive response")?;
        if let Some(err) = resp.error { bail!("LSP Error: {:?}", err); }
        let result = resp.result.context("No result in response")?;
        Ok(serde_json::from_value(result)?)
    }
}
