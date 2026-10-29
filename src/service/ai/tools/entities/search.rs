use crate::service::{RetrievalService, ServiceError, format_hits};
use rig::tool::PortableTool;
use schemars::JsonSchema;
use serde::Deserialize;

pub struct SearchWorld {
    pub retrieval_service: RetrievalService,
    pub world_id: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchWorldArgs {
    /// What to look for, in plain words, e.g. "who runs the tavern on Meridian Station"
    /// or "the Iron Conclave's allies". Names, places, events, or a question all work.
    pub query: String,
}

impl PortableTool for SearchWorld {
    const NAME: &'static str = "search_world";
    type Args = SearchWorldArgs;
    type Output = String;
    type Error = ServiceError;

    fn description(&self) -> String {
        "Search this world's saved lore by meaning and by name. Returns matching \
         entities and their connections, including saved entity IDs, aliases, kinds, \
         and descriptions. Use the returned IDs to identify exact saved entities. \
         Several matches may refer to different subjects; do not select a subject \
         from an alias alone. Results contain only currently saved lore."
            .to_string()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(SearchWorldArgs)).unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let hits = self
            .retrieval_service
            .search(&args.query, &self.world_id, 60.0)
            .await?;
        Ok(format_hits(&hits))
    }
}
