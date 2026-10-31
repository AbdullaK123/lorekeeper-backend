use crate::app::errors::AppError;
use axum::Json;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

pub struct ValidatedQuery<T>(pub T);

impl<S, T> axum::extract::FromRequestParts<S> for ValidatedQuery<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Query(query) =
            axum::extract::Query::<T>::from_request_parts(parts, state)
                .await
                .map_err(|error| AppError::ValidationError(error.to_string()))?;
        query.validate()?;
        Ok(Self(query))
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(body) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::ValidationError(e.to_string()))?;
        body.validate()?;
        Ok(ValidatedJson(body))
    }
}
