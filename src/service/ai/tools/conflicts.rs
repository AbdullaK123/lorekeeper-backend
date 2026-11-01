use crate::{
    data::{Conflict, GraphRepository},
    service::ServiceError,
};
use rig::tool::PortableTool;
use schemars::JsonSchema;
use serde::Deserialize;
use uuid::Uuid;

pub struct FlagConflict {
    pub world_id: String,
    pub conversation_id: String,
    pub source_message_id: String,
    pub graph_repo: GraphRepository,
}

#[derive(Deserialize, JsonSchema)]
pub struct FlagConflictArgs {
    pub entity_id: String,
    /// Exact, case-sensitive key from the saved entity's properties map.
    pub field: String,
    /// Disputed replacement value, preserving its JSON type.
    pub new_value: serde_json::Value,
    pub severity: String,
    pub explanation: String,
}

impl PortableTool for FlagConflict {
    const NAME: &'static str = "flag_conflict";
    type Args = FlagConflictArgs;
    type Output = String;
    type Error = ServiceError;

    fn description(&self) -> String {
        "Record a contradiction against an exact saved entity ID without replacing canon. \
         Read the entity first. field must be an exact, case-sensitive existing key in its properties map. \
         Supply the disputed new_value with its JSON type and an explanation; the server reads the existing value. \
         Do not use a paraphrase, property path, core entity field, or relationship as the target. \
         If no saved property matches, report the unsupported claim without changing canon. Use hard \
         for incompatible claims or soft for possible tension. Explicit corrections are deliberate \
         changes, not unresolved contradictions."
            .into()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(FlagConflictArgs)).unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let entity = self
            .graph_repo
            .get_entity(&args.entity_id, &self.world_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Entity not found in this world.".into()))?;
        let existing_value = entity.properties.get(&args.field).cloned().ok_or_else(|| {
            ServiceError::InvalidInput(
                "field must be an exact, existing entity property key. Read the entity and use its saved key; do not invent a property.".into(),
            )
        })?;
        let conflict = Conflict {
            id: Uuid::new_v4().to_string(),
            world_id: self.world_id.clone(),
            source_conversation: self.conversation_id.clone(),
            entity_id: args.entity_id,
            field: args.field,
            existing_value,
            new_value: args.new_value,
            severity: args.severity,
            explanation: args.explanation,
            resolved: false,
            decision: None,
            source_message_ids: Vec::new(),
        };
        let saved = self
            .graph_repo
            .upsert_conflict(&conflict, &self.source_message_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Entity not found in this world.".into()))?;
        Ok(format!(
            "Recorded conflict [ID: {}]. Canon remains unchanged.",
            saved.id
        ))
    }
}

pub struct GetConflicts {
    pub graph_repo: GraphRepository,
    pub world_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct GetConflictsArgs {}

impl PortableTool for GetConflicts {
    const NAME: &'static str = "get_conflicts";
    type Args = GetConflictsArgs;
    type Output = Vec<Conflict>;
    type Error = ServiceError;

    fn description(&self) -> String {
        "List all saved unresolved conflicts in this world. Call this when the writer asks \
         about contradictions, disputed claims, or unresolved inconsistencies. World search \
         results do not include conflict records; do not infer that no conflicts exist from \
         those results. Each conflict includes the subject entity ID, affected field, both \
         claims, severity, explanation, and available source message IDs. Present both claims \
         as unresolved rather than choosing one or inventing a reconciliation. An empty list \
         means no unresolved conflicts are recorded, not that the world has no possible \
         contradictions. This tool only reads records and does not change canon or resolve conflicts."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(GetConflictsArgs)).unwrap()
    }

    async fn call(&self, _arguments: Self::Args) -> Result<Self::Output, Self::Error> {
        let conflicts = self.graph_repo.get_conflicts(&self.world_id).await?;
        Ok(conflicts)
    }
}
