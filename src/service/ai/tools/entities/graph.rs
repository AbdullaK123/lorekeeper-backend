use std::collections::HashMap;

use crate::{
    data::{Entity, GraphRepository, Relationship, name_key},
    service::ServiceError,
};
use rig::{DynModel, operation::Embedding, tool::PortableTool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct UpdateGraph {
    pub world_id: String,
    pub conversation_id: String,
    pub source_message_id: String,
    pub graph_repo: GraphRepository,
    pub embedding_model: DynModel<Embedding>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GraphEntityInput {
    /// Exact saved ID for updates. Omit or use null for a new subject.
    pub id: Option<String>,
    pub name: String,
    pub aliases: Vec<String>,
    pub kind: Vec<String>,
    /// Complete updated description. Preserve earlier facts when adding details.
    pub description: String,
    /// Complete updated properties. Omitted properties are removed.
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, JsonSchema)]
pub struct GraphRelationshipInput {
    /// Exact saved ID for updates. Omit or use null for a new relationship.
    pub id: Option<String>,
    pub from_id: String,
    pub to_id: String,
    pub rel_type: String,
    pub description: String,
    pub properties: HashMap<String, serde_json::Value>,
}

impl From<(GraphEntityInput, &str)> for Entity {
    fn from((input, world_id): (GraphEntityInput, &str)) -> Self {
        Self {
            id: input.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            world_id: world_id.to_owned(),
            name_key: name_key(&input.name),
            name: input.name,
            aliases: input.aliases,
            kind: input.kind,
            description: input.description,
            properties: input.properties,
            source_message_ids: Vec::new(),
        }
    }
}

impl From<(GraphRelationshipInput, &str, &str)> for Relationship {
    fn from((input, world_id, conversation_id): (GraphRelationshipInput, &str, &str)) -> Self {
        Self {
            id: input.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            world_id: world_id.to_owned(),
            from_id: input.from_id,
            to_id: input.to_id,
            rel_type: input.rel_type,
            description: input.description,
            properties: input.properties,
            source_message_ids: Vec::new(),
            source_conversation: conversation_id.to_owned(),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateGraphArgs {
    #[serde(default)]
    pub entities: Vec<GraphEntityInput>,
    #[serde(default)]
    pub relationships: Vec<GraphRelationshipInput>,
    /// Exact saved relationship IDs. Retract only connections the writer explicitly takes back.
    #[serde(default)]
    pub retracted_relationship_ids: Vec<String>,
}

impl UpdateGraphArgs {
    fn validate_ids(&self) -> Result<(), ServiceError> {
        for entity in &self.entities {
            if let Some(id) = &entity.id {
                validate_graph_id(id)?;
            }
        }
        for relationship in &self.relationships {
            if let Some(id) = &relationship.id {
                validate_graph_id(id)?;
            }
            validate_graph_id(&relationship.from_id)?;
            validate_graph_id(&relationship.to_id)?;
        }
        for id in &self.retracted_relationship_ids {
            validate_graph_id(id)?;
        }
        Ok(())
    }
}

fn validate_graph_id(id: &str) -> Result<(), ServiceError> {
    Uuid::parse_str(id).map_err(|_| {
        ServiceError::InternalError(
            "Use saved graph IDs returned by search or update_graph.".into(),
        )
    })?;
    Ok(())
}

#[derive(Serialize)]
struct SavedEntity {
    id: String,
    name: String,
}

impl From<&Entity> for SavedEntity {
    fn from(entity: &Entity) -> Self {
        Self {
            id: entity.id.clone(),
            name: entity.name.clone(),
        }
    }
}

#[derive(Serialize)]
struct SavedRelationship {
    id: String,
    from_id: String,
    to_id: String,
    rel_type: String,
}

impl From<&Relationship> for SavedRelationship {
    fn from(relationship: &Relationship) -> Self {
        Self {
            id: relationship.id.clone(),
            from_id: relationship.from_id.clone(),
            to_id: relationship.to_id.clone(),
            rel_type: relationship.rel_type.clone(),
        }
    }
}

#[derive(Serialize)]
struct GraphUpdateResult {
    saved_entities: Vec<SavedEntity>,
    saved_relationships: Vec<SavedRelationship>,
    retracted_relationship_ids: Vec<String>,
}

impl GraphUpdateResult {
    fn from_saved(
        entities: &[(Entity, Vec<f64>)],
        relationships: &[Relationship],
        retracted_relationship_ids: Vec<String>,
    ) -> Self {
        Self {
            saved_entities: entities
                .iter()
                .map(|(entity, _)| SavedEntity::from(entity))
                .collect(),
            saved_relationships: relationships.iter().map(SavedRelationship::from).collect(),
            retracted_relationship_ids,
        }
    }
}

impl PortableTool for UpdateGraph {
    const NAME: &'static str = "update_graph";
    type Args = UpdateGraphArgs;
    type Output = String;
    type Error = ServiceError;

    fn description(&self) -> String {
        "Save entities, directed relationships, and explicit relationship retractions in one \
         transaction in this world. Search and read complete saved lore first. Use exact saved \
         IDs for existing records. Omit id or use null for new records; the server generates IDs. \
         Save new entities first, then use their returned IDs in a later relationship call. \
         Relationships use from_id and to_id of saved entities. Never \
         resolve targets by name or alias. Entity and relationship fields are complete replacements: \
         preserve earlier facts, aliases, properties, and conditions. Save only writer-stated or \
         confirmed lore. Do not overwrite canon with unresolved contradictions or speculation. \
         Retract only relationships the writer explicitly takes back. A failed batch saves no changes."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(UpdateGraphArgs)).unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        args.validate_ids()?;

        let mut entities = Vec::new();
        for input in args.entities {
            let entity = Entity::from((input, self.world_id.as_str()));
            let text = format!(
                "{}. {}. {}",
                entity.name,
                entity.kind.join(", "),
                entity.description
            );
            let embedding = self.embedding_model.embed_text(&text).await?.vec;
            entities.push((entity, embedding));
        }
        let mut relationships = Vec::new();
        for input in args.relationships {
            relationships.push(Relationship::from((
                input,
                self.world_id.as_str(),
                self.conversation_id.as_str(),
            )));
        }
        self.graph_repo
            .update_graph_batch(
                &self.world_id,
                &entities,
                &relationships,
                &args.retracted_relationship_ids,
                &self.source_message_id,
            )
            .await?;
        let result = GraphUpdateResult::from_saved(
            &entities,
            &relationships,
            args.retracted_relationship_ids,
        );
        Ok(serde_json::json!(result).to_string())
    }
}
