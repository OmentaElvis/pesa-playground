use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};
use tauri::State;

use crate::db::Database;

use super::db;
use crate::business::{Business, CreateBusiness, UpdateBusiness};

#[tauri::command]
pub async fn create_business(
    state: State<'_, Database>,
    input: CreateBusiness,
) -> Result<Business, String> {
    let db = &state.conn;
    let create = db::ActiveModel {
        name: Set(input.name),
        short_code: Set(input.short_code),
        ..Default::default()
    };

    let business = &create
        .insert(db)
        .await
        .map_err(|err| format!("Failed to create business: {}", err))?;

    Ok(business.into())
}

#[tauri::command]
pub async fn get_business(state: State<'_, Database>, id: u32) -> Result<Business, String> {
    let db = &state.conn;

    // Fetch the business details.
    let business = db::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch business with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Business with ID {} not found", id))?;

    // Return the combined business details.
    Ok(Business {
        id: business.id,
        name: business.name.clone(),
        short_code: business.short_code.clone(),
    })
}

#[tauri::command]
pub async fn get_businesses(state: State<'_, Database>) -> Result<Vec<Business>, String> {
    let db = &state.conn;

    let businesses = db::Entity::find()
        .all(db)
        .await
        .map_err(|err| format!("Failed to fetch businesses: {}", err))?;

    // Map the rows to `BusinessSummary` structs.
    let businesses: Vec<Business> = businesses
        .iter()
        .map(|business| Business {
            id: business.id,
            name: business.name.clone(),
            short_code: business.short_code.clone(),
        })
        .collect();

    Ok(businesses)
}

#[tauri::command]
pub async fn update_business(
    state: State<'_, Database>,
    id: u32,
    input: UpdateBusiness,
) -> Result<Option<Business>, String> {
    let db = &state.conn;
    let business = db::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch business with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Business with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = business.into();

    if let Some(name) = input.name {
        active_model.name = Set(name.clone());
    }
    if let Some(short_code) = input.short_code {
        active_model.short_code = Set(short_code.clone());
    }

    let updated_business = active_model
        .update(db)
        .await
        .map_err(|err| format!("Failed to update business {}: {}", id, err))?;

    Ok(Some(Business {
        id: updated_business.id,
        name: updated_business.name,
        short_code: updated_business.short_code,
    }))
}

#[tauri::command]
pub async fn delete_business(state: State<'_, Database>, id: u32) -> Result<bool, String> {
    let db = &state.conn;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete business with ID {}: {}", id, e))?;

    Ok(result.rows_affected > 0)
}
