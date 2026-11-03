use crate::service::{ChatAgentFactory, RetrievalService};
use axum::{Json, Router, extract::State, routing::post};
use rig::{DynModel, message::Message, operation::Completion};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

async fn capture(
    State(captured): State<Arc<Mutex<Value>>>,
    Json(request): Json<Value>,
) -> Json<Value> {
    *captured.lock().unwrap() = request;
    Json(
        json!({"id":"test", "object":"chat.completion", "created":1, "model":"local",
        "choices":[{"index":0,"message":{"role":"assistant","content":"41"},"finish_reason":"stop"}],
        "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}}),
    )
}

#[tokio::test]
#[ignore = "requires local Memgraph; model requests stay on localhost"]
async fn current_lore_is_system_context_without_rewriting_old_user_messages() {
    let captured = Arc::new(Mutex::new(Value::Null));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = Router::new()
        .route("/chat/completions", post(capture))
        .with_state(captured.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let client = rig::providers::openai::OpenAIConfig::new("local-only")
        .with_base_url(format!("http://{address}"))
        .client();
    let model: DynModel<Completion> = client.chat("local").into();
    let (graph, world) = super::graph_fixture().await;
    let unused = rig::providers::openrouter::new("unused-key");
    let retrieval = RetrievalService::new(graph.clone(), unused.embedding("unused", None).into());
    let factory = ChatAgentFactory::new(model, "Follow current canon.".into(), retrieval, graph);
    let agent = factory.create_agent(&world, "Mara Vale\nProperties: age: 41".into());
    agent
        .prompt("How old is Mara now?")
        .history(vec![
            Message::user("Mara is 34."),
            Message::assistant("She is 34."),
        ])
        .await
        .unwrap();
    server.abort();
    let request = captured.lock().unwrap();
    let messages = request["messages"].as_array().unwrap();
    assert!(
        messages.iter().any(|message| message["role"] == "system"
            && message["content"].to_string().contains("age: 41")),
        "Current lore loses system authority and can be overridden by stale chat history"
    );
    let old_user = messages
        .iter()
        .find(|message| message["role"] == "user")
        .unwrap();
    assert_eq!(
        old_user["content"], "Mara is 34.",
        "Current lore is injected into the old statement instead of being separate current context"
    );
}
