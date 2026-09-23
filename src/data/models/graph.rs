use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use neo4rs::{Node, Relation, Row};
use crate::infrastructure::InfrastructureError;

pub trait FromRow: Sized {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: Vec<String>,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl FromRow for Entity {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Entity {
            id: node.get("id")?,
            name: node.get("name")?,
            aliases: node.get("aliases")?,
            kind: node.get("kind")?,
            description: node.get("description")?,
            properties: serde_json::from_str(&node.get::<String>("properties")?)
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub source_conversation: String,
}

impl FromRow for Relationship {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let rel: Relation = row.get(alias)?;
        Ok(Relationship {
            id: rel.get("id")?,
            from_id: rel.get("from_id")?,
            to_id: rel.get("to_id")?,
            rel_type: rel.typ().to_string(),
            description: rel.get("description")?,
            properties: serde_json::from_str(&rel.get::<String>("properties")?)
                .unwrap_or_default(),
            source_conversation: rel.get("source_conversation")?,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub created_at: String,
    pub summary: Option<String>,
}

impl FromRow for Conversation {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Conversation {
            id: node.get("id")?,
            created_at: node.get("created_at")?,
            summary: node.get::<String>("summary").ok(),
        })
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub id: String,
    pub entity_id: String,
    pub field: String,
    pub existing_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub source_conversation: String,
    pub resolved: bool,
}

impl FromRow for Conflict {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Conflict {
            id: node.get("id")?,
            entity_id: node.get("entity_id")?,
            field: node.get("field")?,
            existing_value: serde_json::from_str(&node.get::<String>("existing_value")?)
                .unwrap_or_default(),
            new_value: serde_json::from_str(&node.get::<String>("new_value")?)
                .unwrap_or_default(),
            source_conversation: node.get("source_conversation")?,
            resolved: node.get("resolved")?,
        })
    }
}