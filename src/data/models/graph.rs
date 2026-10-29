use crate::infrastructure::InfrastructureError;
use neo4rs::{Node, Relation, Row};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait FromRow: Sized {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entity {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub name_key: String,
    pub aliases: Vec<String>,
    pub kind: Vec<String>,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub source_message_ids: Vec<String>,
}

/// Normalized name for search and legacy lookups. IDs identify saved entities.
pub fn name_key(name: &str) -> String {
    name.trim().to_lowercase()
}

impl FromRow for Entity {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Entity {
            id: node.get("id")?,
            world_id: node.get("world_id")?,
            name: node.get("name")?,
            name_key: node.get("name_key")?,
            aliases: node.get("aliases")?,
            source_message_ids: if node.keys().contains(&"source_message_ids") {
                node.get("source_message_ids")?
            } else {
                Vec::new()
            },
            kind: node.get("kind")?,
            description: node.get("description")?,
            properties: serde_json::from_str(&node.get::<String>("properties")?)
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Relationship {
    pub id: String,
    pub world_id: String,
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub source_message_ids: Vec<String>,
    pub source_conversation: String,
}

impl FromRow for Relationship {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let rel: Relation = row.get(alias)?;
        Ok(Relationship {
            id: rel.get("id")?,
            world_id: rel.get("world_id")?,
            from_id: rel.get("from_id")?,
            to_id: rel.get("to_id")?,
            rel_type: rel.typ().to_string(),
            description: rel.get("description")?,
            properties: serde_json::from_str(&rel.get::<String>("properties")?).unwrap_or_default(),
            source_conversation: rel.get("source_conversation")?,
            source_message_ids: if rel.keys().contains(&"source_message_ids") {
                rel.get("source_message_ids")?
            } else {
                Vec::new()
            },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub world_id: String,
    pub created_at: String,
    pub summary: Option<String>,
}

impl FromRow for Conversation {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Conversation {
            id: node.get("id")?,
            world_id: node.get("world_id")?,
            created_at: node.get("created_at")?,
            summary: node.get::<String>("summary").ok(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub world_id: String,
    pub entity_id: String,
    pub field: String,
    pub existing_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub severity: String,
    pub explanation: String,
    pub source_conversation: String,
    pub resolved: bool,
    #[serde(default)]
    pub source_message_ids: Vec<String>,
}

impl FromRow for Conflict {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Conflict {
            id: node.get("id")?,
            world_id: node.get("world_id")?,
            entity_id: node.get("entity_id")?,
            field: node.get("field")?,
            existing_value: serde_json::from_str(&node.get::<String>("existing_value")?)
                .unwrap_or_default(),
            new_value: serde_json::from_str(&node.get::<String>("new_value")?).unwrap_or_default(),
            severity: node.get("severity")?,
            explanation: node.get("explanation")?,
            source_conversation: node.get("source_conversation")?,
            resolved: node.get("resolved")?,
            source_message_ids: if node.keys().contains(&"source_message_ids") {
                node.get("source_message_ids")?
            } else {
                Vec::new()
            },
        })
    }
}
