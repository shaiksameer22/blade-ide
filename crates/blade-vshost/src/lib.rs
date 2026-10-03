use deno_core::{op2, extension, JsRuntime, RuntimeOptions};

#[op2(fast)]
pub fn op_vscode_window_show_information_message(#[string] msg: String) {
    println!("[VSCode API] Information Message: {}", msg);
}

extension!(
    vscode_api,
    ops = [op_vscode_window_show_information_message]
);

pub fn start_extension_host(extension_js: &str) -> anyhow::Result<()> {
    let mut runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![vscode_api::init_ops_and_esm()],
        ..Default::default()
    });

    let mock_vscode = r#"
        globalThis.vscode = {
            window: {
                showInformationMessage: (msg) => {
                    Deno.core.ops.op_vscode_window_show_information_message(msg);
                }
            }
        };
    "#;

    runtime.execute_script("vscode_mock.js", mock_vscode)?;
    runtime.execute_script("extension.js", extension_js.to_string())?;

    Ok(())
}
