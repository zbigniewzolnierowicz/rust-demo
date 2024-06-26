// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { IngredientWithAmountDTO } from "./IngredientWithAmountDTO";
import type { ServingsTypeDTO } from "./ServingsTypeDTO";

export interface RecipeDTO { id: string, name: string, description: string, steps: Array<string>, time: Record<string, bigint>, ingredients: Array<IngredientWithAmountDTO>, servings: ServingsTypeDTO, }