//! Real DBOS/Postgres replay with a local scripted model. No paid provider calls.
use crate::service::{self, ServiceError};
use axum::{Json, Router, extract::State, routing::post};
use rig::{Agent, AgentBuilder, DynModel, operation::Completion, tool::PortableTool};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

#[derive(Default)]
struct Script {
    models: AtomicUsize,
    tools: AtomicUsize,
    block_final: AtomicBool,
    final_entered: tokio::sync::Notify,
    release_final: tokio::sync::Notify,
    unknown_tool: AtomicBool,
    malformed_args: AtomicBool,
}

async fn model_reply(State(script): State<Arc<Script>>, Json(request): Json<Value>) -> Json<Value> {
    script.models.fetch_add(1, Ordering::SeqCst);
    let has_results = request["messages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["role"] == "tool");
    let (message, reason) = if has_results {
        script.final_entered.notify_one();
        if script.block_final.load(Ordering::SeqCst) {
            script.release_final.notified().await;
        }
        (
            json!({"role":"assistant", "content":"Research complete"}),
            "stop",
        )
    } else {
        let name = if script.unknown_tool.load(Ordering::SeqCst) {
            "unregistered_tool"
        } else {
            "record_fact"
        };
        let first = if script.malformed_args.load(Ordering::SeqCst) {
            "{}".to_owned()
        } else {
            json!({"text":"first"}).to_string()
        };
        (
            json!({"role":"assistant", "content":null, "tool_calls":[
                {"id":"call-first", "type":"function", "function":{"name":name,"arguments":first}},
                {"id":"call-second", "type":"function", "function":{"name":name,"arguments":json!({"text":"second"}).to_string()}}
            ]}),
            "tool_calls",
        )
    };
    Json(
        json!({"id":"local-test", "object":"chat.completion", "created":1, "model":"local-test",
        "choices":[{"index":0,"message":message,"finish_reason":reason}],
        "usage":{"prompt_tokens":10,"completion_tokens":5,"total_tokens":15}}),
    )
}

#[derive(Deserialize, schemars::JsonSchema)]
struct RecordArgs {
    text: String,
}
struct RecordFact(Arc<Script>);
impl PortableTool for RecordFact {
    const NAME: &'static str = "record_fact";
    type Args = RecordArgs;
    type Output = String;
    type Error = ServiceError;
    fn description(&self) -> String {
        "Record a synthetic fact".into()
    }
    fn parameters(&self) -> Value {
        serde_json::to_value(schemars::schema_for!(RecordArgs)).unwrap()
    }
    async fn call(&self, args: RecordArgs) -> Result<String, ServiceError> {
        self.0.tools.fetch_add(1, Ordering::SeqCst);
        Ok(format!("Stored {}", args.text))
    }
}

async fn scripted_agent(
    script: Arc<Script>,
) -> (Agent, DynModel<Completion>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let router = Router::new()
        .route("/chat/completions", post(model_reply))
        .with_state(script.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let client = rig::providers::openai::OpenAIConfig::new("local-test-only")
        .with_base_url(format!("http://{address}"))
        .client();
    let model: DynModel<Completion> = client.chat("local-test").into();
    let agent = AgentBuilder::new(model.clone())
        .tool(RecordFact(script))
        .build();
    (agent, model, server)
}

fn config() -> dbos::Config {
    let url = std::env::var("DATABASE_URL").expect("QA database required");
    assert!(
        url.ends_with("/lorekeeper_adversarial"),
        "Durability tests must use QA data"
    );
    let id = format!("test-{}", uuid::Uuid::new_v4());
    let mut config = dbos::Config::new(id.clone(), url);
    config.app_version = Some(id.clone());
    config.executor_id = Some(id);
    config
}

fn register(
    dbos: &dbos::DBOS,
    agent: Agent,
    model: DynModel<Completion>,
) -> dbos::WorkflowRef<String, String, ServiceError> {
    dbos.register_workflow("research", move |input: String| {
        let agent = agent.clone();
        let model = model.clone();
        async move {
            service::ai::researcher::run_durable_agent(&agent, &model, "test researcher", input)
                .await
        }
    })
    .unwrap()
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres; model responses are local and free"]
async fn durable_research_reuses_completed_workflow() {
    let script = Arc::new(Script::default());
    let (agent, model, server) = scripted_agent(script.clone()).await;
    let dbos = dbos::DBOS::new(config());
    let research = register(&dbos, agent, model);
    dbos.launch().await.unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    for _ in 0..2 {
        let handle = research
            .start_with(
                "synthetic input".into(),
                dbos::StartOptions {
                    workflow_id: Some(&id),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let result = tokio::time::timeout(std::time::Duration::from_secs(15), handle.result())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            result, "Research complete",
            "Research cannot finish its model/tool loop"
        );
    }
    dbos.shutdown().await;
    server.abort();
    assert_eq!(
        script.models.load(Ordering::SeqCst),
        2,
        "Duplicate submission repeats completed model calls"
    );
    assert_eq!(
        script.tools.load(Ordering::SeqCst),
        2,
        "Duplicate submission repeats graph-like effects"
    );
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres; model responses are local and free"]
async fn durable_research_restart_skips_recorded_model_and_tool_calls() {
    let script = Arc::new(Script::default());
    script.block_final.store(true, Ordering::SeqCst);
    let (agent, model, server) = scripted_agent(script.clone()).await;
    let config = config();
    let first = dbos::DBOS::new(config.clone());
    let research = register(&first, agent.clone(), model.clone());
    first.launch().await.unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    research
        .start_with(
            "synthetic input".into(),
            dbos::StartOptions {
                workflow_id: Some(&id),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    tokio::time::timeout(
        std::time::Duration::from_secs(15),
        script.final_entered.notified(),
    )
    .await
    .expect("Research never finishes and checkpoints its tool calls");
    first.shutdown().await;
    assert_eq!(
        script.tools.load(Ordering::SeqCst),
        2,
        "The first execution did not run both tool calls"
    );
    script.block_final.store(false, Ordering::SeqCst);
    script.release_final.notify_waiters();
    let second = dbos::DBOS::new(config);
    let research = register(&second, agent, model);
    second.launch().await.unwrap();
    let handle = research
        .start_with(
            "synthetic input".into(),
            dbos::StartOptions {
                workflow_id: Some(&id),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    let result = tokio::time::timeout(std::time::Duration::from_secs(15), handle.result())
        .await
        .unwrap()
        .unwrap();
    second.shutdown().await;
    server.abort();
    assert_eq!(
        result, "Research complete",
        "Recovered research cannot finish"
    );
    assert_eq!(
        script.models.load(Ordering::SeqCst),
        3,
        "Recovery repeats a recorded model request"
    );
    assert_eq!(
        script.tools.load(Ordering::SeqCst),
        2,
        "Recovery repeats recorded tool effects"
    );
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres; model responses are local and free"]
async fn durable_research_unknown_tool_fails_without_execution() {
    let script = Arc::new(Script::default());
    script.unknown_tool.store(true, Ordering::SeqCst);
    let (agent, model, server) = scripted_agent(script.clone()).await;
    let dbos = dbos::DBOS::new(config());
    let research = register(&dbos, agent, model);
    dbos.launch().await.unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        research.run("test".into()),
    )
    .await
    .unwrap();
    dbos.shutdown().await;
    server.abort();
    assert!(
        result.is_err(),
        "Unknown tool names must not leave the loop stuck or execute anything"
    );
    assert_eq!(
        script.tools.load(Ordering::SeqCst),
        0,
        "Unknown calls execute a tool"
    );
}

#[tokio::test]
#[ignore = "requires isolated QA Postgres; model responses are local and free"]
async fn durable_research_invalid_arguments_return_to_model() {
    let script = Arc::new(Script::default());
    script.malformed_args.store(true, Ordering::SeqCst);
    let (agent, model, server) = scripted_agent(script.clone()).await;
    let dbos = dbos::DBOS::new(config());
    let research = register(&dbos, agent, model);
    dbos.launch().await.unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        research.run("test".into()),
    )
    .await
    .unwrap()
    .unwrap();
    dbos.shutdown().await;
    server.abort();
    assert_eq!(
        result, "Research complete",
        "Tool argument errors prevent the model from continuing"
    );
    assert_eq!(
        script.tools.load(Ordering::SeqCst),
        1,
        "Invalid arguments reach the tool body"
    );
}
