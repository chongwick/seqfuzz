use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct LangFile {
    #[serde(rename = "OpNodes")]
    op_nodes: OpNodes,
}

#[derive(Debug, Deserialize)]
struct OpNodes {
    #[serde(rename = "MiddleOpNodes")]
    middle_op_nodes: HashMap<String, String>,
    #[serde(rename = "FrontOpNodes")]
    front_op_nodes: HashMap<String, String>,
    #[serde(rename = "RearOpNodes")]
    rear_op_nodes: HashMap<String, String>,
    #[serde(rename = "InnerOpNodes")]
    inner_op_nodes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LangDescription {
    pub middle_ops: HashMap<String, String>,
    pub front_ops: HashMap<String, String>,
    pub rear_ops: HashMap<String, String>,
    pub inner_ops: HashMap<String, String>,
}

impl LangDescription {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load_from("php_lang_description.json")
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let lang_file: LangFile = serde_json::from_str(&data)?;
        Ok(LangDescription {
            middle_ops: lang_file.op_nodes.middle_op_nodes,
            front_ops: lang_file.op_nodes.front_op_nodes,
            rear_ops: lang_file.op_nodes.rear_op_nodes,
            inner_ops: lang_file.op_nodes.inner_op_nodes,
        })
    }
}
