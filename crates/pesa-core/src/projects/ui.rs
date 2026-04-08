use anyhow::Result;

use crate::AppContext;

use super::{CreateProject, Project, ProjectDetails, ProjectSummary, UpdateProject};

pub async fn create_project(ctx: &AppContext, input: CreateProject) -> Result<ProjectDetails> {
    Project::create(&ctx.db, input).await
}

pub async fn get_project(ctx: &AppContext, id: u32) -> Result<ProjectDetails> {
    Project::get_details(&ctx.db, id).await
}

pub async fn get_projects(ctx: &AppContext) -> Result<Vec<ProjectSummary>> {
    Project::get_all(&ctx.db).await
}

pub async fn get_projects_by_business_id(
    ctx: &AppContext,
    business_id: u32,
) -> Result<Vec<ProjectSummary>> {
    Project::get_by_business_id(&ctx.db, business_id).await
}

pub async fn update_project(
    ctx: &AppContext,
    id: u32,
    input: UpdateProject,
) -> Result<Option<Project>> {
    Project::update(&ctx.db, id, input).await
}

pub async fn delete_project(ctx: &AppContext, id: u32) -> Result<bool> {
    Project::delete(&ctx.db, id).await
}
