//! Real conflict-tool validation and acceptance. No provider calls.
use super::*;
use rig::tool::PortableTool;

#[tokio::test]
#[ignore = "requires isolated Postgres and Memgraph; makes no AI calls"]
async fn conflict_targets_report_then_accept_saved_property() {
    let (pool, _worlds, owner, world, _) = world_deletion_fixture().await;
    let (graph, _) = graph_fixture().await;
    let scope = world.to_string();
    let mut entity = graph_entity(&scope, "Exact conflict target");
    entity
        .properties
        .insert("age".into(), serde_json::json!(27));
    entity
        .properties
        .insert("title".into(), serde_json::json!("Captain"));
    let entity = save_graph_entity(&graph, &entity).await;
    let source = uuid::Uuid::new_v4().to_string();
    let tool = service::FlagConflict {
        world_id: scope.clone(),
        conversation_id: "qa".into(),
        source_message_id: source.clone(),
        graph_repo: graph.clone(),
    };
    tool.call(service::FlagConflictArgs {
        entity_id: entity.id.clone(),
        field: "age".into(),
        new_value: serde_json::json!(31),
        severity: "hard".into(),
        explanation: "Writer states a different age".into(),
    })
    .await
    .unwrap();
    let conflicts = graph.get_conflicts(&scope).await.unwrap();
    assert_eq!(
        conflicts.len(),
        1,
        "A valid exact target does not create one visible conflict"
    );
    let conflict = &conflicts[0];
    assert_eq!(
        conflict.existing_value,
        serde_json::json!(27),
        "Original claim does not come from stored lore"
    );
    assert_eq!(
        conflict.new_value,
        serde_json::json!(31),
        "Reported claim loses its numeric JSON type"
    );
    assert_eq!(
        conflict.source_message_ids,
        vec![source.clone()],
        "Reported conflict loses its message source"
    );
    assert_eq!(
        graph
            .get_entity(&entity.id, &scope)
            .await
            .unwrap()
            .unwrap()
            .properties,
        entity.properties,
        "Reporting a disputed claim changes canon before writer acceptance"
    );
    let description = "A 31-year-old captain from Bellhaven.";
    let accepted = graph
        .accept_conflict(&conflict.id, &scope, description)
        .await
        .unwrap()
        .unwrap();
    assert!(
        accepted.resolved,
        "Writer acceptance leaves the reported conflict unresolved"
    );
    assert!(
        matches!(accepted.decision, Some(data::ConflictDecision::AcceptNew)),
        "Acceptance records the wrong decision"
    );
    let updated = graph.get_entity(&entity.id, &scope).await.unwrap().unwrap();
    assert_eq!(
        updated.description, description,
        "Acceptance updates the property but leaves a stale description"
    );
    graph
        .accept_conflict(
            &conflict.id,
            &scope,
            "A repeated request must not overwrite lore",
        )
        .await
        .unwrap();
    let repeated = graph.get_entity(&entity.id, &scope).await.unwrap().unwrap();
    assert_eq!(
        repeated.description, description,
        "Repeated acceptance overwrites the first accepted description"
    );
    assert_eq!(
        updated.properties["age"],
        serde_json::json!(31),
        "Acceptance does not apply the tool's saved claim"
    );
    assert_eq!(
        updated.properties["title"],
        serde_json::json!("Captain"),
        "Acceptance changes an unrelated fact"
    );
    assert!(
        updated.source_message_ids.contains(&source),
        "Accepted lore loses the disputed claim's source"
    );
    assert!(
        graph.get_conflicts(&scope).await.unwrap().is_empty(),
        "Accepted claim remains in unresolved conflict reads"
    );
    graph.delete_graph(&scope).await.unwrap();
    sqlx::query("DELETE FROM worlds WHERE id = $1")
        .bind(world)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn conflict_targets_reject_non_property_labels() {
    let (graph, world) = graph_fixture().await;
    let mut entity = graph_entity(&world, "Property labels");
    entity
        .properties
        .insert("birthplace".into(), serde_json::json!("Ashlands"));
    let entity = save_graph_entity(&graph, &entity).await;
    let tool = service::FlagConflict {
        world_id: world.clone(),
        conversation_id: "qa".into(),
        source_message_id: TEST_SOURCE_MESSAGE_ID.into(),
        graph_repo: graph.clone(),
    };
    for field in [
        "place of birth",
        "Birthplace",
        "properties.birthplace",
        "description",
        "",
        "unknown",
    ] {
        let result = tool
            .call(service::FlagConflictArgs {
                entity_id: entity.id.clone(),
                field: field.into(),
                new_value: serde_json::json!("Northern Wastes"),
                severity: "hard".into(),
                explanation: "QA disputed birthplace".into(),
            })
            .await;
        assert!(
            matches!(result, Err(service::ServiceError::InvalidInput(_))),
            "An unresolved field label creates an unresolvable saved conflict: {field}"
        );
    }
    assert!(
        graph.get_conflicts(&world).await.unwrap().is_empty(),
        "Rejected field labels still write conflicts"
    );
    assert_eq!(
        graph
            .get_entity(&entity.id, &world)
            .await
            .unwrap()
            .unwrap()
            .properties,
        entity.properties,
        "Rejected labels mutate saved lore"
    );
    graph.delete_graph(&world).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Memgraph; makes no AI calls"]
async fn conflict_targets_reject_foreign_entities() {
    let (graph, world) = graph_fixture().await;
    let mut entity = graph_entity(&world, "Foreign conflict subject");
    entity
        .properties
        .insert("age".into(), serde_json::json!(27));
    let entity = save_graph_entity(&graph, &entity).await;
    let tool = service::FlagConflict {
        world_id: format!("qa-foreign-{}", uuid::Uuid::new_v4()),
        conversation_id: "qa".into(),
        source_message_id: TEST_SOURCE_MESSAGE_ID.into(),
        graph_repo: graph.clone(),
    };
    let result = tool
        .call(service::FlagConflictArgs {
            entity_id: entity.id.clone(),
            field: "age".into(),
            new_value: serde_json::json!(31),
            severity: "hard".into(),
            explanation: "QA foreign claim".into(),
        })
        .await;
    assert!(
        matches!(result, Err(service::ServiceError::NotFound(_))),
        "Tool reads another world's saved property"
    );
    assert!(
        graph.get_conflicts(&world).await.unwrap().is_empty(),
        "Foreign tool creates a conflict against private lore"
    );
    graph.delete_graph(&world).await.unwrap();
}
