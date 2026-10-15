use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use neo4rs::{Node, Relation, Row};
use uuid::Uuid;
use crate::data::{ExtractedConflict, ExtractedEntity, ExtractedRelationship};
use crate::infrastructure::InfrastructureError;

pub trait FromRow: Sized {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub world_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: Vec<String>,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl Entity {
    pub fn from_extraction(extracted_entity: ExtractedEntity, world_id: Uuid) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            world_id: world_id.to_string(),
            name: extracted_entity.name,
            aliases: extracted_entity.aliases,
            kind: extracted_entity.kind,
            description: extracted_entity.description,
            properties: extracted_entity.properties
        }
    }
}


impl FromRow for Entity {
    fn from_row(row: &Row, alias: &str) -> Result<Self, InfrastructureError> {
        let node: Node = row.get(alias)?;
        Ok(Entity {
            id: node.get("id")?,
            world_id: node.get("world_id")?,
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
    pub world_id: String,
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub source_conversation: String,
}

impl Relationship {
    pub fn from_extraction(
        extracted_relationship: ExtractedRelationship,
        world_id: Uuid,
        source_conversation_id: Uuid
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            world_id: world_id.to_string(),
            from_id: extracted_relationship.from_name,
            to_id: extracted_relationship.to_name,
            rel_type: extracted_relationship.rel_type,
            description: extracted_relationship.description,
            source_conversation: source_conversation_id.to_string(),
            properties: extracted_relationship.properties
        }
    }
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
            properties: serde_json::from_str(&rel.get::<String>("properties")?)
                .unwrap_or_default(),
            source_conversation: rel.get("source_conversation")?,
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
}

impl Conflict {
    pub fn from_extraction(
        extracted_conflict: ExtractedConflict,
        world_id: Uuid,
        source_conversation_id: Uuid
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            world_id: world_id.to_string(),
            entity_id: extracted_conflict.entity_id,
            field: extracted_conflict.field,
            existing_value: extracted_conflict.existing_value,
            new_value: extracted_conflict.new_value,
            severity: extracted_conflict.severity,
            explanation: extracted_conflict.explanation,
            source_conversation: source_conversation_id.to_string(),
            resolved: false
        }
    }
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
            new_value: serde_json::from_str(&node.get::<String>("new_value")?)
                .unwrap_or_default(),
            severity: node.get("severity")?,
            explanation: node.get("explanation")?,
            source_conversation: node.get("source_conversation")?,
            resolved: node.get("resolved")?,
        })
    }
}