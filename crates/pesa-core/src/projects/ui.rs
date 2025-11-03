use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, JoinType, QuerySelect, RelationTrait,
    TransactionTrait,
};

use crate::api_keys::ApiKey;
use crate::{api_keys, AppContext};

use super::db;
use super::{CreateProject, Project, ProjectDetails, ProjectSummary, UpdateProject};

pub async fn create_project(ctx: &AppContext, input: CreateProject) -> Result<ProjectDetails> {
    let txn = ctx
        .db
        .begin()
        .await
        .context("Failed to start transaction")?;

    let create = db::ActiveModel {
        business_id: Set(input.business_id),
        name: Set(input.name),
        callback_url: Set(input.callback_url),
        prefix: Set(input.prefix),
        simulation_mode: Set(input.simulation_mode.to_string()),
        stk_delay: Set(input.stk_delay),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let project = &create
        .insert(&txn)
        .await
        .context("Failed to create project")?;

    let key = ApiKey::generate(project.id);
    let create_apikey = api_keys::db::ActiveModel {
        project_id: Set(key.project_id),
        consumer_key: Set(key.consumer_key),
        consumer_secret: Set(key.consumer_secret),
        passkey: Set(key.passkey),
        created_at: Set(Utc::now().to_utc()),
        ..Default::default()
    };

    let key = create_apikey
        .insert(&txn)
        .await
        .context("Failed to create api keys")?;

    txn.commit()
        .await
        .context("Failed to commit db transaction")?;

    Ok(ProjectDetails {
        id: project.id,
        name: project.name.clone(),
        callback_url: project.callback_url.clone(),
        simulation_mode: input.simulation_mode,
        stk_delay: project.stk_delay,
        prefix: project.prefix.clone(),
        created_at: project.created_at,
        consumer_key: key.consumer_key,
        consumer_secret: key.consumer_secret,
        passkey: key.passkey,
        business_id: project.business_id,
    })
}

pub async fn get_project(ctx: &AppContext, id: u32) -> Result<ProjectDetails> {
    let db = &ctx.db;

    // Fetch the project details.
    let project = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch project with ID {}", id))?
        .ok_or_else(|| anyhow!("Project with ID {} not found", id))?;

    let api_key = ApiKey::read_by_project_id(db, project.id as u32)
        .await
        .context(format!(
            "Failed to fetch API keys for project {}",
            project.id
        ))?
        .ok_or_else(|| anyhow!("Failed to fetch API keys for project {}", project.id))?;

    // Return the combined project details.
    Ok(ProjectDetails {
        id: project.id,
        name: project.name,
        callback_url: project.callback_url,
        stk_delay: project.stk_delay,
        created_at: project.created_at,
        simulation_mode: project
            .simulation_mode
            .parse()
            .unwrap_or(super::SimulationMode::Realistic),
        prefix: project.prefix,
        consumer_key: api_key.consumer_key,
        consumer_secret: api_key.consumer_secret,
        passkey: api_key.passkey,
        business_id: project.business_id,
    })
}

pub async fn get_projects(ctx: &AppContext) -> Result<Vec<ProjectSummary>> {
    let db = &ctx.db;

    let projects = db::Entity::find()
        .join(JoinType::InnerJoin, db::Relation::Business.def())
        .select_only()
        .column(db::Column::Id)
        .column(db::Column::Name)
        .column(db::Column::SimulationMode)
        .column(db::Column::CreatedAt)
        .column(db::Column::BusinessId)
        .column(db::Column::StkDelay)
        .column(crate::business::db::Column::ShortCode)
        .column_as(crate::business::db::Column::Name, "business_name")
        .into_model::<ProjectSummary>()
        .all(db)
        .await
        .context("Failed to fetch projects")?;

    Ok(projects)
}

pub async fn get_projects_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<ProjectSummary>> {
    use crate::projects::db::Column;
    use sea_orm::QueryFilter;

    let db = &ctx.db;

    let projects = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .join(JoinType::InnerJoin, db::Relation::Business.def())
        .select_only()
        .column(db::Column::Id)
        .column(db::Column::Name)
        .column(db::Column::SimulationMode)
        .column(db::Column::CreatedAt)
        .column(db::Column::BusinessId)
        .column_as(crate::business::db::Column::Name, "business_name")
        .column(crate::business::db::Column::ShortCode)
        .into_model::<ProjectSummary>()
        .all(db)
        .await
        .context(format!(
            "Failed to fetch projects for business {}",
            business_id
        ))?;

    Ok(projects)
}

pub async fn update_project(
    ctx: &AppContext,
    id: u32,
    input: UpdateProject,
) -> Result<Option<Project>> {
    let db = &ctx.db;
    let project = db::Entity::find_by_id(id)
        .one(db)
        .await
        .context(format!("Failed to fetch project with ID {}", id))?
        .ok_or_else(|| anyhow!("Project with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = project.into();

    if let Some(name) = input.name {
        active_model.name = Set(name);
    }
    if let Some(callback_url) = input.callback_url {
        active_model.callback_url = Set(Some(callback_url));
    }
    if let Some(simulation_mode) = input.simulation_mode {
        active_model.simulation_mode = Set(simulation_mode.to_string());
    }
    if let Some(stk_delay) = input.stk_delay {
        active_model.stk_delay = Set(stk_delay);
    }
    if let Some(prefix) = input.prefix {
        active_model.prefix = Set(Some(prefix));
    }

    let updated_project = active_model
        .update(db)
        .await
        .context(format!("Failed to update project {}", id))?;

    Ok(Some(Project {
        id: updated_project.id,
        name: updated_project.name,
        callback_url: updated_project.callback_url,
        simulation_mode: updated_project
            .simulation_mode
            .parse()
            .unwrap_or(super::SimulationMode::Realistic),
        stk_delay: updated_project.stk_delay,
        prefix: updated_project.prefix,
        created_at: updated_project.created_at,
    }))
}

pub async fn delete_project(ctx: &AppContext, id: u32) -> Result<bool> {
    let db = &ctx.db;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context(format!("Failed to delete project with ID {}", id))?;

    Ok(result.rows_affected > 0)
}
