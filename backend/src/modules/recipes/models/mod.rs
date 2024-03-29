use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct RecipeBase {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub user_id: i32,
}

impl RecipeBase {
    pub fn into_dto(self, steps: Vec<Step>, ingredients: Vec<Ingredient>) -> common::RecipeDTO {
        common::RecipeDTO {
            id: self.id,
            name: self.name,
            description: self.description,
            steps: steps.iter().map(|s| s.instructions.clone()).collect(),
            ingredients: ingredients
                .into_iter()
                .map(common::IngredientWithAmount::from)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct Step {
    pub index: i32,
    pub instructions: String,
}

#[derive(Serialize, Deserialize, sqlx::FromRow, Debug)]
pub struct Ingredient {
    pub id: i32,
    pub unit: String,
    pub amount: f64,
    pub name: String,
    pub ingredient_id: i32,
}

impl From<Ingredient> for common::IngredientWithAmount {
    fn from(val: Ingredient) -> Self {
        common::IngredientWithAmount {
            id: val.id,
            name: val.name,
            unit: val.unit,
            amount: val.amount,
        }
    }
}
