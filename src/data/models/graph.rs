use crate::infrastructure::InfrastructureError;
use crate::shared::{PageResponse, SearchQuery};
use neo4rs::{Node, Relation, Row};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

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

// HTTP query models. Cursor decoding belongs to the repository pagination implementation.
fn default_page_limit() -> u32 {
    50
}

#[derive(Debug, Deserialize, Validate)]
pub struct GraphQuery {
    #[serde(default = "default_page_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u32,
    #[validate(length(max = 2048))]
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EntityListQuery {
    #[serde(default = "default_page_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u32,
    #[validate(length(max = 2048))]
    pub cursor: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct EntitySearchQuery {
    pub query: SearchQuery,
    #[serde(default = "default_page_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct NeighborQuery {
    #[serde(default = "default_page_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u32,
    #[validate(length(max = 2048))]
    pub cursor: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub kind: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub rel_type: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConflictListQuery {
    #[serde(default = "default_page_limit")]
    #[validate(range(min = 1, max = 100))]
    pub limit: u32,
    #[validate(length(max = 2048))]
    pub cursor: Option<String>,
    /// Omit to include both resolved and unresolved conflicts.
    pub resolved: Option<bool>,
}

/// Nodes and edges for initial graph loading or neighborhood expansion.
/// Pagination metadata belongs to PageResponse<GraphResponse>.
#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<EntityResponse>,
    pub edges: Vec<RelationshipResponse>,
}

#[derive(Debug, Serialize)]
pub struct EntityResponse {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: Vec<String>,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub source_message_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RelationshipResponse {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub source_conversation: String,
    pub source_message_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ConflictResponse {
    pub id: String,
    pub entity_id: String,
    pub field: String,
    pub existing_value: serde_json::Value,
    pub new_value: serde_json::Value,
    pub severity: String,
    pub explanation: String,
    pub source_conversation: String,
    pub source_message_ids: Vec<String>,
    pub resolved: bool,
}

impl From<Entity> for EntityResponse {
    fn from(entity: Entity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            aliases: entity.aliases,
            kind: entity.kind,
            description: entity.description,
            properties: entity.properties,
            source_message_ids: entity.source_message_ids,
        }
    }
}

impl From<Relationship> for RelationshipResponse {
    fn from(relationship: Relationship) -> Self {
        Self {
            id: relationship.id,
            from_id: relationship.from_id,
            to_id: relationship.to_id,
            rel_type: relationship.rel_type,
            description: relationship.description,
            properties: relationship.properties,
            source_conversation: relationship.source_conversation,
            source_message_ids: relationship.source_message_ids,
        }
    }
}

impl From<Conflict> for ConflictResponse {
    fn from(conflict: Conflict) -> Self {
        Self {
            id: conflict.id,
            entity_id: conflict.entity_id,
            field: conflict.field,
            existing_value: conflict.existing_value,
            new_value: conflict.new_value,
            severity: conflict.severity,
            explanation: conflict.explanation,
            source_conversation: conflict.source_conversation,
            source_message_ids: conflict.source_message_ids,
            resolved: conflict.resolved,
        }
    }
}

impl From<(Vec<Entity>, Vec<Relationship>)> for GraphResponse {
    fn from((entities, relationships): (Vec<Entity>, Vec<Relationship>)) -> Self {
        Self {
            nodes: entities.into_iter().map(EntityResponse::from).collect(),
            edges: relationships
                .into_iter()
                .map(RelationshipResponse::from)
                .collect(),
        }
    }
}

impl From<Vec<(Entity, Relationship)>> for GraphResponse {
    fn from(neighbors: Vec<(Entity, Relationship)>) -> Self {
        let (entities, relationships) = neighbors.into_iter().unzip();
        Self::from((entities, relationships))
    }
}

impl From<(Vec<(Entity, Relationship)>, Option<String>)> for PageResponse<GraphResponse> {
    fn from((neighbors, next_cursor): (Vec<(Entity, Relationship)>, Option<String>)) -> Self {
        Self {
            data: neighbors.into(),
            next_cursor,
        }
    }
}

impl From<(Vec<Entity>, Option<String>)> for PageResponse<Vec<EntityResponse>> {
    fn from((entities, next_cursor): (Vec<Entity>, Option<String>)) -> Self {
        Self {
            data: entities.into_iter().map(EntityResponse::from).collect(),
            next_cursor,
        }
    }
}

impl From<(Vec<Relationship>, Option<String>)> for PageResponse<Vec<RelationshipResponse>> {
    fn from((relationships, next_cursor): (Vec<Relationship>, Option<String>)) -> Self {
        Self {
            data: relationships
                .into_iter()
                .map(RelationshipResponse::from)
                .collect(),
            next_cursor,
        }
    }
}

impl From<(Vec<Conflict>, Option<String>)> for PageResponse<Vec<ConflictResponse>> {
    fn from((conflicts, next_cursor): (Vec<Conflict>, Option<String>)) -> Self {
        Self {
            data: conflicts.into_iter().map(ConflictResponse::from).collect(),
            next_cursor,
        }
    }
}

impl From<(Vec<Entity>, Vec<Relationship>, Option<String>)> for PageResponse<GraphResponse> {
    fn from(
        (entities, relationships, next_cursor): (Vec<Entity>, Vec<Relationship>, Option<String>),
    ) -> Self {
        Self {
            data: (entities, relationships).into(),
            next_cursor,
        }
    }
}
