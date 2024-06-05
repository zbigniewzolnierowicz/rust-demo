use async_trait::async_trait;
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;

use crate::domain::{entities::recipe::Recipe, repositories::recipe::errors::InsertRecipeError};

use super::{
    errors::{DeleteRecipeError, GetRecipeByIdError},
    RecipeRepository,
};

pub struct InMemoryRecipeRepository(pub Mutex<HashMap<uuid::Uuid, Recipe>>);

#[async_trait]
impl RecipeRepository for InMemoryRecipeRepository {
    async fn insert(&self, input: Recipe) -> Result<Recipe, InsertRecipeError> {
        let mut lock = self.0.lock()?;

        if lock.iter().any(|(id, _)| id == &input.id) {
            tracing::error!("The recipe with ID {} already exists.", input.id);
            return Err(InsertRecipeError::Conflict("recipe id".to_string()));
        };

        lock.insert(input.id, input.clone());

        Ok(input)
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<Recipe, GetRecipeByIdError> {
        let lock = self.0.lock()?;

        let result = lock
            .get(id)
            .cloned()
            .ok_or_else(|| GetRecipeByIdError::NotFound(*id))?;

        Ok(result)
    }

    async fn delete(&self, id: &Uuid) -> Result<(), DeleteRecipeError> {
        let mut lock = self.0.lock()?;

        lock.remove(id).ok_or(DeleteRecipeError::NotFound(*id))?;

        Ok(())
    }
}

impl Default for InMemoryRecipeRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryRecipeRepository {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

impl From<HashMap<uuid::Uuid, Recipe>> for InMemoryRecipeRepository {
    fn from(value: HashMap<uuid::Uuid, Recipe>) -> Self {
        Self(Mutex::new(value))
    }
}

#[cfg(test)]
mod tests;
