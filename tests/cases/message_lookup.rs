//! Source-message repository reads against real isolated Postgres.
use super::*;

async fn fixture() -> (
    sqlx::PgPool,
    data::ConversationRepository,
    uuid::Uuid,
    uuid::Uuid,
    uuid::Uuid,
    Vec<data::Message>,
) {
    let (pool, _, owner, world, conversation) = world_deletion_fixture().await;
    let repo = data::ConversationRepository::new(pool.clone());
    let now = chrono::Utc::now();
    let messages = vec![
        data::Message::User {
            id: uuid::Uuid::new_v4(),
            content: "The harbor closes at dusk.".into(),
            created_at: now,
        },
        data::Message::Assistant {
            id: uuid::Uuid::new_v4(),
            content: None,
            tool_calls: Some(vec![data::ToolCall {
                id: "qa-call".into(),
                name: "search_world".into(),
                arguments: serde_json::json!({"query": "harbor"}),
            }]),
            created_at: now,
        },
        data::Message::Tool {
            id: uuid::Uuid::new_v4(),
            tool_call_id: "qa-call".into(),
            content: "Saved harbor lore".into(),
            created_at: now,
        },
    ];
    repo.append_messages(conversation, owner, &messages)
        .await
        .unwrap()
        .unwrap();
    (pool, repo, owner, world, conversation, messages)
}

async fn cleanup(pool: &sqlx::PgPool, owner: uuid::Uuid, world: uuid::Uuid) {
    sqlx::query("DELETE FROM worlds WHERE id = $1")
        .bind(world)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires isolated Postgres; makes no AI calls"]
async fn message_lookup_preserves_each_stored_role() {
    let (pool, repo, owner, world, conversation, messages) = fixture().await;
    for message in &messages {
        let row = repo
            .get_message(world, owner, &message.id().to_string())
            .await
            .unwrap()
            .expect("Saved source message must remain discoverable");
        let response = data::MessageResponse::from(row);
        assert_eq!(
            response.world_id, world,
            "Source lookup returns the wrong world"
        );
        assert_eq!(
            response.conversation_id, conversation,
            "Source lookup cannot navigate to the correct conversation"
        );
        assert_eq!(
            serde_json::to_value(response.message).unwrap(),
            serde_json::to_value(message).unwrap(),
            "Source lookup changes message role, content, timestamp, or tool details"
        );
    }
    cleanup(&pool, owner, world).await;
}

#[tokio::test]
#[ignore = "requires isolated Postgres; makes no AI calls"]
async fn message_lookup_rejects_wrong_scope_and_missing_sources() {
    let (pool, repo, owner, world, _, messages) = fixture().await;
    let id = messages[0].id().to_string();
    assert!(
        repo.get_message(world, uuid::Uuid::new_v4(), &id)
            .await
            .unwrap()
            .is_none(),
        "Another user receives private source text"
    );
    assert!(
        repo.get_message(uuid::Uuid::new_v4(), owner, &id)
            .await
            .unwrap()
            .is_none(),
        "Source lookup ignores the requested world"
    );
    assert!(
        repo.get_message(world, owner, &uuid::Uuid::new_v4().to_string())
            .await
            .unwrap()
            .is_none(),
        "Missing message returns unrelated conversation text"
    );
    cleanup(&pool, owner, world).await;
}

#[tokio::test]
#[ignore = "requires isolated Postgres; makes no AI calls"]
async fn message_lookup_returns_none_after_conversation_deletion() {
    let (pool, repo, owner, world, conversation, messages) = fixture().await;
    assert!(
        repo.delete(conversation, owner).await.unwrap(),
        "Fixture conversation must be deleted"
    );
    assert!(
        repo.get_message(world, owner, &messages[0].id().to_string())
            .await
            .unwrap()
            .is_none(),
        "Deleted conversation still exposes its source messages"
    );
    cleanup(&pool, owner, world).await;
}
