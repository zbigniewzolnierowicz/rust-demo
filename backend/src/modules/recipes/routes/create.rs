use crate::modules::recipes::{
    errors::create::RecipeCreateError,
    models::{Ingredient, RecipeBase, Step},
};
use actix_web::{web, HttpResponse};
use eyre::{eyre, Context};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use sqlx::PgPool;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/recipes/create",
    request_body = CreateRecipeDTO,
    responses(
        (status = 201, description = "Recipe was created", body = RecipeDTO),
        (status = 400, description = "Ingredients are missing", body = ErrorMessageWithJsonValue),
        (status = 500, description = "Fatal error", body = ErrorMessageWithJsonValue)
    )
)]
#[instrument(name = "Create recipe", skip(session, db))]
pub async fn create_recipe(
    session: actix_session::Session,
    body: web::Json<common::CreateRecipeDTO>,
    db: web::Data<PgPool>,
) -> Result<HttpResponse, RecipeCreateError> {
    let tx = db.begin().await.context("Could not create transaction.")?;
    let user_id = session
        .get::<i32>("user_id")
        .wrap_err("Failed to get session")?
        .ok_or(eyre!("User ID is missing."))?;

    let ingredient_ids = body.ingredients.iter().cloned().map(|r| r.id).collect();
    check_if_ingredients_exist(&db, ingredient_ids)
        .await?
        .map_err(RecipeCreateError::MissingIngredients)?;

    let base = create_base_recipe(&db, &body, &user_id).await?;
    let steps = insert_steps(&db, &base, &body).await?;
    let ingredients = insert_ingredients_to_recipe(&db, &base, &body).await?;

    let result = base.into_dto(steps, ingredients);

    tx.commit().await.context("Could not commit transaction")?;
    Ok(HttpResponse::Created().json(result))
}

#[instrument(name = "Insert ingredients to recipe" skip(db))]
pub async fn insert_ingredients_to_recipe(
    db: &PgPool,
    recipe: &RecipeBase,
    body: &common::CreateRecipeDTO,
) -> eyre::Result<Vec<Ingredient>> {
    let mut ingredients = vec![];
    for common::CreateRecipeIngredient { id, unit, amount } in body.ingredients.iter() {
        let ingredient = sqlx::query_as!(
            Ingredient,
            r#"WITH iir AS (
                INSERT INTO ingredients_in_recipes
                (recipe_id, ingredient_id, unit, amount)
                VALUES ($1, $2, $3, $4)
                RETURNING ingredient_id, unit, amount
            )
            SELECT id, ingredient_id, name, unit, amount
            FROM ingredients INNER JOIN iir ON ingredients.id = iir.ingredient_id;"#,
            recipe.id,
            id,
            unit,
            amount,
        )
        .fetch_one(db)
        .await?;

        ingredients.push(ingredient);
    }

    Ok(ingredients)
}

#[instrument(name = "Create base recipe" skip(db))]
pub async fn create_base_recipe(
    db: &PgPool,
    body: &common::CreateRecipeDTO,
    user_id: &i32,
) -> eyre::Result<RecipeBase> {
    sqlx::query_as!(
        RecipeBase,
        "INSERT INTO recipes (name, description, user_id) VALUES ($1, $2, $3) RETURNING id, name, description, user_id",
        body.name, body.description, user_id
    )
        .fetch_one(db)
        .await
        .context("Could not insert base recipe")
}

#[instrument(name = "Insert steps")]
pub async fn insert_steps(
    db: &PgPool,
    recipe: &RecipeBase,
    body: &common::CreateRecipeDTO,
) -> eyre::Result<Vec<Step>> {
    let mut steps: Vec<Step> = vec![];
    for (index, step) in body.steps.iter().enumerate() {
        let index: i32 = index as i32;
        let step = sqlx::query_as!(
            Step,
            r#"INSERT INTO steps (index, recipe_id, instructions)
               VALUES ($1, $2, $3)
               RETURNING index, instructions"#,
            index,
            recipe.id,
            step
        )
        .fetch_one(db)
        .await?;

        steps.push(step);
    }

    Ok(steps)
}

#[instrument(name = "Check if ingredients exist", skip(db))]
pub async fn check_if_ingredients_exist(
    db: &PgPool,
    ingredient_ids: Vec<i32>,
) -> eyre::Result<Result<(), Vec<i32>>> {
    let ingredient_ids_that_exist: Vec<i32> = sqlx::query!(
        "SELECT id FROM ingredients WHERE id IN (SELECT unnest($1::integer[]))",
        &ingredient_ids
    )
    .fetch_all(db)
    .await?
    .iter()
    .map(|i| i.id)
    .collect();

    let ingredient_ids_that_dont_exist: Vec<i32> = ingredient_ids
        .into_par_iter()
        .filter(|x| !ingredient_ids_that_exist.contains(x))
        .collect();

    if ingredient_ids_that_dont_exist.is_empty() {
        Ok(Ok(()))
    } else {
        Ok(Err(ingredient_ids_that_dont_exist))
    }
}
