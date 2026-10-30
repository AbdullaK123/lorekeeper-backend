use crate::{
    data::GraphRepository,
    service::{GetConflicts, RetrievalService, SearchWorld},
};
use rig::{Agent, AgentBuilder, DynModel, operation::Completion};

#[derive(Clone)]
pub struct ChatAgentFactory {
    model: DynModel<Completion>,
    preamble: String,
    retrieval_service: RetrievalService,
    graph_repo: GraphRepository,
}

impl ChatAgentFactory {
    pub fn new(
        model: DynModel<Completion>,
        preamble: String,
        retrieval_service: RetrievalService,
        graph_repo: GraphRepository,
    ) -> Self {
        Self {
            model,
            preamble,
            retrieval_service,
            graph_repo,
        }
    }

    pub fn create_agent(&self, world_id: &str, lore: String) -> Agent {
        AgentBuilder::new(self.model.clone())
            .preamble(&self.preamble)
            .context(format!("<lore>\n{lore}\n</lore>"))
            .tool(SearchWorld {
                world_id: world_id.to_string(),
                retrieval_service: self.retrieval_service.clone(),
            })
            .tool(GetConflicts {
                world_id: world_id.to_string(),
                graph_repo: self.graph_repo.clone(),
            })
            .build()
    }
}
