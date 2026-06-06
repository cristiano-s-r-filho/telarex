use tokio::process::{Command, Child, ChildStdin};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader, AsyncReadExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct LspClient {
    _child: Child,
    writer: Arc<tokio::sync::Mutex<ChildStdin>>,
    request_id: Arc<Mutex<i64>>,
    completion_rx: mpsc::Receiver<Vec<CompletionItem>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<i32>,
    pub detail: Option<String>,
    #[serde(rename = "insertText")]
    pub insert_text: Option<String>,
}

impl LspClient {
    pub fn start(command: &str, _root: &std::path::Path, event_tx: mpsc::Sender<Value>) -> anyhow::Result<Self> {
        let mut child = Command::new(command)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()?;

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");
        
        let (c_tx, c_rx) = mpsc::channel(100);
        let writer = Arc::new(tokio::sync::Mutex::new(stdin));
        let request_id = Arc::new(Mutex::new(0));

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut line = String::new();
                let mut content_length = None;
                loop {
                    line.clear();
                    if reader.read_line(&mut line).await.is_err() || line.is_empty() { return; }
                    if line == "\r\n" || line == "\n" { break; }
                    if line.to_lowercase().starts_with("content-length: ") {
                        if let Ok(len) = line["content-length: ".len()..].trim().parse::<usize>() { content_length = Some(len); }
                    }
                }
                if let Some(len) = content_length {
                    let mut body = vec![0u8; len];
                    if reader.read_exact(&mut body).await.is_ok() {
                        if let Ok(json) = serde_json::from_slice::<Value>(&body) {
                            // Check if it's a completion response
                            if let Some(result) = json.get("result") {
                                if let Ok(items) = serde_json::from_value::<Vec<CompletionItem>>(result.clone()) {
                                    let _ = c_tx.send(items).await;
                                } else if let Some(items_obj) = result.get("items") {
                                    if let Ok(items) = serde_json::from_value::<Vec<CompletionItem>>(items_obj.clone()) {
                                        let _ = c_tx.send(items).await;
                                    }
                                }
                            }
                            let _ = event_tx.send(json).await; 
                        }
                    }
                }
            }
        });

        Ok(Self { _child: child, writer, request_id, completion_rx: c_rx })
    }

    pub fn try_recv_completions(&mut self) -> Result<Vec<CompletionItem>, mpsc::error::TryRecvError> {
        self.completion_rx.try_recv()
    }

    pub fn notify_open(&self, path: &std::path::Path, lang: &str, text: String) {
        let uri = format!("file:///{}", path.display());
        let params = json!({ "textDocument": { "uri": uri, "languageId": lang, "version": 1, "text": text } });
        self.notify("textDocument/didOpen", params);
    }

    pub fn notify_change(&self, uri: &str, version: i32, text: String) {
        let params = json!({ "textDocument": { "uri": uri, "version": version }, "contentChanges": [{ "text": text }] });
        self.notify("textDocument/didChange", params);
    }

    pub fn request_completions(&self, uri: &str, line: usize, col: usize) {
        let id = self.next_id();
        let params = json!({
            "textDocument": { "uri": uri },
            "position": { "line": line.saturating_sub(1), "character": col.saturating_sub(1) }
        });
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": "textDocument/completion", "params": params });
        let writer = self.writer.clone();
        tokio::spawn(async move {
            if let Ok(body) = serde_json::to_string(&request) {
                let content = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
                let mut w = writer.lock().await;
                let _ = w.write_all(content.as_bytes()).await;
                let _ = w.flush().await;
            }
        });
    }

    fn notify(&self, method: &str, params: Value) {
        let notification = json!({ "jsonrpc": "2.0", "method": method, "params": params });
        let writer = self.writer.clone();
        tokio::spawn(async move {
            if let Ok(body) = serde_json::to_string(&notification) {
                let content = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
                let mut w = writer.lock().await;
                let _ = w.write_all(content.as_bytes()).await;
                let _ = w.flush().await;
            }
        });
    }

    fn next_id(&self) -> i64 {
        let mut id = self.request_id.lock().unwrap();
        *id += 1;
        *id
    }
}