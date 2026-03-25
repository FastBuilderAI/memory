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
    let re = Regex::new(r"(?s)## \[ID: (.*?)\]\s*\*\*Action:\*\* (.*?)\s*\*\*Input:\*\* (.*?)\s*\*\*Logic:\*\* (.*?)\s*\*\*Data_Connections:\*\* (.*?)\s*\*\*Access:\*\* (.*?)\s*\*\*Events:\*\* (.*?)(?:\n##|\n*$)").unwrap();
    let mut atfs = Vec::new();

    for cap in re.captures_iter(text) {
        let links_str = cap[5].trim();
        let links_re = Regex::new(r"\[(.*?)\]").unwrap();
        let data_connections: Vec<String> = links_re.captures_iter(links_str).map(|c| c[1].to_string()).collect();

        atfs.push(Atf {
            id: cap[1].trim().to_string(),
            action: cap[2].trim().to_string(),
            input: cap[3].trim().to_string(),
            logic: cap[4].trim().to_string(),
            data_connections,
            access: cap[6].trim().to_string(),
            events: cap[7].trim().to_string(),
        });
    }

    atfs
}
