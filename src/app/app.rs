use crate::app::{
    create_auth_controller, create_conversation_controller, create_world_controller, setup_dbos,
};
use crate::cron_worker::start_scheduler;
use crate::data::{ConversationRepository, GraphRepository, UserRepository, WorldRepository};
use crate::infrastructure::{create_graph, create_pool, load_config, load_prompt, load_settings};
use crate::service::{
    AuthService, ChatAgentFactory, ConversationService, ResearcherService, RetrievalService,
    WorldService,
};
use axum::Router;
use axum::routing::get;
use dbos::DBOS;
use neo4rs::Graph;
use rig::DynModel;
use rig::operation::Embedding;
use rig::providers::openrouter;
use sqlx::postgres::PgPool;
use tokio_cron_scheduler::JobScheduler;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    graph: Graph,
    pub auth_service: AuthService,
    pub world_service: WorldService,
    pub conversation_service: ConversationService,
}

pub async fn create_app() -> (Router, JobScheduler, DBOS) {
    let settings = load_settings();
    let config = load_config();
    let pool = create_pool(&settings, &config.db).await;
    let graph = create_graph(&settings, &config.memgraph).await;
    let graph_repo = GraphRepository::new(graph.clone());
    let user_repo = UserRepository::new(pool.clone());
    let world_repo = WorldRepository::new(pool.clone());
    let conversation_repo = ConversationRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repo);
    let world_service = WorldService::new(world_repo.clone());

    graph_repo
        .ensure_constraints()
        .await
        .expect("Failed to ensure constraints");

    let client = openrouter::from_env().expect("OPENROUTER_API_KEY not set");

    let model = client.chat("openai/gpt-6-luna");
    let embedding_model: DynModel<Embedding> = client
        .embedding("openai/text-embedding-3-small", None)
        .into();

    let retrieval_service = RetrievalService::new(graph_repo.clone(), embedding_model.clone());

    let agent_factory = ChatAgentFactory::new(
        model.clone().into(),
        load_prompt("agents").expect("Agent prompt not found"),
        retrieval_service.clone(),
        graph_repo.clone(),
    );
    let researcher_service = ResearcherService::new(
        model.into(),
        load_prompt("researcher").expect("Researcher prompt not found"),
        graph_repo.clone(),
        retrieval_service.clone(),
        embedding_model,
    );

    let (dbos, research) = setup_dbos(&settings, researcher_service)
        .await
        .expect("Failed to initialize dbos");

    let conversation_service = ConversationService::new(
        conversation_repo,
        world_repo,
        retrieval_service,
        agent_factory,
        research,
    );

    let app_state = AppState {
        pool: pool.clone(),
        graph,
        auth_service,
        world_service,
        conversation_service,
    };

    let scheduler = start_scheduler(pool.clone(), &graph_repo)
        .await
        .expect("Failed to start the world cleanup scheduler");

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .nest("/auth", create_auth_controller())
        .nest("/world", create_world_controller())
        .nest("/conversation", create_conversation_controller())
        .layer(settings.trace_layer())
        .layer(settings.session_layer(pool.clone()).await)
        .layer(settings.cors_layer())
        .with_state(app_state);

    (app, scheduler, dbos)
}
