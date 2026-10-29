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
    pub field: String,
    pub existing_value: serde_json::Value,
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
         Read the entity first. Include both claims, affected field, and explanation. Use hard \
         for incompatible claims or soft for possible tension. Explicit corrections are deliberate \
         changes, not unresolved contradictions."
            .into()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(FlagConflictArgs)).unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let conflict = Conflict {
            id: Uuid::new_v4().to_string(),
            world_id: self.world_id.clone(),
            source_conversation: self.conversation_id.clone(),
            entity_id: args.entity_id,
            field: args.field,
            existing_value: args.existing_value,
            new_value: args.new_value,
            severity: args.severity,
            explanation: args.explanation,
            resolved: false,
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
