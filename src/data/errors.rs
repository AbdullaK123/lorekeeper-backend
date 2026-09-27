use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataError {
    #[error("Validation error: {0}")]
    ValidationError(String)
}

