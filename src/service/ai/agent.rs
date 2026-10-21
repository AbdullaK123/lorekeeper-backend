use rig::{Agent, AgentBuilder, DynModel, operation::Completion, providers::openrouter};

use crate::{
    infrastructure::load_prompt,
    service::{RetrievalService, SearchWorld},
};

#[derive(Clone)]
pub struct ChatAgentFactory {
    model: DynModel<Completion>,
    preamble: String,
    retrieval_service: RetrievalService,
}

impl ChatAgentFactory {
    pub fn new(
        model: DynModel<Completion>,
        preamble: String,
        retrieval_service: RetrievalService,
    ) -> Self {
        Self {
            model,
            preamble,
            retrieval_service,
        }
    }

    pub fn create_agent(&self, world_id: String, lore: String) -> Agent {
        AgentBuilder::new(self.model.clone())
            .preamble(&self.preamble)
            .context(lore)
            .tool(SearchWorld {
                world_id,
                retrieval_service: self.retrieval_service.clone(),
            })
            .build()
    }
}
