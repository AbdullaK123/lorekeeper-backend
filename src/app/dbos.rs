use std::sync::Arc;

use dbos::{Config, DBOS};

use crate::{
    infrastructure::{InfrastructureError, Settings},
    service::{ResearchInput, ResearcherService, ServiceError},
};

pub async fn setup_dbos(
    settings: &Settings,
    researcher_service: ResearcherService,
) -> Result<(DBOS, dbos::WorkflowRef<ResearchInput, String, ServiceError>), InfrastructureError> {
    let mut config = Config::new("lorekeeper", &settings.database_url);
    config.app_version = Some("dev-v1".into());
    let dbos = DBOS::new(config);

    let researcher_service = Arc::new(researcher_service);
    let research = dbos
        .register_workflow("research", move |input: ResearchInput| {
            let service = researcher_service.clone();

            async move {
                let conversation_id = input.conversation_id;
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
                        tracing::info!("research completed for conversation {conversation_id}")
                    }
                    Err(error) => tracing::error!(
                        "research failed for conversation {conversation_id}: {error}"
                    ),
                }
                result
            }
        })
        .map_err(|e| InfrastructureError::InternalError(e.to_string()))?;

    dbos.launch()
        .await
        .map_err(|e| InfrastructureError::InternalError(e.to_string()))?;

    Ok((dbos, research))
}
