use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait, ColumnTrait};
use tauri::State;

use crate::{api_keys::ApiKey, db::Database};

use super::db;
use super::{CreateProject, Project, ProjectDetails, ProjectSummary, UpdateProject};

#[tauri::command]
pub async fn create_project(
    state: State<'_, Database>,
    input: CreateProject,
) -> Result<Project, String> {
    let db = &state.conn;
    let create = db::ActiveModel {
        business_id: Set(input.business_id),
        name: Set(input.name),
        callback_url: Set(input.callback_url),
        prefix: Set(input.prefix),
        simulation_mode: Set(input.simulation_mode),
        stk_delay: Set(input.stk_delay),
        ..Default::default()
    };

    let project = &create
        .insert(db)
        .await
        .map_err(|err| format!("Failed to create project: {}", err))?;

    Ok(project.into())
}

#[tauri::command]
pub async fn get_project(state: State<'_, Database>, id: u32) -> Result<ProjectDetails, String> {
    let db = &state.conn;

    // Fetch the project details.
    let project = db::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch project with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Project with ID {} not found", id))?;

    let api_key = ApiKey::read_by_project_id(db, project.id as u32)
        .await
        .map_err(|err| {
            format!(
                "Failed to fetch API keys for project {}: {}",
                project.id, err
            )
        })?
        .ok_or_else(|| format!("Failed to fetch API keys for project {}", project.id))?;

    // Return the combined project details.
    Ok(ProjectDetails {
        id: project.id,
        name: project.name,
        callback_url: project.callback_url,
        stk_delay: project.stk_delay,
        created_at: project.created_at,
        simulation_mode: project.simulation_mode,
        prefix: project.prefix,
        consumer_key: api_key.consumer_key,
        consumer_secret: api_key.consumer_secret,
        passkey: api_key.passkey,
    })
}

#[tauri::command]
pub async fn get_projects(state: State<'_, Database>) -> Result<Vec<ProjectSummary>, String> {
    let db = &state.conn;

    let projects = db::Entity::find()
        .all(db)
        .await
        .map_err(|err| format!("Failed to fetch projects: {}", err))?;

    // Map the rows to `ProjectSummary` structs.
    let projects: Vec<ProjectSummary> = projects
        .iter()
        .map(|project| ProjectSummary {
            id: project.id,
            name: project.name.clone(),
            simulation_mode: project.simulation_mode.clone(),
            created_at: project.created_at,
        })
        .collect();

    Ok(projects)
}

#[tauri::command]
pub async fn get_projects_by_business_id(
    state: State<'_, Database>,
    business_id: u32,
) -> Result<Vec<ProjectSummary>, String> {
    use sea_orm::QueryFilter;
    use crate::projects::db::Column;

    let db = &state.conn;

    let projects = db::Entity::find()
        .filter(Column::BusinessId.eq(business_id))
        .all(db)
        .await
        .map_err(|err| format!("Failed to fetch projects for business {}: {}", business_id, err))?;

    let projects: Vec<ProjectSummary> = projects
        .iter()
        .map(|project| ProjectSummary {
            id: project.id,
            name: project.name.clone(),
            simulation_mode: project.simulation_mode.clone(),
            created_at: project.created_at,
        })
        .collect();

    Ok(projects)
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, Database>,
    id: u32,
    input: UpdateProject,
) -> Result<Option<Project>, String> {
    let db = &state.conn;
    let project = db::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| format!("Failed to fetch project with ID {}: {}", id, err))?
        .ok_or_else(|| format!("Project with ID {} not found", id))?;

    let mut active_model: db::ActiveModel = project.into();

    if let Some(name) = input.name {
        active_model.name = Set(name);
    }
    if let Some(callback_url) = input.callback_url {
        active_model.callback_url = Set(Some(callback_url));
    }
    if let Some(simulation_mode) = input.simulation_mode {
        active_model.simulation_mode = Set(simulation_mode);
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
        .map_err(|err| format!("Failed to update project {}: {}", id, err))?;

    Ok(Some(Project {
        id: updated_project.id,
        name: updated_project.name,
        callback_url: updated_project.callback_url,
        simulation_mode: updated_project.simulation_mode,
        stk_delay: updated_project.stk_delay,
        prefix: updated_project.prefix,
        created_at: updated_project.created_at,
    }))
}

#[tauri::command]
pub async fn delete_project(state: State<'_, Database>, id: u32) -> Result<bool, String> {
    let db = &state.conn;
    let result = db::Entity::delete_by_id(id)
        .exec(db)
        .await
        .map_err(|e| format!("Failed to delete project with ID {}: {}", id, e))?;

    Ok(result.rows_affected > 0)
}
