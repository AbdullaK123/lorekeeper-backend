use crate::{data, service};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires QA Postgres and Memgraph; no model calls"]
async fn missing_research_uses_only_earlier_messages() {
    let (pool, _, user, world, conversation) = super::world_deletion_fixture().await;
    let repo = data::ConversationRepository::new(pool.clone());
    let source = Uuid::new_v4();
    let earlier = Uuid::new_v4();
    let messages = vec![
        data::Message::User {
            id: earlier,
            content: "earlier lore".into(),
            created_at: chrono::Utc::now(),
        },
        data::Message::User {
            id: source,
            content: "source lore".into(),
            created_at: chrono::Utc::now(),
        },
        data::Message::User {
            id: Uuid::new_v4(),
            content: "later lore".into(),
            created_at: chrono::Utc::now(),
        },
    ];
    repo.append_messages(conversation, user, &messages)
        .await
        .unwrap();
    let database = std::env::var("DATABASE_URL").unwrap();
    let name = format!("retry-test-{}", Uuid::new_v4());
    let mut config = dbos::Config::new(name.clone(), &database);
    config.app_version = Some(name.clone());
    config.executor_id = Some(name);
    let runtime = dbos::DBOS::new(config);
    let captured = Arc::new(Mutex::new(Vec::<service::ResearchInput>::new()));
    let seen = captured.clone();
    let job = runtime
        .register_workflow("research", move |input: service::ResearchInput| {
            let seen = seen.clone();
            async move {
                seen.lock().unwrap().push(input);
                Ok::<_, dbos::Error<service::ServiceError>>("done".to_string())
            }
        })
        .unwrap();
    runtime.launch().await.unwrap();
    let client = dbos::Client::connect(dbos::ClientConfig::new(&database))
        .await
        .unwrap();
    let (graph, _) = super::graph_fixture().await;
    let provider = rig::providers::openrouter::new("unused-test-key");
    let retrieval =
        service::RetrievalService::new(graph.clone(), provider.embedding("unused", None).into());
    let factory = service::ChatAgentFactory::new(
        provider.chat("unused").into(),
        "test".into(),
        retrieval.clone(),
        graph,
    );
    let svc = service::ConversationService::new(
        repo.clone(),
        data::WorldRepository::new(pool.clone()),
        retrieval,
        factory,
        job,
        client,
    );
    svc.retry_research_run(world, user, source).await.unwrap();
    for _ in 0..100 {
        if matches!(
            svc.get_researcher_status(world, user, source)
                .await
                .unwrap()
                .status,
            data::ResearchStatus::Completed
        ) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(
        matches!(
            svc.get_researcher_status(world, user, source)
                .await
                .unwrap()
                .status,
            data::ResearchStatus::Completed
        ),
        "Submitted research never reaches completion"
    );
    {
        let inputs = captured.lock().unwrap();
        assert_eq!(inputs.len(), 1, "Submission executes more than once");
        assert_eq!(
            inputs[0].message, "source lore",
            "Retry researches another message"
        );
        assert_eq!(
            inputs[0].history.len(),
            1,
            "Retry includes its source or later claims as earlier history"
        );
        assert_eq!(
            inputs[0].history[0].id(),
            earlier,
            "Retry drops earlier context"
        );
    }
    assert!(
        matches!(
            svc.retry_research_run(world, user, source).await,
            Err(service::ServiceError::Conflict(_))
        ),
        "Completed research executes again"
    );
    assert_eq!(
        repo.get_research_workflow(source).await.unwrap(),
        Some(source.to_string()),
        "Submission loses workflow tracking"
    );
    runtime.shutdown().await;
    sqlx::query("DELETE FROM message_research_workflows WHERE message_id=$1")
        .bind(source)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM dbos.workflow_status WHERE workflow_uuid=$1")
        .bind(source.to_string())
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM worlds WHERE id=$1")
        .bind(world)
        .execute(&pool)
        .await
        .unwrap();
}
