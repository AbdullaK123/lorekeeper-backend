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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "requires an isolated Postgres database, Memgraph, and an OpenRouter key"]
async fn adversarial_live_api() {
    let database = std::env::var("DATABASE_URL").expect("DATABASE_URL is required");
    assert!(database.ends_with("/lorekeeper_adversarial"), "Fault injection must never target development data");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:18000").await.unwrap();
    let (router, mut scheduler) = app::create_app().await;
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
    scheduler.shutdown().await.unwrap();
    server.abort();
    assert!(result.success(), "Live failures identify behavior that can harm users; inspect Python assertions");
}

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
    graph.upsert_entity(&graph_entity(&first.to_string(), "First"), embedding()).await.unwrap();
    graph.upsert_entity(&graph_entity(&second.to_string(), "Second"), embedding()).await.unwrap();
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
    }
}

fn embedding() -> Vec<f64> {
    let mut value = vec![0.0; 1536];
    value[0] = 1.0;
    value
}

fn graph_relationship(world: &str, from: &str, to: &str, label: &str) -> data::Relationship {
    data::Relationship {
        id: uuid::Uuid::new_v4().to_string(), world_id: world.into(),
        from_id: from.into(), to_id: to.into(), rel_type: label.into(),
        description: "QA edge".into(), properties: std::collections::HashMap::new(),
        source_conversation: "qa".into(),
    }
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_retry_preserves_entity_identity() {
    let (repo, world) = graph_fixture().await;
    let first = repo.upsert_entity(&graph_entity(&world, "Kael"), embedding()).await.unwrap();
    let second = repo.upsert_entity(&graph_entity(&world, "  KAEL  "), embedding()).await.unwrap();
    assert_eq!(first.id, second.id, "Retrying normalized names duplicates lore nodes");
    assert_eq!(repo.get_entities_by_world(&world).await.unwrap().len(), 1,
        "A retry grows the graph instead of updating the existing entity");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_relationships_cannot_cross_worlds() {
    let (repo, world) = graph_fixture().await;
    let other = format!("qa-graph-{}", uuid::Uuid::new_v4());
    let a = repo.upsert_entity(&graph_entity(&world, "A"), embedding()).await.unwrap();
    let b = repo.upsert_entity(&graph_entity(&other, "B"), embedding()).await.unwrap();
    let result = repo.upsert_relationship(&graph_relationship(&world, &a.id, &b.id, "KNOWS")).await.unwrap();
    assert!(result.is_none(), "An edge connects private lore across user worlds");
    assert!(repo.get_entity(&a.id, &other).await.unwrap().is_none(),
        "A guessed entity ID bypasses world isolation");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_relationship_retry_does_not_duplicate_edges() {
    let (repo, world) = graph_fixture().await;
    let a = repo.upsert_entity(&graph_entity(&world, "A"), embedding()).await.unwrap();
    let b = repo.upsert_entity(&graph_entity(&world, "B"), embedding()).await.unwrap();
    let first = repo.upsert_relationship(&graph_relationship(&world, &a.id, &b.id, "KNOWS")).await.unwrap().unwrap();
    let second = repo.upsert_relationship(&graph_relationship(&world, &a.id, &b.id, "KNOWS")).await.unwrap().unwrap();
    assert_eq!(first.id, second.id, "Retrying extraction duplicates a relationship");
    assert_eq!(repo.get_entity_neighbors(&a.id, &world, None, None).await.unwrap().len(), 1,
        "Repeated edges inflate retrieval context and ranking");
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_relationship_injection_is_rejected() {
    let (repo, world) = graph_fixture().await;
    for label in ["", "1BAD", "KNOWS]->(n) DETACH DELETE n //", "A`B", "A B", "认识"] {
        let result = repo.upsert_relationship(&graph_relationship(&world, "a", "b", label)).await;
        assert!(matches!(result, Err(infrastructure::InfrastructureError::CypherInjection(_))),
            "Untrusted relationship labels can alter a graph query: {label}");
    }
}

#[tokio::test]
#[ignore = "requires live Memgraph"]
async fn graph_delete_entity_removes_attached_conflicts() {
    let (repo, world) = graph_fixture().await;
    let entity = repo.upsert_entity(&graph_entity(&world, "A"), embedding()).await.unwrap();
    let conflict = data::Conflict {
        id: uuid::Uuid::new_v4().to_string(), world_id: world.clone(), entity_id: entity.id.clone(),
        field: "description".into(), existing_value: serde_json::json!("old"),
        new_value: serde_json::json!("new"), severity: "hard".into(), explanation: "QA".into(),
        source_conversation: "qa".into(), resolved: false,
    };
    repo.upsert_conflict(&conflict).await.unwrap().expect("QA conflict must exist");
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
    let written = repo.upsert_entity(&entity, embedding()).await.unwrap();
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
