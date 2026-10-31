use crate::shared::{Description, EntityName};
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
    pub name: EntityName,
    pub aliases: Vec<EntityName>,
    pub kind: Vec<String>,
    /// Complete updated description. Preserve earlier facts when adding details.
    pub description: Description,
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
    pub description: Description,
    pub properties: HashMap<String, serde_json::Value>,
}

impl From<(GraphEntityInput, &str)> for Entity {
    fn from((input, world_id): (GraphEntityInput, &str)) -> Self {
        Self {
            id: input.id.unwrap_or_else(|| Uuid::new_v4().to_string()),
            world_id: world_id.to_owned(),
            name_key: name_key(input.name.as_str()),
            name: input.name.into_inner(),
            aliases: input.aliases.into_iter().map(String::from).collect(),
            kind: input.kind,
            description: input.description.into_inner(),
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
            description: input.description.into_inner(),
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
struct SavedEntity<'a> {
    id: &'a str,
    name: &'a str,
}

impl<'a> From<&'a Entity> for SavedEntity<'a> {
    fn from(entity: &'a Entity) -> Self {
        Self {
            id: &entity.id,
            name: &entity.name,
        }
    }
}

#[derive(Serialize)]
struct SavedRelationship<'a> {
    id: &'a str,
    from_id: &'a str,
    to_id: &'a str,
    rel_type: &'a str,
}

impl<'a> From<&'a Relationship> for SavedRelationship<'a> {
    fn from(relationship: &'a Relationship) -> Self {
        Self {
            id: &relationship.id,
            from_id: &relationship.from_id,
            to_id: &relationship.to_id,
            rel_type: &relationship.rel_type,
        }
    }
}

#[derive(Serialize)]
struct GraphUpdateResult<'a> {
    saved_entities: Vec<SavedEntity<'a>>,
    saved_relationships: Vec<SavedRelationship<'a>>,
    retracted_relationship_ids: Vec<String>,
}

impl<'a> GraphUpdateResult<'a> {
    fn from_saved(
        entities: &'a [(Entity, Vec<f64>)],
        relationships: &'a [Relationship],
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
