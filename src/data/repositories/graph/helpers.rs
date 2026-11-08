use crate::data::models::FromRow;
use crate::infrastructure::InfrastructureError;
use neo4rs::{Graph, Query};

pub(super) fn validate_cypher_identifier(value: &str) -> Result<(), InfrastructureError> {
    if value.is_empty()
        || !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        || value.chars().next().unwrap().is_ascii_digit()
    {
        return Err(InfrastructureError::CypherInjection(value.to_string()));
    }
    Ok(())
}

pub(super) async fn fetch_all<T: FromRow>(
    graph: &Graph,
    q: Query,
    alias: &str,
) -> Result<Vec<T>, InfrastructureError> {
    let mut result = graph.execute(q).await?;
    let mut rows = Vec::new();
    while let Some(row) = result.next().await? {
        rows.push(T::from_row(&row, alias)?)
    }
    tracing::debug!(row_count = rows.len(), "graph rows loaded");
    Ok(rows)
}

pub(super) async fn fetch_one<T: FromRow>(
    graph: &Graph,
    q: Query,
    alias: &str,
) -> Result<Option<T>, InfrastructureError> {
    let mut result = graph.execute(q).await?;
    match result.next().await? {
        Some(row) => Ok(Some(T::from_row(&row, alias)?)),
        None => Ok(None),
    }
}
