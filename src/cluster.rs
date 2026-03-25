use std::process::{Command, Stdio};
use std::io::Write;
use serde::{Serialize, Deserialize};
use std::env::consts;
use std::fs;
use std::env::temp_dir;

#[derive(Serialize)]
struct Edge {
    source: String,
    target: String,
}

const MACOS_BIN: &[u8] = include_bytes!("../bin/macos/rust-louvain");
const LINUX_BIN: &[u8] = include_bytes!("../bin/linux/rust-louvain");
const WINDOWS_BIN: &[u8] = include_bytes!("../bin/windows/rust-louvain.exe");

pub fn run_louvain(edges: &Vec<(String, String)>, atfs: &Vec<crate::parser::Atf>) -> String {
    let bin_data = match consts::OS {
        "macos" => MACOS_BIN,
        "linux" => LINUX_BIN,
        "windows" => WINDOWS_BIN,
        _ => MACOS_BIN,
    };

    let ext = if consts::OS == "windows" { ".exe" } else { "" };
    let temp_file = temp_dir().join(format!("rust-louvain-{}", ext));
    
    // Write binary to temp file and make executable if necessary
    fs::write(&temp_file, bin_data).expect("Failed to write to temp file");
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_file).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_file, perms).unwrap();
    }

    // Run the bin
    let mut child = Command::new(&temp_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start rust-louvain binary");

    let edge_objects: Vec<Edge> = edges.iter().map(|e| Edge { source: e.0.clone(), target: e.1.clone() }).collect();
    let json_input = serde_json::to_string(&edge_objects).unwrap();

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(json_input.as_bytes()).unwrap();
    }

    let output = child.wait_with_output().unwrap();
    let result_json = String::from_utf8_lossy(&output.stdout).to_string();

    // Clean up
    let _ = fs::remove_file(temp_file);

    // Map Louvain generic naming to CBFDAE schema
    if let Ok(mut parsed) = serde_json::from_str::<serde_json::Value>(&result_json) {
        let mut atf_map = std::collections::HashMap::new();
        for atf in atfs {
            atf_map.insert(format!("F_{}", atf.id), serde_json::json!({
                "id": format!("F_{}", atf.id),
                "cbfdae_level": "Function",
                "action": atf.action,
                "data_connections": atf.data_connections.iter().map(|d| format!("D_{}", d)).collect::<Vec<_>>(),
                "access": atf.access.split(',').map(|s| format!("A_{}", s.trim())).filter(|s| s.len() > 2).collect::<Vec<_>>(),
                "events": atf.events.split(',').map(|s| format!("E_{}", s.trim())).filter(|s| s.len() > 2).collect::<Vec<_>>()
            }));
        }

        // We can't capture atf_map inside the recursive process_block, 
        // so we just define it outside and mutate via an inner function.
        let mut parsed_clone = parsed.clone();

        fn process_block(val: &mut serde_json::Value, map: &std::collections::HashMap<String, serde_json::Value>) {
            if let Some(obj) = val.as_object_mut() {
                let depth = obj.get("depth").and_then(|v| v.as_u64()).unwrap_or(0);
                
                let original_name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("Layer_Unknown").to_string();
                
                if depth == 0 {
                    obj.insert("cbfdae_level".to_string(), serde_json::Value::String("Component".to_string()));
                    obj.insert("name".to_string(), serde_json::Value::String(format!("C - {}", original_name)));
                } else {
                    obj.insert("cbfdae_level".to_string(), serde_json::Value::String("Block".to_string()));
                    obj.insert("name".to_string(), serde_json::Value::String(format!("B - {}", original_name)));
                }
                obj.insert("block_type".to_string(), serde_json::Value::String("cbfdae_memory".to_string()));
                
                if let Some(nodes) = obj.get_mut("nodes").and_then(|v| v.as_array_mut()) {
                    let mut full_nodes = Vec::new();
                    for node in nodes.iter() {
                        if let Some(node_id) = node.as_str() {
                            if let Some(full_obj) = map.get(node_id) {
                                full_nodes.push(full_obj.clone());
                            } else if node_id.starts_with("F_") {
                                full_nodes.push(serde_json::json!({"id": node_id, "action": &node_id[2..], "cbfdae_level": "Function"}));
                            } else if node_id.starts_with("D_") {
                                full_nodes.push(serde_json::json!({"id": node_id, "action": &node_id[2..], "cbfdae_level": "Data"}));
                            } else if node_id.starts_with("A_") {
                                full_nodes.push(serde_json::json!({"id": node_id, "action": &node_id[2..], "cbfdae_level": "Access"}));
                            } else if node_id.starts_with("E_") {
                                full_nodes.push(serde_json::json!({"id": node_id, "action": &node_id[2..], "cbfdae_level": "Event"}));
                            } else {
                                full_nodes.push(serde_json::json!({"id": node_id, "action": node_id, "cbfdae_level": "Unknown"}));
                            }
                        } else if node.is_object() {
                            // If it's already an object, preserve it
                            full_nodes.push(node.clone());
                        }
                    }
                    *nodes = full_nodes;
                }

                if let Some(sub_blocks) = obj.get_mut("sub_blocks").and_then(|v| v.as_array_mut()) {
                    for sub in sub_blocks.iter_mut() {
                        process_block(sub, map);
                    }
                }
            }
        }

        if let Some(arr) = parsed.as_array_mut() {
            for mut root_block in arr.iter_mut() {
                process_block(&mut root_block, &atf_map);
            }
        }
        
        return serde_json::to_string(&parsed).unwrap_or(result_json);
    }

    result_json
}
