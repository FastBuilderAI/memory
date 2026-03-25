use std::io::{self, BufRead, Write};
use serde_json::{Value, json};

pub async fn start_mcp_server(memory_json: String) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if let Ok(req) = serde_json::from_str::<Value>(&line) {
                if let Some(method) = req.get("method").and_then(|v| v.as_str()) {
                    let id = req.get("id").cloned().unwrap_or(Value::Null);
                    let mut response = json!({
                        "jsonrpc": "2.0",
                        "id": id,
                    });

                    match method {
                        "initialize" => {
                            response["result"] = json!({
                                "protocolVersion": "2024-11-05",
                                "capabilities": {},
                                "serverInfo": { "name": "fastmemory", "version": "0.1.0" }
                            });
                        }
                        "tools/list" => {
                            response["result"] = json!({
                                "tools": [
                                    {
                                        "name": "query_memory",
                                        "description": "Query unstructured CBFDAE graph memory clusters for a specific keyword or entity",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "query": { "type": "string" }
                                            },
                                            "required": ["query"]
                                        }
                                    }
                                ]
                            });
                        }
                        "tools/call" => {
                            let tool_name = req.get("params").and_then(|p| p.get("name")).and_then(|n| n.as_str()).unwrap_or("");
                            if tool_name == "query_memory" {
                                let q = req.get("params")
                                    .and_then(|p| p.get("arguments"))
                                    .and_then(|a| a.get("query"))
                                    .and_then(|q| q.as_str())
                                    .unwrap_or("");
                                
                                let findings = crate::query::search_memory(&memory_json, q);
                                response["result"] = json!({
                                    "content": [
                                        {
                                            "type": "text",
                                            "text": findings
                                        }
                                    ]
                                });
                            } else {
                                response["error"] = json!({ "code": -32601, "message": "Method not found" });
                            }
                        }
                        _ => {
                            // ignore others
                        }
                    }

                    if response.get("result").is_some() || response.get("error").is_some() {
                        if let Ok(out) = serde_json::to_string(&response) {
                            println!("{}", out);
                            let _ = stdout.flush();
                        }
                    }
                }
            }
        }
    }
}
