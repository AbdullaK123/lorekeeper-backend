use crate::data::{
    ConflictListQuery, ConflictResponse, CreateWorldRequest, EntityListQuery, EntityResponse,
    EntitySearchQuery, GraphQuery, GraphRepository, GraphResponse, NeighborQuery,
    RelationshipResponse, UpdateWorldRequest, WorldRepository, WorldResponse,
};
use crate::service::ServiceError;
use crate::shared::PageResponse;
use uuid::Uuid;

#[derive(Clone)]
pub struct WorldService {
    world_repo: WorldRepository,
    graph_repo: GraphRepository,
}

impl WorldService {
    pub fn new(repo: WorldRepository, graph_repo: GraphRepository) -> Self {
        Self {
            world_repo: repo,
            graph_repo,
        }
    }

    // create a world
    pub async fn create_world(
        &self,
        user_id: Uuid,
        payload: CreateWorldRequest,
    ) -> Result<WorldResponse, ServiceError> {
        let result = self
            .world_repo
            .create(
                payload.name.into_inner(),
                payload.description.into_inner(),
                user_id,
            )
            .await?;
        Ok(result.into())
    }

    // update a world
    pub async fn update_world(
        &self,
        id: Uuid,
        user_id: Uuid,
        payload: UpdateWorldRequest,
    ) -> Result<WorldResponse, ServiceError> {
        let result = self
            .world_repo
            .update(
                id,
                user_id,
                payload.new_name.map(String::from),
                payload.new_description.map(String::from),
            )
            .await?;
        match result {
            Some(world) => Ok(world.into()),
            None => Err(ServiceError::NotFound("World not found".to_string())),
        }
    }

    // delete a world
    pub async fn delete_world(&self, id: Uuid, user_id: Uuid) -> Result<bool, ServiceError> {
        let result = self.world_repo.delete(id, user_id).await?;
        Ok(result)
    }

    // get a world by id
    pub async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> Result<WorldResponse, ServiceError> {
        let result = self.world_repo.get_by_id(id, user_id).await?;
        match result {
            Some(world) => Ok(world.into()),
            None => Err(ServiceError::NotFound("World not found".to_string())),
        }
    }

    // get a user's worlds
    pub async fn get(&self, user_id: Uuid) -> Result<Vec<WorldResponse>, ServiceError> {
        let result = self.world_repo.get(user_id).await?;
        Ok(result.into_iter().map(|w| w.into()).collect())
    }

    /// Check world ownership, then load a bounded set of nodes and edges.
    pub async fn get_graph(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        query: &GraphQuery,
    ) -> Result<PageResponse<GraphResponse>, ServiceError> {
        let world = self.world_repo.get_by_id(world_id, user_id).await?;
        if world.is_none() {
            return Err(ServiceError::NotFound("World not found".to_string()));
        }
        let result = self
            .graph_repo
            .get_graph(&world_id.to_string(), query.limit, query.cursor.as_deref())
            .await?;
        Ok(PageResponse {
            data: (result.0, result.1).into(),
            next_cursor: result.2,
        })
    }

    /// Check world ownership, then page entities using the requested filters.
    pub async fn list_entities(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        query: &EntityListQuery,
    ) -> Result<PageResponse<Vec<EntityResponse>>, ServiceError> {
        let world = self.world_repo.get_by_id(world_id, user_id).await?;
        if world.is_none() {
            return Err(ServiceError::NotFound("World not found".to_string()));
        }
        let result = self
            .graph_repo
            .list_entities(
                &world_id.to_string(),
                query.limit,
                query.cursor.as_deref(),
                query.kind.as_deref(),
            )
            .await?;
        Ok(result.into())
    }

    /// Check world ownership, then search saved entities.
    pub async fn search_entities(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        query: &EntitySearchQuery,
    ) -> Result<PageResponse<Vec<EntityResponse>>, ServiceError> {
        let world = self.world_repo.get_by_id(world_id, user_id).await?;
        if world.is_none() {
            return Err(ServiceError::NotFound("World not found".to_string()));
        }
        let result = self
            .graph_repo
            .fts_search_entities(&world_id.to_string(), &query.query.as_str(), query.limit)
            .await?;
        Ok((result, None).into())
    }

    /// Check world ownership, then read an entity within that world.
    pub async fn get_entity(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        entity_id: Uuid,
    ) -> Result<EntityResponse, ServiceError> {
        let world = self.world_repo.get_by_id(world_id, user_id).await?;
        if world.is_none() {
            return Err(ServiceError::NotFound("World not found".to_string()));
        }
        let result = self
            .graph_repo
            .get_entity(&entity_id.to_string(), &world_id.to_string())
            .await?;
        if result.is_none() {
            return Err(ServiceError::NotFound("Entity not found".to_string()));
        }
        Ok(result.unwrap().into())
    }

    /// Check world ownership, then page the entity's neighbors and connecting edges.
    pub async fn get_entity_neighbors(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        entity_id: Uuid,
        query: &NeighborQuery,
    ) -> Result<PageResponse<GraphResponse>, ServiceError> {
        let world = self.world_repo.get_by_id(world_id, user_id).await?;
        if world.is_none() {
            return Err(ServiceError::NotFound("World not found".to_string()));
        }
        let entity = self
            .graph_repo
            .get_entity(&entity_id.to_string(), &world_id.to_string())
            .await?;
        if entity.is_none() {
            return Err(ServiceError::NotFound("Entity not found".to_string()));
        }
        let entity = entity.unwrap();
        let neighbours = self
            .graph_repo
            .get_entity_neighbors_page(
                &entity.id,
                &world_id.to_string(),
                query.rel_type.as_deref(),
                query.kind.as_deref(),
                query.limit,
                query.cursor.as_deref(),
            )
            .await?;
        Ok(neighbours.into())
    }

    /// Check world ownership, then read a relationship within that world.
    pub async fn get_relationship(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        relationship_id: Uuid,
    ) -> Result<RelationshipResponse, ServiceError> {
        self.get_by_id(world_id, user_id).await?;
        let relationship = self
            .graph_repo
            .get_relationship(&relationship_id.to_string(), &world_id.to_string())
            .await?
            .ok_or_else(|| ServiceError::NotFound("Relationship not found".into()))?;
        Ok(relationship.into())
    }

    /// Check world ownership, then page conflicts using the resolution filter.
    pub async fn list_conflicts(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        query: &ConflictListQuery,
    ) -> Result<PageResponse<Vec<ConflictResponse>>, ServiceError> {
        self.get_by_id(world_id, user_id).await?;
        let result = self
            .graph_repo
            .list_conflicts(
                &world_id.to_string(),
                query.limit,
                query.cursor.as_deref(),
                query.resolved,
            )
            .await?;
        Ok(result.into())
    }

    /// Check world ownership, then read both claims and their sources.
    pub async fn get_conflict(
        &self,
        world_id: Uuid,
        user_id: Uuid,
        conflict_id: Uuid,
    ) -> Result<ConflictResponse, ServiceError> {
        self.get_by_id(world_id, user_id).await?;
        let conflict = self
            .graph_repo
            .get_conflict(&conflict_id.to_string(), &world_id.to_string())
            .await?
            .ok_or_else(|| ServiceError::NotFound("Conflict not found".into()))?;
        Ok(conflict.into())
    }
}
