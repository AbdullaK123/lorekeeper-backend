use uuid::Uuid;
use crate::data::{CreateWorldRequest, UpdateWorldRequest, WorldRepository, WorldResponse};
use crate::service::ServiceError;

#[derive(Clone)]
pub struct WorldService {
    world_repo: WorldRepository
}


impl WorldService {
    pub fn new(repo: WorldRepository) -> Self {
        Self {
            world_repo: repo
        }
    }

    // create a world
    pub async fn create_world(
        &self,
        user_id: Uuid,
        payload: CreateWorldRequest
    ) -> Result<WorldResponse, ServiceError> {
        let result = self.world_repo.create(
            payload.name,
            payload.description,
            user_id
        ).await?;
        Ok(result.into())
    }

    // update a world
    pub async fn update_world(
        &self,
        id: Uuid,
        user_id: Uuid,
        payload: UpdateWorldRequest
    ) -> Result<Option<WorldResponse>, ServiceError> {
        let result = self.world_repo.update(
            id,
            user_id,
            payload.new_name,
            payload.new_description
        ).await?;
        Ok(result.map(|w| w.into()))
    }

    // delete a world
    pub async fn delete_world(
        &self,
        id: Uuid,
        user_id: Uuid
    ) -> Result<bool, ServiceError> {
        let result = self.world_repo.delete(
            id,
            user_id
        ).await?;
        Ok(result)
    }

    // get a world by id
    pub async fn get_by_id(
        &self,
        id: Uuid,
        user_id: Uuid
    ) -> Result<Option<WorldResponse>, ServiceError> {
        let result = self.world_repo.get_by_id(
            id,
            user_id
        ).await?;
        Ok(result.map(|w| w.into()))
    }

    // get a user's worlds
    pub async fn get(
        &self,
        user_id: Uuid
    ) -> Result<Vec<WorldResponse>, ServiceError> {
        let result = self.world_repo.get(user_id).await?;
        Ok(
            result
                .into_iter()
                .map(|w| w.into())
                .collect()
        )
    }
}