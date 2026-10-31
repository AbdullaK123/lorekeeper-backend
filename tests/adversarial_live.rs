//! Opt-in live tests. See tests/README.md for isolated setup.
#[path = "../src/app/mod.rs"]
mod app;
#[path = "../src/cron_worker.rs"]
mod cron_worker;
#[path = "../src/data/mod.rs"]
mod data;
#[path = "../src/infrastructure/mod.rs"]
mod infrastructure;
#[path = "../src/service/mod.rs"]
mod service;
#[path = "../src/shared/mod.rs"]
mod shared;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires an isolated Postgres database, Memgraph, and an OpenRouter key"]
async fn adversarial_live_api() {
    let database = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    assert!(database.ends_with("/lorekeeper_adversarial"), "Fault injection must never target development data");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:18000").await.unwrap();
    let (router, mut scheduler, dbos) = app::create_app().await;
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("python3")
            .args(["tests/adversarial_api.py", "-v"])
            .args(std::env::var("QA_TESTS").unwrap_or_default().split_whitespace())
            .env("QA_BASE_URL", "http://127.0.0.1:18000")
            .status()
            .expect("python3 with requests must be installed")
    }).await.unwrap();
    let scheduler_result = scheduler.shutdown().await;
    dbos.shutdown().await;
    server.abort();
    scheduler_result.unwrap();
    assert!(result.success(), "Live failures identify behavior that can harm users; inspect Python assertions");
}

const TEST_SOURCE_MESSAGE_ID: &str = "00000000-0000-4000-8000-000000000001";

async fn graph_fixture() -> (data::GraphRepository, String) {
    let settings = infrastructure::load_settings();
    let config = infrastructure::load_config();
    let graph = infrastructure::create_graph(&settings, &config.memgraph).await;
    (data::GraphRepository::new(graph), format!("qa-graph-{}", uuid::Uuid::new_v4()))
}

async fn world_deletion_fixture() -> (sqlx::PgPool, data::WorldRepository, uuid::Uuid, uuid::Uuid, uuid::Uuid) {
    let database = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    assert!(database.ends_with("/lorekeeper_adversarial"), "Deletion tests must use the isolated QA database");
    let pool = sqlx::PgPool::connect(&database).await.unwrap();
    let user = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, email) VALUES ($1, 'QA', $2)")
        .bind(user).bind(format!("qa-{user}@example.com")).execute(&pool).await.unwrap();
    let repo = data::WorldRepository::new(pool.clone());
    let world = repo.create("QA deletion".into(), "".into(), user).await.unwrap();
    let conversation = data::ConversationRepository::new(pool.clone())
        .create(world.id, user, "QA".into()).await.unwrap();
    (pool, repo, user, world.id, conversation.id)
}

#[tokio::test]
#[ignore = "requires the isolated Postgres test database"]
async fn world_deletion_commits_its_cleanup_job() {
    let (pool, repo, user, world, conversation) = world_deletion_fixture().await;
    assert!(repo.delete(world, user).await.unwrap(), "Owned world deletion incorrectly reports failure");
    let job: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM world_cleanup_jobs WHERE world_id = $1)")
        .bind(world).fetch_one(&pool).await.unwrap();
    assert!(job, "Deleting a world without saving its cleanup job permanently abandons graph data");
    assert!(repo.get_by_id(world, user).await.unwrap().is_none(), "Successful deletion leaves the world accessible");
    assert!(data::ConversationRepository::new(pool).get_by_id(conversation, user).await.unwrap().is_none(),
        "World deletion leaves its conversation accessible");
}

#[tokio::test]
#[ignore = "requires the isolated Postgres test database"]
async fn world_deletion_cannot_queue_cleanup_for_another_users_world() {
    let (pool, repo, owner, world, _) = world_deletion_fixture().await;
    assert!(!repo.delete(world, uuid::Uuid::new_v4()).await.unwrap(), "Another user deletes the owner's world");
    let job: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM world_cleanup_jobs WHERE world_id = $1)")
        .bind(world).fetch_one(&pool).await.unwrap();
    assert!(!job, "An unauthorized deletion queues destruction of another user's graph");
    assert!(repo.get_by_id(world, owner).await.unwrap().is_some(), "Unauthorized deletion removes the owner's world");
}

#[tokio::test]
#[ignore = "requires the isolated Postgres test database"]
async fn world_deletion_rolls_back_when_cleanup_job_insertion_fails() {
    let (pool, repo, user, world, conversation) = world_deletion_fixture().await;
    // Force the real job INSERT to fail through its unique constraint.
    sqlx::query("INSERT INTO world_cleanup_jobs (world_id) VALUES ($1)")
        .bind(world).execute(&pool).await.unwrap();
    assert!(repo.delete(world, user).await.is_err(), "A rejected cleanup job is reported as successful deletion");
    assert!(repo.get_by_id(world, user).await.unwrap().is_some(),
        "A failed cleanup-job insertion still commits world deletion");
    assert!(data::ConversationRepository::new(pool).get_by_id(conversation, user).await.unwrap().is_some(),
        "Rolling back world deletion fails to restore its cascaded conversations");
}

#[tokio::test]
#[ignore = "requires isolated Postgres and live Memgraph"]
async fn cleanup_retries_failed_job_removal_without_blocking_other_jobs() {
    let (pool, worlds, owner, first, _) = world_deletion_fixture().await;
    let second = worlds.create("QA second cleanup".into(), "".into(), owner).await.unwrap().id;
    let (graph, _) = graph_fixture().await;
    save_graph_entity(&graph, &graph_entity(&first.to_string(), "First")).await;
    save_graph_entity(&graph, &graph_entity(&second.to_string(), "Second")).await;
    worlds.delete(first, owner).await.unwrap();
    worlds.delete(second, owner).await.unwrap();

    let trigger = format!("qa_ack_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(&format!(
        "CREATE FUNCTION {trigger}() RETURNS trigger LANGUAGE plpgsql AS $$
         BEGIN IF OLD.world_id = '{first}'::uuid THEN RAISE EXCEPTION 'QA job removal rejected';
         END IF; RETURN OLD; END $$;
         CREATE TRIGGER {trigger} BEFORE DELETE ON world_cleanup_jobs
         FOR EACH ROW EXECUTE FUNCTION {trigger}();"
    )).execute(&pool).await.unwrap();

    let service = service::CleanupService::new(pool.clone(), graph.clone());
    let pass = service.run_once().await;
    let first_nodes = graph.get_entities_by_world(&first.to_string()).await.unwrap();
    let second_nodes = graph.get_entities_by_world(&second.to_string()).await.unwrap();
    let first_jobs: i64 = sqlx::query_scalar("SELECT count(*) FROM world_cleanup_jobs WHERE world_id = $1")
        .bind(first).fetch_one(&pool).await.unwrap();
    let second_jobs: i64 = sqlx::query_scalar("SELECT count(*) FROM world_cleanup_jobs WHERE world_id = $1")
        .bind(second).fetch_one(&pool).await.unwrap();

    sqlx::raw_sql(&format!("DROP TRIGGER {trigger} ON world_cleanup_jobs; DROP FUNCTION {trigger}();"))
        .execute(&pool).await.unwrap();
    let retry = service.run_once().await;
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM world_cleanup_jobs WHERE world_id = $1")
        .bind(first).fetch_one(&pool).await.unwrap();
    graph.delete_graph(&first.to_string()).await.unwrap();
    graph.delete_graph(&second.to_string()).await.unwrap();

    assert!(pass.is_ok(), "One rejected job removal aborts the whole cleanup pass");
    assert!(first_nodes.is_empty(), "Graph deletion fails before job removal is attempted");
    assert_eq!(first_jobs, 1, "Failed job removal loses the pending retry");
    assert!(second_nodes.is_empty(), "One failed job prevents another world's graph cleanup");
    assert_eq!(second_jobs, 0, "One failed job prevents another successful job from completing");
    assert!(retry.is_ok(), "Retrying cleanup against an already empty graph fails");
    assert_eq!(remaining, 0, "Retry never removes the job after the database recovers");
}

fn graph_entity(world: &str, name: &str) -> data::Entity {
    data::Entity {
        id: uuid::Uuid::new_v4().to_string(), world_id: world.into(),
        name: name.into(), name_key: data::name_key(name), aliases: vec![],
        kind: vec!["character".into()], description: "QA fixture".into(),
        properties: std::collections::HashMap::new(),
        source_message_ids: vec![],
    }
}

fn embedding() -> Vec<f64> {
    let mut value = vec![0.0; 1536];
    value[0] = 1.0;
    value
}

async fn save_graph_entity(repo: &data::GraphRepository, entity: &data::Entity) -> data::Entity {
    repo.update_graph_batch(&entity.world_id, &[(entity.clone(), embedding())], &[], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    repo.get_entity(&entity.id, &entity.world_id).await.unwrap().unwrap()
}

fn graph_relationship(world: &str, from: &str, to: &str, label: &str) -> data::Relationship {
    data::Relationship {
        id: uuid::Uuid::new_v4().to_string(), world_id: world.into(),
        from_id: from.into(), to_id: to.into(), rel_type: label.into(),
        description: "QA edge".into(), properties: std::collections::HashMap::new(),
        source_conversation: "qa".into(),
        source_message_ids: vec![],
    }
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_retry_preserves_entity_identity() {
    let (repo, world) = graph_fixture().await;
    let first = save_graph_entity(&repo, &graph_entity(&world, "Kael")).await;
    let mut updated = first.clone();
    updated.name = "  KAEL  ".into();
    updated.description = "An updated description".into();
    let second = save_graph_entity(&repo, &updated).await;
    assert_eq!(first.id, second.id, "Updating an exact entity ID must preserve its identity");
    assert_eq!(second.description, updated.description, "Retry must apply new content");
    assert_eq!(repo.get_entities_by_world(&world).await.unwrap().len(), 1,
        "A retry grows the graph instead of updating the existing entity");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_relationships_cannot_cross_worlds() {
    let (repo, world) = graph_fixture().await;
    let other = format!("qa-graph-{}", uuid::Uuid::new_v4());
    let a = save_graph_entity(&repo, &graph_entity(&world, "A")).await;
    let b = save_graph_entity(&repo, &graph_entity(&other, "B")).await;
    let result = repo.update_graph_batch(&world, &[], &[graph_relationship(&world, &a.id, &b.id, "KNOWS")], &[], TEST_SOURCE_MESSAGE_ID).await;
    assert!(result.is_err(), "An edge connects private lore across user worlds");
    assert!(repo.get_entity(&a.id, &other).await.unwrap().is_none(),
        "A guessed entity ID bypasses world isolation");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_relationship_retry_does_not_duplicate_edges() {
    let (repo, world) = graph_fixture().await;
    let a = save_graph_entity(&repo, &graph_entity(&world, "A")).await;
    let b = save_graph_entity(&repo, &graph_entity(&world, "B")).await;
    let relationship = graph_relationship(&world, &a.id, &b.id, "KNOWS");
    for _ in 0..2 {
        repo.update_graph_batch(&world, &[], &[relationship.clone()], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    }
    assert_eq!(repo.get_entity_neighbors(&a.id, &world, None, None).await.unwrap().len(), 1,
        "Repeated edges inflate retrieval context and ranking");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_relationship_injection_is_rejected() {
    let (repo, world) = graph_fixture().await;
    for label in ["", "1BAD", "KNOWS]->(n) DETACH DELETE n //", "A`B", "A B", "认识"] {
        let result = repo.update_graph_batch(&world, &[], &[graph_relationship(&world, "a", "b", label)], &[], TEST_SOURCE_MESSAGE_ID).await;
        assert!(matches!(result, Err(infrastructure::InfrastructureError::CypherInjection(_))),
            "Untrusted relationship labels can alter a graph query: {label}");
    }
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_delete_entity_removes_attached_conflicts() {
    let (repo, world) = graph_fixture().await;
    let entity = save_graph_entity(&repo, &graph_entity(&world, "A")).await;
    let conflict = data::Conflict {
        id: uuid::Uuid::new_v4().to_string(), world_id: world.clone(), entity_id: entity.id.clone(),
        field: "description".into(), existing_value: serde_json::json!("old"),
        new_value: serde_json::json!("new"), severity: "hard".into(), explanation: "QA".into(),
        source_conversation: "qa".into(), resolved: false, source_message_ids: vec![],
    };
    repo.upsert_conflict(&conflict, TEST_SOURCE_MESSAGE_ID).await.unwrap().expect("QA conflict must exist");
    assert!(repo.delete_entity(&entity.id, &world).await.unwrap(), "Existing entities cannot be deleted");
    assert!(repo.get_conflict(&conflict.id, &world).await.unwrap().is_none(),
        "Deleting an entity leaves orphan conflicts that users cannot resolve");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_entity_properties_round_trip_without_loss() {
    let (repo, world) = graph_fixture().await;
    let mut entity = graph_entity(&world, "O'Brien 🐉 ");
    entity.properties.insert("nested".into(), serde_json::json!({"a": [null, true, 3, "雪"]}));
    let written = save_graph_entity(&repo, &entity).await;
    let read = repo.get_entity(&written.id, &world).await.unwrap().unwrap();
    assert_eq!(read.properties, entity.properties, "Graph storage silently corrupts structured or Unicode lore");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires the isolated Postgres test database"]
async fn concurrent_message_appends_do_not_lose_writes() {
    let database = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    assert!(database.ends_with("/lorekeeper_adversarial"), "Concurrency fixtures must use the QA database");
    let pool = sqlx::PgPool::connect(&database).await.unwrap();
    let user_id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, email, password_hash) VALUES ($1, 'QA', $2, 'unused')")
        .bind(user_id).bind(format!("qa-{user_id}@example.com")).execute(&pool).await.unwrap();
    let world = data::WorldRepository::new(pool.clone()).create("QA concurrency".into(), "".into(), user_id).await.unwrap();
    let repo = data::ConversationRepository::new(pool.clone());
    let conversation = repo.create(world.id, user_id, "Concurrent appends".into()).await.unwrap();
    let mut tasks = Vec::new();
    for index in 0..40 {
        let repo = repo.clone();
        let id = conversation.id;
        tasks.push(tokio::spawn(async move {
            repo.append_messages(id, user_id, &[data::Message::User {
                id: uuid::Uuid::new_v4(),
                content: format!("message-{index}"), created_at: chrono::Utc::now(),
            }]).await
        }));
    }
    let mut errors = Vec::new();
    for task in tasks {
        if let Err(error) = task.await.unwrap() { errors.push(error.to_string()); }
    }
    let saved = repo.get_by_id(conversation.id, user_id).await.unwrap().unwrap();
    repo.delete(conversation.id, user_id).await.unwrap();
    data::WorldRepository::new(pool.clone()).delete(world.id, user_id).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1").bind(user_id).execute(&pool).await.unwrap();
    assert!(errors.is_empty(), "Concurrent appends fail and lose accepted messages: {errors:?}");
    assert_eq!(saved.messages.0.len(), 40, "Concurrent appends silently overwrite each other's messages");
    let unique: std::collections::HashSet<_> = saved.messages.0.iter().filter_map(|m| match m {
        data::Message::User { content, .. } => Some(content.as_str()), _ => None,
    }).collect();
    assert_eq!(unique.len(), 40, "Concurrent append results duplicate messages instead of preserving each write");
}

#[tokio::test]
#[ignore = "requires Memgraph"]
async fn graph_batch_missing_endpoint_rolls_back_entities() {
    let (repo, world) = graph_fixture().await;
    let entity = graph_entity(&world, "Batch rollback");
    let edge = graph_relationship(&world, &entity.id, &uuid::Uuid::new_v4().to_string(), "KNOWS");
    let result = repo.update_graph_batch(&world, &[(entity.clone(), embedding())], &[edge], &[], TEST_SOURCE_MESSAGE_ID).await;
    let saved = repo.get_entity(&entity.id, &world).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert!(result.is_err(), "Missing endpoints must fail the batch");
    assert!(saved.is_none(), "A failed relationship must not leave an accepted entity write behind");
}

#[tokio::test]
#[ignore = "requires Memgraph"]
async fn graph_batch_retry_preserves_distinct_connections() {
    let (repo, world) = graph_fixture().await;
    let a = graph_entity(&world, "Batch A");
    let b = graph_entity(&world, "Batch B");
    repo.update_graph_batch(&world, &[(a.clone(), embedding()), (b.clone(), embedding())], &[], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let first = graph_relationship(&world, &a.id, &b.id, "KNOWS");
    let mut second = first.clone();
    second.id = uuid::Uuid::new_v4().to_string();
    second.description = "A separate conditional connection".into();
    for _ in 0..2 {
        repo.update_graph_batch(&world, &[], &[first.clone(), second.clone()], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    }
    let neighbors = repo.get_entity_neighbors(&a.id, &world, None, None).await.unwrap();
    repo.update_graph_batch(&world, &[], &[], &[first.id.clone()], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let remaining = repo.get_entity_neighbors(&a.id, &world, None, None).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert_eq!(neighbors.len(), 2, "Retries must preserve two distinct connection IDs without duplicate edges");
    assert_eq!(remaining.len(), 1, "Retraction must delete only the selected relationship");
    assert_eq!(remaining[0].1.id, second.id, "Retraction deleted the wrong relationship");
}

#[tokio::test]
#[ignore = "requires Memgraph"]
async fn graph_batch_foreign_endpoint_rolls_back_changes() {
    let (repo, world) = graph_fixture().await;
    let foreign_world = format!("qa-foreign-{}", uuid::Uuid::new_v4());
    let foreign = graph_entity(&foreign_world, "Foreign endpoint");
    repo.update_graph_batch(&foreign_world, &[(foreign.clone(), embedding())], &[], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let local = graph_entity(&world, "Local endpoint");
    let edge = graph_relationship(&world, &local.id, &foreign.id, "KNOWS");
    let result = repo.update_graph_batch(&world, &[(local.clone(), embedding())], &[edge], &[], TEST_SOURCE_MESSAGE_ID).await;
    let saved = repo.get_entity(&local.id, &world).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    repo.delete_graph(&foreign_world).await.unwrap();
    assert!(result.is_err(), "An endpoint from another world must never create a relationship");
    assert!(saved.is_none(), "Foreign endpoint failure must roll back earlier writes");
}

#[tokio::test]
#[ignore = "requires Memgraph"]
async fn graph_batch_rejects_retargeting_saved_relationship_id() {
    let (repo, world) = graph_fixture().await;
    let a = graph_entity(&world, "Retarget A");
    let b = graph_entity(&world, "Retarget B");
    let c = graph_entity(&world, "Retarget C");
    let edge = graph_relationship(&world, &a.id, &b.id, "KNOWS");
    repo.update_graph_batch(&world, &[(a.clone(), embedding()), (b.clone(), embedding()), (c.clone(), embedding())], &[edge.clone()], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let mut changed = edge.clone();
    changed.to_id = c.id;
    let result = repo.update_graph_batch(&world, &[], &[changed], &[], TEST_SOURCE_MESSAGE_ID).await;
    let neighbors = repo.get_entity_neighbors(&a.id, &world, None, None).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert!(result.is_err(), "An existing relationship ID must not silently select another target");
    assert_eq!(neighbors.len(), 1, "Rejected retargeting must not add a second edge");
    assert_eq!(neighbors[0].1.to_id, b.id, "Rejected retargeting must preserve the original target");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn chat_agent_has_no_graph_write_tools() {
    let (repo, world) = graph_fixture().await;
    let client = rig::providers::openrouter::new("unused-test-key");
    let embedding_model: rig::DynModel<rig::operation::Embedding> = client.embedding("openai/text-embedding-3-small", None).into();
    let retrieval = service::RetrievalService::new(repo.clone(), embedding_model);
    let factory = service::ChatAgentFactory::new(client.chat("unused-test-model").into(), "test".into(), retrieval, repo);
    let agent = factory.create_agent(&world, String::new());
    let mut names: Vec<_> = agent.tool_server_handle().static_tool_defs().into_iter().map(|d| d.name).collect();
    names.sort();
    assert_eq!(names, vec!["get_conflicts", "search_world"], "Chat must have conflict lookup while remaining read-only");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn researcher_agent_has_complete_lore_tools() {
    let (repo, world) = graph_fixture().await;
    let client = rig::providers::openrouter::new("unused-test-key");
    let embedding_model: rig::DynModel<rig::operation::Embedding> = client.embedding("openai/text-embedding-3-small", None).into();
    let retrieval = service::RetrievalService::new(repo.clone(), embedding_model.clone());
    let researcher = service::ResearcherService::new(client.chat("unused-test-model").into(), "test".into(), repo, retrieval, embedding_model);
    let agent = researcher.create_agent(world, uuid::Uuid::new_v4().to_string(), TEST_SOURCE_MESSAGE_ID.into());
    let mut names: Vec<_> = agent.tool_server_handle().static_tool_defs().into_iter().map(|d| d.name).collect();
    names.sort();
    assert_eq!(names, vec!["flag_conflict", "read_entity", "search_world", "update_graph"],
        "The researcher needs complete reads and conflict reporting alongside scoped graph writes");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn researcher_read_preserves_details_and_rejects_foreign_entities() {
    use rig::tool::PortableTool;
    let (repo, world) = graph_fixture().await;
    let a = graph_entity(&world, "Research read A");
    let mut b = graph_entity(&world, "Research read B");
    b.properties.insert("population".into(), serde_json::json!(53000));
    let mut edge = graph_relationship(&world, &a.id, &b.id, "VISITS");
    edge.description = "Only during winter.".into();
    repo.update_graph_batch(&world, &[(a.clone(), embedding()), (b, embedding())], &[edge.clone()], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let read = service::ReadEntity { world_id: world.clone(), graph_repo: repo.clone() };
    let details = read.call(service::ReadEntityArgs { entity_id: a.id.clone() }).await.unwrap();
    let foreign = service::ReadEntity { world_id: "other-world".into(), graph_repo: repo.clone() };
    let denied = foreign.call(service::ReadEntityArgs { entity_id: a.id }).await;
    repo.delete_graph(&world).await.unwrap();
    assert_eq!(details["neighbors"][0][0]["properties"]["population"], 53000,
        "Full reads must retain neighbor facts before replacement writes");
    assert_eq!(details["neighbors"][0][1]["description"], "Only during winter.",
        "Full reads must retain relationship conditions");
    assert_eq!(details["neighbors"][0][1]["id"], edge.id,
        "The researcher needs exact IDs to update or retract relationships");
    assert!(denied.is_err(), "Foreign entity IDs must not expose private lore");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "paid conversation smoke test; requires explicit authorization, isolated Postgres, and Memgraph"]
async fn researcher_smoke_conversation() {
    assert_eq!(std::env::var("QA_SMOKE_PAID").as_deref(), Ok("1"), "Explicit paid smoke-test opt-in required");
    let database = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    assert!(database.ends_with("/lorekeeper_adversarial"), "Smoke tests must use the isolated QA database");
    infrastructure::init_tracing();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:18001").await.unwrap();
    let (router, mut scheduler, dbos) = app::create_app().await;
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap(); });
    let result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("python3")
            .arg(std::env::var("QA_SMOKE_SCRIPT").unwrap_or_else(|_| "tests/smoke_conversation.py".into()))
            .env("QA_BASE_URL", "http://127.0.0.1:18001")
            .status().expect("Python with requests required")
    }).await.unwrap();
    let scheduler_result = scheduler.shutdown().await;
    dbos.shutdown().await;
    server.abort();
    scheduler_result.unwrap();
    assert!(result.success(), "Smoke test did not complete; inspect retained artifacts and logs");
}

#[tokio::test]
#[ignore = "requires isolated Postgres; makes no AI calls"]
async fn message_id_migration_preserves_history_and_existing_ids() {
    let database = std::env::var("DATABASE_URL").unwrap();
    assert!(database.ends_with("/lorekeeper_adversarial"), "Migration fixtures require the QA database");
    let pool = sqlx::PgPool::connect(&database).await.unwrap();
    let mut txn = pool.begin().await.unwrap();
    // A temporary table shadows the real table so this test changes no existing histories.
    sqlx::query("CREATE TEMP TABLE conversations (id uuid PRIMARY KEY, messages jsonb NOT NULL) ON COMMIT DROP")
        .execute(&mut *txn).await.unwrap();
    let conversation_id = uuid::Uuid::new_v4();
    let empty_id = uuid::Uuid::new_v4();
    let assistant_id = uuid::Uuid::new_v4();
    let original = serde_json::json!([
        {"role": "user", "content": "Asha owns a brass key.", "created_at": "2026-10-29T00:00:00Z"},
        {"role": "assistant", "id": assistant_id, "content": "Who knows?", "tool_calls": [{"id": "call-1", "name": "search_world", "arguments": {"query": "key"}}], "created_at": "2026-10-29T00:00:01Z"},
        {"role": "tool", "id": null, "tool_call_id": "call-1", "content": "No matches", "created_at": "2026-10-29T00:00:02Z"}
    ]);
    sqlx::query("INSERT INTO conversations VALUES ($1, $2), ($3, '[]'::jsonb)")
        .bind(conversation_id).bind(&original).bind(empty_id).execute(&mut *txn).await.unwrap();
    let migration = include_str!("../migrations/20261029000000_add_message_ids.sql");
    sqlx::raw_sql(migration).execute(&mut *txn).await.unwrap();
    let saved: serde_json::Value = sqlx::query_scalar("SELECT messages FROM conversations WHERE id = $1")
        .bind(conversation_id).fetch_one(&mut *txn).await.unwrap();
    sqlx::raw_sql(migration).execute(&mut *txn).await.unwrap();
    let again: serde_json::Value = sqlx::query_scalar("SELECT messages FROM conversations WHERE id = $1")
        .bind(conversation_id).fetch_one(&mut *txn).await.unwrap();
    let empty: serde_json::Value = sqlx::query_scalar("SELECT messages FROM conversations WHERE id = $1")
        .bind(empty_id).fetch_one(&mut *txn).await.unwrap();
    txn.rollback().await.unwrap();
    assert_eq!(saved, again, "Rerunning migration must not replace stable message IDs");
    assert_eq!(empty, serde_json::json!([]), "Empty histories must stay empty arrays");
    assert_eq!(saved[1]["id"], assistant_id.to_string(), "Migration must preserve existing IDs");
    let messages: Vec<data::Message> = serde_json::from_value(saved.clone()).unwrap();
    let ids: std::collections::HashSet<_> = messages.iter().map(data::Message::id).collect();
    assert_eq!(ids.len(), 3, "User, assistant, and tool messages need distinct IDs");
    let mut expected = original;
    for index in 0..3 {
        expected[index]["id"] = saved[index]["id"].clone();
    }
    assert_eq!(saved, expected, "Adding IDs must preserve content, roles, timestamps, order, and tool-call references");
}

#[tokio::test]
#[ignore = "requires isolated Postgres; makes no AI calls"]
async fn stored_message_ids_survive_repeated_reads() {
    let (pool, _, user_id, world_id, conversation_id) = world_deletion_fixture().await;
    let repo = data::ConversationRepository::new(pool.clone());
    let user = uuid::Uuid::new_v4();
    let assistant = uuid::Uuid::new_v4();
    let tool = uuid::Uuid::new_v4();
    let messages = [
        data::Message::User { id: user, content: "Asha owns a key.".into(), created_at: chrono::Utc::now() },
        data::Message::Assistant { id: assistant, content: None, tool_calls: None, created_at: chrono::Utc::now() },
        data::Message::Tool { id: tool, tool_call_id: "call-1".into(), content: "Saved".into(), created_at: chrono::Utc::now() },
    ];
    let appended = repo.append_messages(conversation_id, user_id, &messages).await.unwrap().unwrap();
    for _ in 0..2 {
        let read = repo.get_by_id(conversation_id, user_id).await.unwrap().unwrap();
        assert_eq!(read.messages.0.iter().map(data::Message::id).collect::<Vec<_>>(), vec![user, assistant, tool],
            "Loading history must never generate replacement message IDs");
        assert_eq!(serde_json::to_value(&read.messages).unwrap(), serde_json::to_value(&appended.messages).unwrap(),
            "Message payloads must remain stable across database reads");
    }
    sqlx::query("DELETE FROM worlds WHERE id = $1").bind(world_id).execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1").bind(user_id).execute(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn graph_source_message_ids_accumulate_without_duplicates() {
    use rig::tool::PortableTool;
    let (repo, world) = graph_fixture().await;
    let a = graph_entity(&world, "Source A");
    let b = graph_entity(&world, "Source B");
    let edge = graph_relationship(&world, &a.id, &b.id, "KNOWS");
    let first = uuid::Uuid::new_v4().to_string();
    let second = uuid::Uuid::new_v4().to_string();
    for source in [&first, &second, &first] {
        repo.update_graph_batch(&world, &[(a.clone(), embedding()), (b.clone(), embedding())], &[edge.clone()], &[], source).await.unwrap();
    }
    let read = service::ReadEntity { world_id: world.clone(), graph_repo: repo.clone() };
    let result = read.call(service::ReadEntityArgs { entity_id: a.id }).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    let expected = serde_json::json!([first, second]);
    assert_eq!(result["entity"]["source_message_ids"], expected,
        "Entity reads must expose all source IDs without duplicating a retried source");
    assert_eq!(result["neighbors"][0][0]["source_message_ids"], expected,
        "Neighbor reads must expose their saved source IDs");
    assert_eq!(result["neighbors"][0][1]["source_message_ids"], expected,
        "Relationship reads must preserve source IDs through replacement updates");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn legacy_graph_records_without_source_ids_remain_readable() {
    let settings = infrastructure::load_settings();
    let config = infrastructure::load_config();
    let graph = infrastructure::create_graph(&settings, &config.memgraph).await;
    let repo = data::GraphRepository::new(graph.clone());
    let world = format!("qa-legacy-sources-{}", uuid::Uuid::new_v4());
    let a = graph_entity(&world, "Legacy A");
    let b = graph_entity(&world, "Legacy B");
    let edge = graph_relationship(&world, &a.id, &b.id, "KNOWS");
    repo.update_graph_batch(&world, &[(a.clone(), embedding()), (b, embedding())], &[edge], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    graph.run(neo4rs::query("MATCH (e:Entity {world_id: $world}) REMOVE e.source_message_ids")
        .param("world", world.clone())).await.unwrap();
    graph.run(neo4rs::query("MATCH ()-[r]->() WHERE r.world_id = $world REMOVE r.source_message_ids")
        .param("world", world.clone())).await.unwrap();
    let entity = repo.get_entity(&a.id, &world).await.unwrap().unwrap();
    let neighbors = repo.get_entity_neighbors(&a.id, &world, None, None).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert!(entity.source_message_ids.is_empty(), "Missing historical sources must read as an empty list");
    assert!(neighbors[0].1.source_message_ids.is_empty(), "Legacy relationships must remain readable without invented sources");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn conflict_message_sources_accumulate_without_duplicates() {
    use rig::tool::PortableTool;
    let settings = infrastructure::load_settings();
    let config = infrastructure::load_config();
    let graph = infrastructure::create_graph(&settings, &config.memgraph).await;
    let repo = data::GraphRepository::new(graph.clone());
    let world = format!("qa-conflict-sources-{}", uuid::Uuid::new_v4());
    let entity = save_graph_entity(&repo, &graph_entity(&world, "Sealed archive")).await;
    let first = uuid::Uuid::new_v4().to_string();
    let second = uuid::Uuid::new_v4().to_string();
    for source in [&first, &second, &first] {
        let tool = service::FlagConflict {
            world_id: world.clone(), conversation_id: "qa".into(),
            source_message_id: source.clone(), graph_repo: repo.clone(),
        };
        tool.call(service::FlagConflictArgs {
            entity_id: entity.id.clone(), field: "access".into(),
            existing_value: serde_json::json!("sealed"), new_value: serde_json::json!("entered"),
            severity: "hard".into(), explanation: "Unresolved entry into sealed archive".into(),
        }).await.unwrap();
    }
    let conflicts = repo.get_conflicts_by_entity(&entity.id, &world).await.unwrap();
    assert_eq!(conflicts.len(), 1, "Reporting the same conflict must preserve one unresolved record");
    assert_eq!(conflicts[0].source_message_ids, vec![first, second],
        "Conflict sources must preserve earlier reports without duplicating retries");
    graph.run(neo4rs::query("MATCH (c:Conflict {world_id: $world}) REMOVE c.source_message_ids")
        .param("world", world.clone())).await.unwrap();
    let legacy = repo.get_conflicts_by_entity(&entity.id, &world).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert!(legacy[0].source_message_ids.is_empty(), "Legacy conflicts need empty source lists, not invented message IDs");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn conflict_lookup_excludes_resolved_and_foreign_records() {
    use rig::tool::PortableTool;
    let (repo, world) = graph_fixture().await;
    let foreign_world = format!("qa-foreign-conflicts-{}", uuid::Uuid::new_v4());
    let local = save_graph_entity(&repo, &graph_entity(&world, "Local archive")).await;
    let foreign = save_graph_entity(&repo, &graph_entity(&foreign_world, "Foreign archive")).await;
    let mut saved_local = None;
    for (scope, entity, field, resolved) in [
        (&world, &local, "access", false),
        (&world, &local, "opening", true),
        (&foreign_world, &foreign, "secret", false),
    ] {
        let conflict = data::Conflict {
            id: uuid::Uuid::new_v4().to_string(), world_id: scope.clone(), entity_id: entity.id.clone(),
            field: field.into(), existing_value: serde_json::json!("daylight"), new_value: serde_json::json!("midnight"),
            severity: "hard".into(), explanation: "Unresolved access hours".into(),
            source_conversation: "qa".into(), resolved: false, source_message_ids: vec![],
        };
        let mut saved = repo.upsert_conflict(&conflict, TEST_SOURCE_MESSAGE_ID).await.unwrap().unwrap();
        if resolved {
            saved.resolved = true;
            assert!(repo.update_conflict(&saved.id, scope, &saved).await.unwrap(), "Fixture must resolve the selected conflict");
        } else if scope == &world {
            saved_local = Some(saved);
        }
    }
    let tool = service::GetConflicts { graph_repo: repo.clone(), world_id: world.clone() };
    let results = tool.call(service::GetConflictsArgs {}).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    repo.delete_graph(&foreign_world).await.unwrap();
    assert_eq!(results.len(), 1, "Lookup must exclude resolved conflicts and other writers' records");
    let expected = saved_local.unwrap();
    assert_eq!(results[0].id, expected.id, "Lookup returns the wrong conflict");
    assert_eq!(results[0].entity_id, local.id, "Conflict subject identity must reach chat");
    assert_eq!(results[0].existing_value, serde_json::json!("daylight"), "Original claim must reach chat");
    assert_eq!(results[0].new_value, serde_json::json!("midnight"), "Disputed claim must reach chat");
    assert_eq!(results[0].source_message_ids, vec![TEST_SOURCE_MESSAGE_ID], "Source messages must reach chat");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn conflict_lookup_empty_world_uses_object_arguments() {
    use rig::tool::PortableTool;
    let (repo, world) = graph_fixture().await;
    let tool = service::GetConflicts { graph_repo: repo, world_id: world };
    assert_eq!(tool.parameters()["type"], "object", "Tool arguments must be an object for provider tool calls");
    assert!(tool.call(service::GetConflictsArgs {}).await.unwrap().is_empty(), "An empty world must return an empty conflict list");
}

#[test]
fn tool_text_search_rejects_invalid_queries_before_execution() {
    for query in ["".to_owned(), " \u{200b} ".into(), "safe\0evil".into(), "safe\u{202e}evil".into(), "x".repeat(501)] {
        assert!(serde_json::from_value::<service::SearchWorldArgs>(serde_json::json!({"query": query})).is_err(),
            "Invalid queries reach embedding and search calls");
    }
    let args: service::SearchWorldArgs = serde_json::from_value(serde_json::json!({"query": "  Café   council "})).unwrap();
    assert_eq!(args.query.as_str(), "Café council", "Search uses unnormalized input");
}

#[test]
fn tool_text_graph_rejects_invalid_names_aliases_and_descriptions() {
    let valid = serde_json::json!({"name":"Mira", "aliases":[], "kind":["character"], "description":"Engineer", "properties":{}});
    for (field, value) in [
        ("name", serde_json::json!(" \u{200b} ")),
        ("name", serde_json::json!("x".repeat(256))),
        ("aliases", serde_json::json!(["safe\u{202e}evil"])),
        ("aliases", serde_json::json!([""])),
        ("description", serde_json::json!("safe\0evil")),
        ("description", serde_json::json!("x".repeat(10_001))),
    ] {
        let mut entity = valid.clone();
        entity[field] = value;
        assert!(serde_json::from_value::<service::UpdateGraphArgs>(serde_json::json!({"entities":[entity]})).is_err(),
            "Invalid graph text reaches embedding or storage: {field}");
    }
    let relationship = serde_json::json!({"from_id":"a", "to_id":"b", "rel_type":"KNOWS", "description":"bad\0text", "properties":{}});
    assert!(serde_json::from_value::<service::UpdateGraphArgs>(serde_json::json!({"relationships":[relationship]})).is_err(),
        "Relationship descriptions bypass text validation");
}

#[test]
fn tool_text_graph_conversion_preserves_normalized_values() {
    let input: service::GraphEntityInput = serde_json::from_value(serde_json::json!({
        "name":" Cafe\u{301}\u{200b}  Keeper ", "aliases":["  The   Keeper "], "kind":["character"],
        "description":" first\r\nsecond ", "properties":{}
    })).unwrap();
    let entity = data::Entity::from((input, "world"));
    assert_eq!(entity.name, "Café Keeper", "Saved entity name bypasses normalization");
    assert_eq!(entity.name_key, data::name_key("Café Keeper"), "Name key differs from normalized name");
    assert_eq!(entity.aliases, vec!["The Keeper"], "Aliases bypass normalization");
    assert_eq!(entity.description, "first\nsecond", "Prose loses its line structure");
}

#[test]
fn tool_text_schemas_publish_string_length_bounds() {
    let query = serde_json::to_value(schemars::schema_for!(shared::SearchQuery)).unwrap();
    assert_eq!(query["type"], "string");
    assert_eq!(query["minLength"], 1);
    assert_eq!(query["maxLength"], 500);
    let name = serde_json::to_value(schemars::schema_for!(shared::EntityName)).unwrap();
    assert_eq!(name["maxLength"], 255);
    let description = serde_json::to_value(schemars::schema_for!(shared::Description)).unwrap();
    assert_eq!(description["minLength"], 0);
    let search = serde_json::to_value(schemars::schema_for!(service::SearchWorldArgs)).unwrap();
    assert!(search["properties"]["query"].is_object(), "Search tool schema loses its query field");
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres; makes no AI calls"]
async fn dbos_research_registration_starts_and_shuts_down() {
    let database = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    assert!(database.ends_with("/lorekeeper_adversarial"), "DBOS startup must use QA data");
    let _ = tracing_subscriber::fmt().with_ansi(false).with_env_filter("info").try_init();
    let (_router, mut scheduler, dbos) = app::create_app().await;
    assert!(dbos.is_launched(), "DBOS stops before startup returns");
    let scheduler_result = scheduler.shutdown().await;
    dbos.shutdown().await;
    scheduler_result.unwrap();
    assert!(!dbos.is_launched(), "DBOS remains active after shutdown");
}

#[path = "cases/durable_research.rs"]
mod durable_research;

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn graph_page_keeps_edges_within_its_nodes_and_world() {
    let (repo, world) = graph_fixture().await;
    let foreign_world = format!("qa-graph-{}", uuid::Uuid::new_v4());
    let mut a = graph_entity(&world, "Page A");
    let mut b = graph_entity(&world, "Page B");
    let mut c = graph_entity(&world, "Page C");
    a.id = uuid::Uuid::from_u128(1).to_string();
    b.id = uuid::Uuid::from_u128(2).to_string();
    c.id = uuid::Uuid::from_u128(3).to_string();
    let ab = graph_relationship(&world, &a.id, &b.id, "KNOWS");
    let ba = graph_relationship(&world, &b.id, &a.id, "MENTORS");
    let bc = graph_relationship(&world, &b.id, &c.id, "KNOWS");
    repo.update_graph_batch(&world, &[(a.clone(), embedding()), (b.clone(), embedding()), (c.clone(), embedding())],
        &[ab.clone(), ba.clone(), bc], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let mut foreign = graph_entity(&foreign_world, "Foreign page B");
    foreign.id = b.id.clone();
    repo.update_graph_batch(&foreign_world, &[(foreign, embedding())], &[], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    // A malformed foreign connection must not enter a page, even with a matching endpoint ID.
    let settings = infrastructure::load_settings();
    let config = infrastructure::load_config();
    let graph = infrastructure::create_graph(&settings, &config.memgraph).await;
    graph.run(neo4rs::query("MATCH (a:Entity {world_id: $world, id: $a}), (b:Entity {world_id: $foreign, id: $b}) CREATE (a)-[:LEAKS {world_id: $world}]->(b)")
        .param("world", world.as_str()).param("foreign", foreign_world.as_str())
        .param("a", a.id.as_str()).param("b", b.id.as_str())).await.unwrap();
    let first = repo.get_graph(&world, 2, None).await.unwrap();
    let second = repo.get_graph(&world, 2, first.2.as_deref()).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    repo.delete_graph(&foreign_world).await.unwrap();
    assert_eq!(first.0.iter().map(|e| &e.id).collect::<Vec<_>>(), vec![&a.id, &b.id], "Graph pages lose stable ID ordering");
    assert!(first.0.iter().all(|e| e.world_id == world), "Graph page exposes a foreign entity");
    assert_eq!(first.1.len(), 2, "Page includes cross-page/foreign edges or drops internal edges");
    assert!(first.1.iter().any(|r| r.id == ab.id && r.from_id == a.id && r.to_id == b.id), "Graph loses edge direction");
    assert!(first.1.iter().any(|r| r.id == ba.id), "Reverse-direction edge disappears");
    assert_eq!(first.2.as_deref(), Some(b.id.as_str()), "Cursor points past an entity that was not returned");
    assert_eq!(second.0.len(), 1, "Next page repeats or skips an entity");
    assert_eq!(second.0[0].id, c.id, "Next page does not resume after its cursor");
    assert!(second.1.is_empty(), "Graph returns an edge with a missing endpoint");
    assert!(second.2.is_none(), "Last page advertises another page");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn graph_page_exact_limit_has_no_next_cursor() {
    let (repo, world) = graph_fixture().await;
    let a = graph_entity(&world, "Exactly one");
    repo.update_graph_batch(&world, &[(a.clone(), embedding())], &[], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let page = repo.get_graph(&world, 1, None).await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert_eq!(page.0.len(), 1, "Exact-size page loses its last record");
    assert!(page.2.is_none(), "Exact-size final page produces a useless extra request");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn graph_page_empty_world_has_no_cursor_or_edges() {
    let (repo, world) = graph_fixture().await;
    let page = repo.get_graph(&world, 50, None).await.unwrap();
    assert!(page.0.is_empty() && page.1.is_empty() && page.2.is_none(), "Empty world returns phantom graph data or a cursor");
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn graph_page_rejects_invalid_limits_and_cursors() {
    let (repo, world) = graph_fixture().await;
    for limit in [0, 101, u32::MAX] {
        assert!(matches!(repo.get_graph(&world, limit, None).await, Err(infrastructure::InfrastructureError::InvalidInput(_))),
            "Unbounded graph page limit reaches the database");
    }
    for cursor in ["", "not-a-cursor", "' RETURN 1 //"] {
        assert!(matches!(repo.get_graph(&world, 50, Some(cursor)).await, Err(infrastructure::InfrastructureError::InvalidInput(_))),
            "Malformed cursors are accepted or reported as server failures");
    }
    let error: app::errors::AppError = service::ServiceError::from(
        infrastructure::InfrastructureError::InvalidInput("Invalid graph cursor".into())).into();
    use axum::response::IntoResponse;
    assert_eq!(error.into_response().status(), axum::http::StatusCode::UNPROCESSABLE_ENTITY,
        "Invalid pagination input becomes an HTTP 500");
}

#[tokio::test]
#[ignore = "requires isolated Postgres and live Memgraph; makes no AI calls"]
async fn world_reads_reject_foreign_users_and_missing_records() {
    let (pool, worlds, owner, world, _) = world_deletion_fixture().await;
    let (graph, _) = graph_fixture().await;
    let scope = world.to_string();
    let a = graph_entity(&scope, "Reader A");
    let b = graph_entity(&scope, "Reader B");
    let edge = graph_relationship(&scope, &a.id, &b.id, "KNOWS");
    graph.update_graph_batch(&scope, &[(a.clone(), embedding()), (b, embedding())], &[edge.clone()], &[], TEST_SOURCE_MESSAGE_ID).await.unwrap();
    let conflict = data::Conflict {
        id: uuid::Uuid::new_v4().to_string(), world_id: scope.clone(), entity_id: a.id,
        field: "age".into(), existing_value: serde_json::json!(20), new_value: serde_json::json!(30),
        severity: "hard".into(), explanation: "Conflicting ages".into(),
        source_conversation: "qa".into(), resolved: false, source_message_ids: vec![],
    };
    let saved = graph.upsert_conflict(&conflict, TEST_SOURCE_MESSAGE_ID).await.unwrap().unwrap();
    let service = service::WorldService::new(worlds, graph.clone());
    let edge_id = uuid::Uuid::parse_str(&edge.id).unwrap();
    let conflict_id = uuid::Uuid::parse_str(&saved.id).unwrap();
    let query = data::ConflictListQuery { limit: 1, cursor: None, resolved: None };
    let stranger = uuid::Uuid::new_v4();
    assert!(matches!(service.get_relationship(world, stranger, edge_id).await, Err(service::ServiceError::NotFound(_))), "Foreign user reads a private relationship");
    assert!(matches!(service.get_conflict(world, stranger, conflict_id).await, Err(service::ServiceError::NotFound(_))), "Foreign user reads private conflict claims");
    assert!(matches!(service.list_conflicts(world, stranger, &query).await, Err(service::ServiceError::NotFound(_))), "Foreign user lists private conflicts");
    assert_eq!(service.get_relationship(world, owner, edge_id).await.unwrap().id, edge.id, "Owner cannot read the relationship");
    assert_eq!(service.get_conflict(world, owner, conflict_id).await.unwrap().id, saved.id, "Owner cannot read the conflict");
    assert_eq!(service.list_conflicts(world, owner, &query).await.unwrap().data.len(), 1, "Owner loses saved conflicts");
    let missing = uuid::Uuid::new_v4();
    assert!(matches!(service.get_relationship(world, owner, missing).await, Err(service::ServiceError::NotFound(_))), "Missing relationship does not return NotFound");
    assert!(matches!(service.get_conflict(world, owner, missing).await, Err(service::ServiceError::NotFound(_))), "Missing conflict does not return NotFound");
    graph.delete_graph(&scope).await.unwrap();
    sqlx::query("DELETE FROM worlds WHERE id = $1").bind(world).execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1").bind(owner).execute(&pool).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn world_reads_conflict_pages_preserve_filters_and_scope() {
    let (repo, world) = graph_fixture().await;
    let entity = save_graph_entity(&repo, &graph_entity(&world, "Conflict pages")).await;
    for index in 1..=3 {
        let conflict = data::Conflict {
            id: uuid::Uuid::from_u128(index).to_string(), world_id: world.clone(), entity_id: entity.id.clone(),
            field: format!("field-{index}"), existing_value: serde_json::json!("old"), new_value: serde_json::json!("new"),
            severity: "hard".into(), explanation: "QA conflict".into(), source_conversation: "qa".into(),
            resolved: false, source_message_ids: vec![],
        };
        let mut saved = repo.upsert_conflict(&conflict, TEST_SOURCE_MESSAGE_ID).await.unwrap().unwrap();
        if index == 2 {
            saved.resolved = true;
            repo.update_conflict(&saved.id, &world, &saved).await.unwrap();
        }
    }
    let first = repo.list_conflicts(&world, 1, None, Some(false)).await.unwrap();
    let second = repo.list_conflicts(&world, 1, first.1.as_deref(), Some(false)).await.unwrap();
    let resolved = repo.list_conflicts(&world, 10, None, Some(true)).await.unwrap();
    let all = repo.list_conflicts(&world, 10, None, None).await.unwrap();
    let foreign = repo.list_conflicts("missing-world", 10, None, None).await.unwrap();
    let foreign_lookup = repo.get_conflict(&first.0[0].id, "missing-world").await.unwrap();
    repo.delete_graph(&world).await.unwrap();
    assert_eq!(first.0[0].id, uuid::Uuid::from_u128(1).to_string(), "First conflict page loses ID order");
    assert_eq!(second.0[0].id, uuid::Uuid::from_u128(3).to_string(), "Cursor skips unresolved conflicts or includes resolved records");
    assert!(second.1.is_none(), "Final conflict page advertises another page");
    assert_eq!(resolved.0.len(), 1, "Resolved filter includes unresolved conflicts");
    assert!(resolved.0[0].resolved, "Resolved filter returns the wrong claim");
    assert_eq!(all.0.len(), 3, "Omitted filter excludes stored conflicts");
    assert!(foreign.0.is_empty() && foreign_lookup.is_none(), "Foreign world receives private conflicts");
    assert!(matches!(repo.list_conflicts(&world, 0, None, None).await, Err(infrastructure::InfrastructureError::InvalidInput(_))), "Zero limit bypasses pagination validation");
    assert!(matches!(repo.list_conflicts(&world, 1, Some("invalid"), None).await, Err(infrastructure::InfrastructureError::InvalidInput(_))), "Malformed conflict cursor reaches the database");
}
