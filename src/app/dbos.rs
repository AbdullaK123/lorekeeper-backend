use std::sync::Arc;

use dbos::{Config, DBOS};
use tracing::Instrument;

use crate::{
    infrastructure::{InfrastructureError, Settings},
    service::{ResearchInput, ResearcherService, ServiceError},
};

pub async fn setup_dbos(
    settings: &Settings,
    researcher_service: ResearcherService,
) -> Result<(DBOS, dbos::WorkflowRef<ResearchInput, String, ServiceError>), InfrastructureError> {
    tracing::info!("initializing DBOS research workflow");
    let mut config = Config::new("lorekeeper", &settings.database_url);
    config.app_version = Some("dev-v1".into());
    let dbos = DBOS::new(config);

    let researcher_service = Arc::new(researcher_service);
    let research = dbos
        .register_workflow("research", move |input: ResearchInput| {
            let service = researcher_service.clone();
            let span = tracing::info_span!("research_workflow",
                workflow_id = %input.source_message_id,
                conversation_id = %input.conversation_id,
                world_id = %input.world_id);

            async move {
                let started = std::time::Instant::now();
                tracing::info!("research workflow started");
                let result = service
                    .process_message(
                        input.world_id,
                        input.conversation_id,
                        input.source_message_id,
                        &input.message,
                        &input.history,
                    )
                    .await;
                match &result {
                    Ok(_) => {
                        tracing::info!(
                            elapsed_ms = started.elapsed().as_millis() as u64,
                            "research workflow completed"
                        )
                    }
                    Err(_) => tracing::error!(
                        elapsed_ms = started.elapsed().as_millis() as u64,
                        "research workflow failed"
                    ),
                }
                result
            }
            .instrument(span)
        })
        .map_err(|e| InfrastructureError::InternalError(e.to_string()))?;

    dbos.launch()
        .await
        .map_err(|e| InfrastructureError::InternalError(e.to_string()))?;
    tracing::info!("DBOS research workflow ready");

    Ok((dbos, research))
}
