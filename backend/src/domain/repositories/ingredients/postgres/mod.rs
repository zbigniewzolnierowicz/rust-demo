use std::{collections::HashMap, sync::OnceLock};

use crate::domain::entities::ingredient::{
    errors::ValidationError, Ingredient, IngredientChangeset, IngredientModel,
};
use async_trait::async_trait;
use rayon::iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator};
use sqlx::{error::Error as SQLXError, PgPool};
use uuid::Uuid;

use super::{
    errors::{
        DeleteIngredientError, GetAllIngredientsError, GetIngredientByIdError,
        InsertIngredientError, UpdateIngredientError,
    },
    IngredientRepository,
};

pub struct PostgresIngredientRepository(pub PgPool);

/// Turns out Postgres doesn't return the column name for unique constraints isn't returned.
/// This function maps constraints to fields
fn constraint_to_field(field: &str) -> &str {
    static HASHMAP: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    let m = HASHMAP.get_or_init(|| {
        HashMap::from_iter([("ingredients_name_key", "name"), ("ingredients_pkey", "id")])
    });
    m.get(field).unwrap_or(&field)
}

#[async_trait]
impl IngredientRepository for PostgresIngredientRepository {
    #[tracing::instrument(
        "[INGREDIENT REPOSITORY] [POSTGRES] Insert a new ingredient",
        skip(self)
    )]
    async fn insert(&self, ingredient: Ingredient) -> Result<Ingredient, InsertIngredientError> {
        let diet_friendly: Vec<String> = ingredient
            .clone()
            .diet_friendly
            .0
            .into_iter()
            .map(|d| d.to_string())
            .collect();

        let ingredient = sqlx::query_file_as!(
            IngredientModel,
            "queries/ingredients/insert_ingredient.sql",
            ingredient.id,
            &ingredient.name,
            &ingredient.description,
            &diet_friendly
        )
        .fetch_one(&self.0)
        .await
        .map_err(|e| match e {
            SQLXError::Database(dberror) if dberror.is_unique_violation() => {
                InsertIngredientError::Conflict(
                    constraint_to_field(dberror.constraint().unwrap_or_default()).to_string(),
                )
            }
            _ => InsertIngredientError::UnknownError(e.into()),
        })?;

        Ok(ingredient.try_into()?)
    }

    #[tracing::instrument(
        "[INGREDIENT REPOSITORY] [POSTGRES] Get ingredient with ID",
        skip(self)
    )]
    async fn get_by_id(&self, id: &Uuid) -> Result<Ingredient, GetIngredientByIdError> {
        let ingredient = sqlx::query_file_as!(
            IngredientModel,
            "queries/ingredients/get_ingredient_by_id.sql",
            id
        )
        .fetch_one(&self.0)
        .await
        .map_err(|e| match e {
            SQLXError::RowNotFound => GetIngredientByIdError::NotFound(*id),
            _ => GetIngredientByIdError::UnknownError(e.into()),
        })?;

        Ok(ingredient.try_into()?)
    }

    #[tracing::instrument("[INGREDIENT REPOSITORY] [POSTGRES] Get all ingredients", skip(self))]
    async fn get_all(&self) -> Result<Vec<Ingredient>, GetAllIngredientsError> {
        let ingredients = sqlx::query_file_as!(
            IngredientModel,
            "queries/ingredients/get_all_ingredients.sql",
        )
        .fetch_all(&self.0)
        .await
        .map_err(|e| GetAllIngredientsError::UnknownError(e.into()))?
        .par_iter()
        .filter_map(|i| i.try_into().ok())
        .collect();

        Ok(ingredients)
    }

    #[tracing::instrument("[INGREDIENT REPOSITORY] [POSTGRES] Update ingredient", skip(self))]
    async fn update(
        &self,
        id: Uuid,
        changeset: IngredientChangeset,
    ) -> Result<Ingredient, UpdateIngredientError> {
        let mut ingredient_to_update: IngredientModel = self.get_by_id(&id).await?.into();

        let name: Option<String> = changeset.name.map(|n| n.to_string());
        let description: Option<String> = changeset.description.map(|n| n.to_string());
        let diet_friendly: Option<Vec<String>> = changeset.diet_friendly.map(|df| df.into());

        if name.is_none() && description.is_none() && diet_friendly.is_none() {
            return Err(UpdateIngredientError::ValidationError(
                ValidationError::EmptyField(vec!["name", "description", "diet_friendly"]),
            ));
        };

        let tx = self
            .0
            .begin()
            .await
            .map_err(|e| UpdateIngredientError::UnknownError(e.into()))?;

        if let Some(name) = name {
            if name != *ingredient_to_update.name {
                ingredient_to_update = sqlx::query_as!(
                    IngredientModel,
                    r#"
                    UPDATE ingredients
                    SET
                    name = $2
                    WHERE id = $1
                    RETURNING id, name, description, diet_friendly
                "#,
                    id,
                    name,
                )
                .fetch_one(&self.0)
                .await
                .map_err(|e| UpdateIngredientError::UnknownError(e.into()))?;
            };
        };

        if let Some(description) = description {
            if description != ingredient_to_update.description {
                ingredient_to_update = sqlx::query_as!(
                    IngredientModel,
                    r#"
                    UPDATE ingredients
                    SET
                    description = $2
                    WHERE id = $1
                    RETURNING id, name, description, diet_friendly
                    "#,
                    id,
                    description,
                )
                .fetch_one(&self.0)
                .await
                .map_err(|e| UpdateIngredientError::UnknownError(e.into()))?;
            }
        };

        if let Some(diet_friendly) = diet_friendly {
            if diet_friendly != ingredient_to_update.diet_friendly {
                ingredient_to_update = sqlx::query_as!(
                    IngredientModel,
                    r#"
                    UPDATE ingredients
                    SET
                    diet_friendly = $2
                    WHERE id = $1
                    RETURNING id, name, description, diet_friendly
                    "#,
                    id,
                    &diet_friendly
                )
                .fetch_one(&self.0)
                .await
                .map_err(|e| UpdateIngredientError::UnknownError(e.into()))?;
            }
        };

        tx.commit()
            .await
            .map_err(|e| UpdateIngredientError::UnknownError(e.into()))?;

        Ok(ingredient_to_update.try_into()?)
    }

    #[tracing::instrument("[INGREDIENT REPOSITORY] [POSTGRES] Delete an ingredient", skip(self))]
    async fn delete(&self, id: Uuid) -> Result<(), DeleteIngredientError> {
        let ingredient_to_delete = self.get_by_id(&id).await?;

        sqlx::query_file!(
            "queries/ingredients/delete_ingredient.sql",
            ingredient_to_delete.id
        )
        .execute(&self.0)
        .await
        .map_err(|e| DeleteIngredientError::UnknownError(e.into()))?;

        Ok(())
    }

    async fn get_all_by_id(&self, ids: &[Uuid]) -> Result<Vec<Ingredient>, GetAllIngredientsError> {
        let results: Result<Vec<Ingredient>, GetAllIngredientsError> = sqlx::query_file_as!(
            IngredientModel,
            "queries/ingredients/get_all_ingredients_by_id.sql",
            ids
        )
        .fetch_all(&self.0)
        .await
        .map_err(|e| match e {
            SQLXError::RowNotFound => {
                GetAllIngredientsError::MultipleIngredientsMissing(ids.to_vec())
            }
            e => GetAllIngredientsError::UnknownError(e.into()),
        })?
        .into_par_iter()
        .map(|ingredient| ingredient.try_into().map_err(GetAllIngredientsError::from))
        .collect();

        let results = results?;

        let results_ids: Vec<Uuid> = results.iter().map(|i| i.id).collect();

        let omitted_ids: Vec<Uuid> = ids
            .to_vec()
            .iter()
            .filter(|id| !results_ids.contains(id))
            .cloned()
            .collect();

        if omitted_ids.is_empty() {
            Ok(results)
        } else {
            Err(GetAllIngredientsError::MultipleIngredientsMissing(
                omitted_ids,
            ))
        }
    }
}

impl PostgresIngredientRepository {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }
}

#[cfg(test)]
mod tests;
