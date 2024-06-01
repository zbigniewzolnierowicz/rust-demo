use axum::{extract::State, response::IntoResponse, Json};
use common::{CreateIngredientDTO, IngredientDTO};

use crate::{
    api::{errors::MakeError, AppState},
    domain::commands::ingredients::create::{
        create_ingredient, CreateIngredient, CreateIngredientError,
    },
};

impl MakeError<String> for CreateIngredientError {
    fn get_message(&self) -> String {
        self.to_string()
    }
    fn get_status_code(&self) -> reqwest::StatusCode {
        reqwest::StatusCode::BAD_REQUEST
    }
}

impl IntoResponse for CreateIngredientError {
    fn into_response(self) -> axum::response::Response {
        (self.get_status_code(), self.get_message()).into_response()
    }
}

#[tracing::instrument("[ROUTE] Creating a new ingredient", skip(ingredient_repository))]
pub async fn create_ingredient_route(
    State(AppState {
        ingredient_repository,
        ..
    }): State<AppState>,
    Json(body): Json<CreateIngredientDTO>,
) -> Result<Json<IngredientDTO>, CreateIngredientError> {
    let input = CreateIngredient {
        name: &body.name,
        description: &body.description,
        diet_friendly: body.diet_friendly.unwrap_or_default(),
    };
    let result = create_ingredient(ingredient_repository, &input).await?;

    Ok(Json(result.into()))
}
