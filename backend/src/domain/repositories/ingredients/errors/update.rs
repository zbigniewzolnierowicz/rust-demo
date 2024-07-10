use eyre::eyre;
use std::sync::PoisonError;
use thiserror::Error;

use super::GetIngredientByIdError;
use crate::domain::entities::ingredient::errors::ValidationError;

#[derive(Error, Debug)]
pub enum UpdateIngredientError {
    #[error(transparent)]
    Get(#[from] GetIngredientByIdError),

    #[error("The ingredient with field {0} of the given value already exists")]
    Conflict(String),

    #[error(transparent)]
    ValidationError(#[from] ValidationError),

    #[error(transparent)]
    UnknownError(#[from] eyre::Error),
}

impl<T> From<PoisonError<T>> for UpdateIngredientError {
    fn from(_value: PoisonError<T>) -> Self {
        eyre!("Ingredient repository lock was poisoned during a previous access and can no longer be locked").into()
    }
}
