use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: Vec<String>,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub is_updated: bool,
    pub existing_id: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedRelationship {
    pub from_name: String,
    pub to_name: String,
    pub rel_type: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedConflict {
    pub entity_id: String,
    pub field: String,
    pub existing_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub severity: String,
    pub explanation: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExtraction {
    pub entities: Vec<ExtractedEntity>,
    pub relationships: Vec<ExtractedRelationship>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictExtraction {
    pub conflicts: Vec<ExtractedConflict>
}