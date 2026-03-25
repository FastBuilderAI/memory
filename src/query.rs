use serde_json::Value;

pub fn search_memory(memory_json: &str, query: &str) -> String {
    let memory: Value = serde_json::from_str(memory_json).unwrap_or(Value::Null);
    let mut results = Vec::new();

    if let Some(blocks) = memory.as_array() {
        for block in blocks {
            if let Some(deepest) = extract_deepest_matching_block(block, query) {
                results.push(deepest);
            }
        }
    }

    serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string())
}

fn has_match(val: &Value, query: &str) -> bool {
    let q = query.to_lowercase();
    if let Some(obj) = val.as_object() {
        if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
            if name.to_lowercase().contains(&q) { return true; }
        }
        if let Some(action) = obj.get("action").and_then(|v| v.as_str()) {
            if action.to_lowercase().contains(&q) { return true; }
        }
        if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
            if id.to_lowercase().contains(&q) { return true; }
        }
        
        if let Some(nodes) = obj.get("nodes").and_then(|v| v.as_array()) {
            for n in nodes {
                if has_match(n, query) { return true; }
            }
        }
        if let Some(sub_blocks) = obj.get("sub_blocks").and_then(|v| v.as_array()) {
            for b in sub_blocks {
                if has_match(b, query) { return true; }
            }
        }
    }
    false
}

fn extract_deepest_matching_block(block: &Value, query: &str) -> Option<Value> {
    if !has_match(block, query) { return None; }
    
    // Check sub_blocks first to see if the match is deeper down
    if let Some(sub_blocks) = block.get("sub_blocks").and_then(|v| v.as_array()) {
        let mut deeper_matches = Vec::new();
        for sb in sub_blocks {
            if let Some(m) = extract_deepest_matching_block(sb, query) {
                deeper_matches.push(m);
            }
        }
        if !deeper_matches.is_empty() {
            // Return current block but ONLY with matching sub_blocks.
            let mut clone = block.clone();
            clone["sub_blocks"] = Value::Array(deeper_matches);
            return Some(clone);
        }
    }
    
    // Check if immediate nodes match
    let mut node_matches = false;
    if let Some(nodes) = block.get("nodes").and_then(|v| v.as_array()) {
        for n in nodes {
            if has_match(n, query) {
                node_matches = true;
                break;
            }
        }
    }
    
    // Check block's own name/id
    let q = query.to_lowercase();
    let self_matches = block.get("name").and_then(|v| v.as_str()).map(|s| s.to_lowercase().contains(&q)).unwrap_or(false) ||
                       block.get("id").and_then(|v| v.as_str()).map(|s| s.to_lowercase().contains(&q)).unwrap_or(false);

    if node_matches || self_matches {
        let mut clone = block.clone();
        clone["sub_blocks"] = Value::Array(Vec::new()); // Strip deep sub-blocks to isolate the context tightly
        return Some(clone);
    }
    
    None
}
