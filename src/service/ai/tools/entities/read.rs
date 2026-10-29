use crate::{data::GraphRepository, service::ServiceError};
use rig::tool::PortableTool;
use schemars::JsonSchema;
use serde::Deserialize;

pub struct ReadEntity {
    pub world_id: String,
    pub graph_repo: GraphRepository,
}

#[derive(Deserialize, JsonSchema)]
pub struct ReadEntityArgs {
    pub entity_id: String,
}

impl PortableTool for ReadEntity {
    const NAME: &'static str = "read_entity";
    type Args = ReadEntityArgs;
    type Output = serde_json::Value;
    type Error = ServiceError;

    fn description(&self) -> String {
        "Read a complete saved entity by exact ID in this world, including properties, \
         connected entities, complete relationships, and conflicts. Read before replacing \
         records through update_graph to preserve earlier facts and conditions."
            .into()
    }

    fn parameters(&self) -> serde_json::Value {
        serde_json::to_value(schemars::schema_for!(ReadEntityArgs)).unwrap()
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let entity = self
            .graph_repo
            .get_entity(&args.entity_id, &self.world_id)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Entity not found in this world.".into()))?;
        let neighbors = self
            .graph_repo
            .get_entity_neighbors(&entity.id, &self.world_id, None, None)
            .await?;
        let conflicts = self
            .graph_repo
            .get_conflicts_by_entity(&entity.id, &self.world_id)
            .await?;
        Ok(serde_json::json!({"entity": entity, "neighbors": neighbors, "conflicts": conflicts}))
    }
}
