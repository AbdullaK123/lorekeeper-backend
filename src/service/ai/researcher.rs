use std::collections::BTreeSet;

use crate::{
    data::{GraphRepository, Message},
    service::{FlagConflict, ReadEntity, RetrievalService, SearchWorld, ServiceError, UpdateGraph},
};
use rig::{
    Agent, AgentBuilder, AgentRun, DynModel,
    completion::CompletionRequest,
    message::UserContent,
    operation::{Completion, Embedding},
    run::{AgentRunStep, InvalidToolCallAction, ModelTurn, ModelTurnOutcome},
    tool::ToolContext,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct ResearchInput {
    pub world_id: Uuid,
    pub conversation_id: Uuid,
    pub source_message_id: Uuid,
    pub message: String,
    pub history: Vec<Message>,
}

/// Writes lore after the conversation turn is saved. Chat agents remain read-only.
#[derive(Clone)]
pub struct ResearcherService {
    model: DynModel<Completion>,
    preamble: String,
    graph_repo: GraphRepository,
    retrieval_service: RetrievalService,
    embedding_model: DynModel<Embedding>,
}

impl ResearcherService {
    pub fn new(
        model: DynModel<Completion>,
        preamble: String,
        graph_repo: GraphRepository,
        retrieval_service: RetrievalService,
        embedding_model: DynModel<Embedding>,
    ) -> Self {
        Self {
            model,
            preamble,
            graph_repo,
            retrieval_service,
            embedding_model,
        }
    }

    pub(crate) fn create_agent(
        &self,
        world_id: String,
        conversation_id: String,
        source_message_id: String,
    ) -> Agent {
        AgentBuilder::new(self.model.clone())
            .preamble(&self.preamble)
            .tool(SearchWorld {
                world_id: world_id.clone(),
                retrieval_service: self.retrieval_service.clone(),
            })
            .tool(ReadEntity {
                world_id: world_id.clone(),
                graph_repo: self.graph_repo.clone(),
            })
            .tool(FlagConflict {
                world_id: world_id.clone(),
                conversation_id: conversation_id.clone(),
                source_message_id: source_message_id.clone(),
                graph_repo: self.graph_repo.clone(),
            })
            .tool(UpdateGraph {
                world_id,
                conversation_id,
                source_message_id,
                graph_repo: self.graph_repo.clone(),
                embedding_model: self.embedding_model.clone(),
            })
            .build()
    }

    pub async fn process_message(
        &self,
        world_id: Uuid,
        conversation_id: Uuid,
        source_message_id: Uuid,
        message: &str,
        history: &[Message],
    ) -> dbos::Result<String, ServiceError> {
        let world_id = world_id.to_string();
        let conversation_id = conversation_id.to_string();
        let source_message_id = source_message_id.to_string();
        let input = serde_json::json!({
            "world_id": world_id, "message": message, "recent_messages": history,
            "source_conversation": conversation_id,
        });
        let agent = self.create_agent(world_id, conversation_id, source_message_id);
        run_durable_agent(&agent, &self.model, &self.preamble, input.to_string()).await
    }
}

/// Replay completed model/tool operations through DBOS, then advance Rig's state machine.
/// Tool calls run sequentially so their checkpoint order is stable during recovery.
pub(crate) async fn run_durable_agent(
    agent: &Agent,
    model: &DynModel<Completion>,
    preamble: &str,
    input: String,
) -> dbos::Result<String, ServiceError> {
    let definitions = agent.tool_server_handle().static_tool_defs();
    let names: BTreeSet<String> = definitions
        .iter()
        .map(|tool| tool.name.to_string())
        .collect();
    let mut run = AgentRun::new(input).max_turns(12);
    loop {
        match run.next_step().map_err(ServiceError::from)? {
            AgentRunStep::CallModel {
                prompt, history, ..
            } => {
                let response = dbos::step("research_model", || async {
                    let request = CompletionRequest::new(prompt.clone())
                        .messages(history.clone())
                        .preamble(preamble)
                        .tools(definitions.clone());
                    let response = model.call(request).await.map_err(ServiceError::from)?;
                    Ok::<_, dbos::Error<ServiceError>>(response)
                })
                .await?;
                let turn = ModelTurn::from_response_parts(&response, names.clone(), names.clone());
                let outcome = run.model_response(turn).map_err(ServiceError::from)?;
                if let ModelTurnOutcome::NeedsResolution(_) = outcome {
                    run.resolve_invalid_tool_call(InvalidToolCallAction::fail())
                        .map_err(ServiceError::from)?;
                }
            }
            AgentRunStep::CallTools { calls } => {
                let mut results = Vec::with_capacity(calls.len());
                for pending in calls {
                    if let Some(result) = pending.preresolved_result {
                        results.push(result);
                        continue;
                    }
                    let call = pending.tool_call;
                    let result = dbos::step("research_tool", || async {
                        let mut context = ToolContext::new();
                        let result = agent
                            .tool_server_handle()
                            .execute(
                                call.function.name.as_ref(),
                                &call.function.arguments_value().to_string(),
                                &mut context,
                            )
                            .await;
                        Ok::<_, dbos::Error<ServiceError>>(UserContent::tool_result(
                            call.id.clone(),
                            call.function.name.clone(),
                            result.output().clone().into_content(),
                        ))
                    })
                    .await?;
                    results.push(result);
                }
                run.tool_results(results).map_err(ServiceError::from)?;
            }
            AgentRunStep::Done(response) => return Ok(response.output().to_string()),
        }
    }
}
