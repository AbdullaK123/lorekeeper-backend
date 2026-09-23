use thiserror::Error;
use sqlx;
use neo4rs;

#[derive(Error, Debug)]
pub enum InfrastructureError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Memgraph error")]
    Memgraph(#[from] neo4rs::Error),
    #[error("Memgraph deserialization error")]
    MemgraphDeserialize(#[from] neo4rs::DeError),
}