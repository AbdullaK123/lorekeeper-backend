//! Opt-in live tests. See tests/README.md for isolated setup.
#[path = "../src/app/mod.rs"]
mod app;
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
    let server = tokio::spawn(async move {
        axum::serve(listener, app::create_app().await).await.unwrap();
    });
    let result = tokio::task::spawn_blocking(|| {
        std::process::Command::new("python3")
            .args(["tests/adversarial_api.py", "-v"])
            .args(std::env::var("QA_TESTS").unwrap_or_default().split_whitespace())
            .env("QA_BASE_URL", "http://127.0.0.1:18000")
            .status()
            .expect("python3 with requests must be installed")
    }).await.unwrap();
    server.abort();
    assert!(result.success(), "Live failures identify behavior that can harm users; inspect Python assertions");
}

async fn graph_fixture() -> (data::GraphRepository, String) {
    let settings = infrastructure::load_settings();
    let config = infrastructure::load_config();
    let graph = infrastructure::create_graph(&settings, &config.memgraph).await;
    (data::GraphRepository::new(graph), format!("qa-graph-{}", uuid::Uuid::new_v4()))
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
