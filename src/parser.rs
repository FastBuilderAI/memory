use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atf {
    pub id: String,
    pub action: String,
    pub input: String,
    pub logic: String,
    pub data_connections: Vec<String>,
    pub access: String,
    pub events: String,
}

pub fn parse_markdown(text: &str) -> Vec<Atf> {
    let mut atfs = Vec::new();
    let paragraphs: Vec<&str> = text.split("\n\n").collect();
    
    let mut current_component = String::new();
    let mut current_block = String::new();
    
    let re_entity = Regex::new(r"\((Component|Block|Function|Data|Access|Event)\s+([A-Za-z0-9_]+)\)").unwrap();

    for para in paragraphs {
        let mut functions = Vec::new();
        let mut data = Vec::new();
        let mut access = Vec::new();
        let mut events = Vec::new();

        for cap in re_entity.captures_iter(para) {
            let entity_type = &cap[1];
            let entity_name = cap[2].to_string();
            
            match entity_type {
                "Component" => current_component = entity_name,
                "Block" => current_block = entity_name,
                "Function" => functions.push(entity_name),
                "Data" => data.push(entity_name),
                "Access" => access.push(entity_name),
                "Event" => events.push(entity_name),
                _ => {}
            }
        }
        
        // Map to functions, or fallback to block/component if paragraph only has data/events
        let target_functions = if functions.is_empty() {
            if !current_block.is_empty() {
                vec![current_block.clone()]
            } else if !current_component.is_empty() {
                vec![current_component.clone()]
            } else {
                continue; // Nothing to anchor to
            }
        } else {
            functions
        };

        for f_name in target_functions {
            let mut all_data = data.clone();
            if !current_component.is_empty() && f_name != current_component {
                all_data.push(current_component.clone());
            }
            if !current_block.is_empty() && f_name != current_block {
                all_data.push(current_block.clone());
            }

            atfs.push(Atf {
                id: f_name.clone(),
                action: "Dynamic Parse".to_string(),
                input: String::new(),
                logic: String::new(),
                data_connections: all_data,
                access: access.join(","),
                events: events.join(","),
            });
        }
    }
    
    atfs
}
