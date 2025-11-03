use anyhow::{anyhow, Context, Result};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};

use crate::AppContext;

use super::db;
use crate::business::{Business, CreateBusiness, UpdateBusiness};

pub async fn create_business(ctx: &AppContext, input: CreateBusiness) -> Result<Business> {
    let db = &ctx.db;
    let create = db::ActiveModel {
        name: Set(input.name),
        short_code: Set(input.short_code),
        ..Default::default()
    };

    let business = &create
        .insert(db)
        .await
        .context("Failed to create business")?;

    Ok(business.into())
}

pub async fn get_business(ctx: &AppContext, id: u32) -> Result<Business> {
    let db = &ctx.db;

    // Fetch the business details.
    let business = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch business with ID {}", id))?
        .ok_or_else(|| anyhow!("Business with ID {} not found", id))?;

    // Return the combined business details.
    Ok(Business {
        id: business.id,
        name: business.name.clone(),
        short_code: business.short_code.clone(),
    })
}

pub async fn get_businesses(ctx: &AppContext) -> Result<Vec<Business>> {
    let db = &ctx.db;

    let businesses = db::Entity::find()
        .all(db)
        .await
        .context("Failed to fetch businesses")?;

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

pub async fn update_business(
    ctx: &AppContext,
    id: u32,
    input: UpdateBusiness,
) -> Result<Option<Business>> {
    let db = &ctx.db;
    let business = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch business with ID {}", id))?
        .ok_or_else(|| anyhow!("Business with ID {} not found", id))?;

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
        .context(format!("Failed to update business {}", id))?;

    Ok(Some(Business {
        id: updated_business.id,
        name: updated_business.name,
        short_code: updated_business.short_code,
    }))
}

pub async fn delete_business(ctx: &AppContext, id: u32) -> Result<bool> {
    let db = &ctx.db;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context(format!("Failed to delete business with ID {}", id))?;

    Ok(result.rows_affected > 0)
}
