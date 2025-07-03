use anyhow::Result;
use chrono::{DateTime, Utc};
use sea_query::{ColumnDef, Expr, Iden, Order, Query, SqliteQueryBuilder, Table};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, Row, SqlitePool};
use tauri::State;

use crate::api_keys::{ApiKey, ApiKeys};
use crate::db::Database;
use crate::user::{User, Users};

#[derive(Iden)]
pub enum Projects {
    Table,
    Id,
    Name,
    ShortCode,
    CallbackUrl,
    SimulationMode,
    StkDelay,
    Prefix,
    CreatedAt,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub shortcode: Option<String>,
    pub callback_url: Option<String>,
    pub simulation_mode: String,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: i64, // Unix timestamp
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateProject {
    pub name: String,
    pub shortcode: Option<String>,
    pub callback_url: Option<String>,
    pub simulation_mode: String,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub initial_users: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub shortcode: Option<String>,
    pub callback_url: Option<String>,
    pub simulation_mode: Option<String>,
    pub stk_delay: Option<u32>,
    pub prefix: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ProjectCredentials {
    pub consumer_key: String,
    pub consumer_secret: String,
    pub project_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectDetails {
    pub id: i64,
    pub name: String,
    pub shortcode: Option<String>,
    pub callback_url: Option<String>,
    pub simulation_mode: String,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: DateTime<Utc>,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectSummary {
    pub id: i64,
    pub name: String,
    pub simulation_mode: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub simulation_mode: Option<String>,
    pub shortcode: Option<String>,
    pub has_callback_url: Option<bool>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Project {
    pub async fn init_table(db: &SqlitePool) -> Result<()> {
        let sql = {
            Table::create()
                .table(Projects::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Projects::Id)
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Projects::Name).text().not_null())
                .col(ColumnDef::new(Projects::ShortCode).text())
                .col(ColumnDef::new(Projects::CallbackUrl).text())
                .col(ColumnDef::new(Projects::SimulationMode).text().not_null())
                .col(ColumnDef::new(Projects::StkDelay).integer().not_null())
                .col(ColumnDef::new(Projects::Prefix).text())
                .col(
                    ColumnDef::new(Projects::CreatedAt)
                        .integer()
                        .default(Expr::cust("(strftime('%s', 'now'))")),
                )
                .to_string(SqliteQueryBuilder)
        };
        db.execute(sql.as_str()).await?;
        Ok(())
    }

    // CREATE
    pub async fn create(db: &SqlitePool, data: CreateProject) -> Result<ApiKey> {
        let sql = {
            Query::insert()
                .into_table(Projects::Table)
                .columns([
                    Projects::Name,
                    Projects::ShortCode,
                    Projects::CallbackUrl,
                    Projects::SimulationMode,
                    Projects::StkDelay,
                    Projects::Prefix,
                ])
                .values_panic([
                    data.name.into(),
                    data.shortcode
                        .unwrap_or_else(|| "174379".to_string())
                        .into(),
                    data.callback_url.unwrap_or_default().into(),
                    data.simulation_mode.into(),
                    (data.stk_delay as i64).into(),
                    data.prefix.unwrap_or_default().into(),
                ])
                .to_string(SqliteQueryBuilder)
        };

        let result = sqlx::query(&sql).execute(db).await?;
        let project_id = result.last_insert_rowid();

        // Create API keys for the project
        let api_key = ApiKey::generate(project_id);
        ApiKey::create(db, api_key.clone()).await?;

        // Create initial users if specified
        if let Some(num_users) = data.initial_users {
            if num_users > 0 {
                let users = User::generate_users(num_users);
                for mut user in users {
                    user.project_id = project_id;
                    User::create(db, &user).await?;
                }
            }
        }

        Ok(api_key)
    }

    // READ - Single by ID
    pub async fn find_by_id(db: &SqlitePool, id: i64) -> Result<Option<Project>> {
        let sql = Query::select()
            .columns([
                Projects::Id,
                Projects::Name,
                Projects::ShortCode,
                Projects::CallbackUrl,
                Projects::SimulationMode,
                Projects::StkDelay,
                Projects::Prefix,
                Projects::CreatedAt,
            ])
            .from(Projects::Table)
            .and_where(Expr::col(Projects::Id).eq(id))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&sql).fetch_optional(db).await?;
        match row {
            Some(row) => Ok(Some(Self::from_row(&row)?)),
            None => Ok(None),
        }
    }

    // READ - All with optional filtering
    pub async fn find_all(db: &SqlitePool, filter: ProjectFilter) -> Result<Vec<Project>> {
        let mut query = Query::select();
        query
            .columns([
                Projects::Id,
                Projects::Name,
                Projects::ShortCode,
                Projects::CallbackUrl,
                Projects::SimulationMode,
                Projects::StkDelay,
                Projects::Prefix,
                Projects::CreatedAt,
            ])
            .from(Projects::Table);

        // Apply filters
        if let Some(name) = filter.name {
            query.and_where(Expr::col(Projects::Name).like(format!("%{}%", name)));
        }
        if let Some(simulation_mode) = filter.simulation_mode {
            query.and_where(Expr::col(Projects::SimulationMode).eq(simulation_mode));
        }
        if let Some(shortcode) = filter.shortcode {
            query.and_where(Expr::col(Projects::ShortCode).eq(shortcode));
        }
        if let Some(has_callback_url) = filter.has_callback_url {
            if has_callback_url {
                query.and_where(Expr::col(Projects::CallbackUrl).is_not_null());
                query.and_where(Expr::col(Projects::CallbackUrl).ne(""));
            } else {
                query.and_where(
                    Expr::col(Projects::CallbackUrl)
                        .is_null()
                        .or(Expr::col(Projects::CallbackUrl).eq("")),
                );
            }
        }
        if let Some(created_after) = filter.created_after {
            query.and_where(Expr::col(Projects::CreatedAt).gt(created_after));
        }
        if let Some(created_before) = filter.created_before {
            query.and_where(Expr::col(Projects::CreatedAt).lt(created_before));
        }

        // Order by created_at descending (newest first)
        query.order_by(Projects::CreatedAt, Order::Desc);

        // Apply pagination
        if let Some(limit) = filter.limit {
            query.limit(limit);
        }
        if let Some(offset) = filter.offset {
            query.offset(offset);
        }

        let sql = query.to_string(SqliteQueryBuilder);
        let rows = sqlx::query(&sql).fetch_all(db).await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(Self::from_row(&row)?);
        }

        Ok(results)
    }

    // READ - Project details with API keys
    pub async fn find_details_by_id(db: &SqlitePool, id: i64) -> Result<Option<ProjectDetails>> {
        let project = match Self::find_by_id(db, id).await? {
            Some(project) => project,
            None => return Ok(None),
        };

        let api_key = ApiKey::read_by_project_id(db, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("API key not found for project {}", id))?;

        let created_at =
            DateTime::from_timestamp(project.created_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

        Ok(Some(ProjectDetails {
            id: project.id,
            name: project.name,
            shortcode: project.shortcode,
            callback_url: project.callback_url,
            simulation_mode: project.simulation_mode,
            stk_delay: project.stk_delay,
            prefix: project.prefix,
            created_at,
            consumer_key: api_key.consumer_key,
            consumer_secret: api_key.consumer_secret,
            passkey: api_key.passkey,
        }))
    }

    // UPDATE
    pub async fn update(db: &SqlitePool, id: i64, data: UpdateProject) -> Result<Option<Project>> {
        let mut has_updates = false;
        let sql = {
            let mut query = Query::update();
            query
                .table(Projects::Table)
                .and_where(Expr::col(Projects::Id).eq(id));

            // Only update fields that are provided
            if let Some(name) = data.name {
                query.value(Projects::Name, name);
                has_updates = true;
            }
            if let Some(shortcode) = data.shortcode {
                query.value(Projects::ShortCode, shortcode);
                has_updates = true;
            }
            if let Some(callback_url) = data.callback_url {
                query.value(Projects::CallbackUrl, callback_url);
                has_updates = true;
            }
            if let Some(simulation_mode) = data.simulation_mode {
                query.value(Projects::SimulationMode, simulation_mode);
                has_updates = true;
            }
            if let Some(stk_delay) = data.stk_delay {
                query.value(Projects::StkDelay, stk_delay as i64);
                has_updates = true;
            }
            if let Some(prefix) = data.prefix {
                query.value(Projects::Prefix, prefix);
                has_updates = true;
            }

            query.to_string(SqliteQueryBuilder)
        };
        if !has_updates {
            return Self::find_by_id(db, id).await;
        }

        let result = sqlx::query(sql.as_str()).execute(db).await?;

        if result.rows_affected() > 0 {
            Self::find_by_id(db, id).await
        } else {
            Ok(None)
        }
    }

    // DELETE
    pub async fn delete(db: &SqlitePool, id: i64) -> Result<bool> {
        // Delete in order: API keys, users, then project

        // 1. Delete associated API keys
        let sql_delete_api_keys = {
            Query::delete()
                .from_table(ApiKeys::Table)
                .and_where(Expr::col(ApiKeys::ProjectId).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        db.execute(sql_delete_api_keys.as_str()).await?;

        // 2. Delete associated users
        let sql_delete_users = {
            Query::delete()
                .from_table(Users::Table)
                .and_where(Expr::col(Users::ProjectId).eq(id))
                .to_string(SqliteQueryBuilder)
        };

        db.execute(sql_delete_users.as_str()).await?;

        // 3. Delete the project itself
        let sql_delete_project = Query::delete()
            .from_table(Projects::Table)
            .and_where(Expr::col(Projects::Id).eq(id))
            .to_string(SqliteQueryBuilder);

        let result = sqlx::query(&sql_delete_project).execute(db).await?;
        Ok(result.rows_affected() > 0)
    }

    // UTILITY FUNCTIONS

    // Find projects by name (partial match)
    pub async fn find_by_name(db: &SqlitePool, name: &str) -> Result<Vec<Project>> {
        let filter = ProjectFilter {
            name: Some(name.to_string()),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find projects by simulation mode
    pub async fn find_by_simulation_mode(
        db: &SqlitePool,
        simulation_mode: &str,
    ) -> Result<Vec<Project>> {
        let filter = ProjectFilter {
            simulation_mode: Some(simulation_mode.to_string()),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find projects by shortcode
    pub async fn find_by_shortcode(db: &SqlitePool, shortcode: &str) -> Result<Option<Project>> {
        let sql = Query::select()
            .columns([
                Projects::Id,
                Projects::Name,
                Projects::ShortCode,
                Projects::CallbackUrl,
                Projects::SimulationMode,
                Projects::StkDelay,
                Projects::Prefix,
                Projects::CreatedAt,
            ])
            .from(Projects::Table)
            .and_where(Expr::col(Projects::ShortCode).eq(shortcode))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&sql).fetch_optional(db).await?;
        match row {
            Some(row) => Ok(Some(Self::from_row(&row)?)),
            None => Ok(None),
        }
    }

    // Find projects with callback URLs
    pub async fn find_with_callback_urls(db: &SqlitePool) -> Result<Vec<Project>> {
        let filter = ProjectFilter {
            has_callback_url: Some(true),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find projects without callback URLs
    pub async fn find_without_callback_urls(db: &SqlitePool) -> Result<Vec<Project>> {
        let filter = ProjectFilter {
            has_callback_url: Some(false),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Find recent projects (within last N seconds)
    pub async fn find_recent_projects(db: &SqlitePool, seconds: i64) -> Result<Vec<Project>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as i64;
        let since = now - seconds;

        let filter = ProjectFilter {
            created_after: Some(since),
            ..Default::default()
        };
        Self::find_all(db, filter).await
    }

    // Get project summaries
    pub async fn get_summaries(db: &SqlitePool) -> Result<Vec<ProjectSummary>> {
        let sql = Query::select()
            .columns([
                Projects::Id,
                Projects::Name,
                Projects::SimulationMode,
                Projects::CreatedAt,
            ])
            .from(Projects::Table)
            .order_by(Projects::CreatedAt, Order::Desc)
            .to_string(SqliteQueryBuilder);

        let rows = sqlx::query(&sql).fetch_all(db).await?;
        let mut summaries = Vec::new();

        for row in rows {
            let created_at_timestamp: i64 = row.get(3);
            let created_at =
                DateTime::from_timestamp(created_at_timestamp, 0).unwrap_or(DateTime::UNIX_EPOCH);

            summaries.push(ProjectSummary {
                id: row.get(0),
                name: row.get(1),
                simulation_mode: row.get(2),
                created_at,
            });
        }

        Ok(summaries)
    }

    // Count projects by simulation mode
    pub async fn count_by_simulation_mode(db: &SqlitePool, simulation_mode: &str) -> Result<i64> {
        let sql = Query::select()
            .expr(Expr::col(Projects::Id).count())
            .from(Projects::Table)
            .and_where(Expr::col(Projects::SimulationMode).eq(simulation_mode))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&sql).fetch_one(db).await?;
        Ok(row.get(0))
    }

    // Count total projects
    pub async fn count_total(db: &SqlitePool) -> Result<i64> {
        let sql = Query::select()
            .expr(Expr::col(Projects::Id).count())
            .from(Projects::Table)
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&sql).fetch_one(db).await?;
        Ok(row.get(0))
    }

    // Update project callback URL
    pub async fn update_callback_url(
        db: &SqlitePool,
        id: i64,
        callback_url: &str,
    ) -> Result<Option<Project>> {
        let update_data = UpdateProject {
            callback_url: Some(callback_url.to_string()),
            ..Default::default()
        };
        Self::update(db, id, update_data).await
    }

    // Update project simulation mode
    pub async fn update_simulation_mode(
        db: &SqlitePool,
        id: i64,
        simulation_mode: &str,
    ) -> Result<Option<Project>> {
        let update_data = UpdateProject {
            simulation_mode: Some(simulation_mode.to_string()),
            ..Default::default()
        };
        Self::update(db, id, update_data).await
    }

    // Update STK delay
    pub async fn update_stk_delay(
        db: &SqlitePool,
        id: i64,
        stk_delay: u32,
    ) -> Result<Option<Project>> {
        let update_data = UpdateProject {
            stk_delay: Some(stk_delay),
            ..Default::default()
        };
        Self::update(db, id, update_data).await
    }

    // Check if project exists
    pub async fn exists(db: &SqlitePool, id: i64) -> Result<bool> {
        let sql = Query::select()
            .expr(Expr::col(Projects::Id).count())
            .from(Projects::Table)
            .and_where(Expr::col(Projects::Id).eq(id))
            .to_string(SqliteQueryBuilder);

        let row = sqlx::query(&sql).fetch_one(db).await?;
        let count: i64 = row.get(0);
        Ok(count > 0)
    }

    // Check if shortcode is unique
    pub async fn is_shortcode_unique(
        db: &SqlitePool,
        shortcode: &str,
        exclude_id: Option<i64>,
    ) -> Result<bool> {
        let mut query = Query::select();
        query
            .expr(Expr::col(Projects::Id).count())
            .from(Projects::Table)
            .and_where(Expr::col(Projects::ShortCode).eq(shortcode));

        if let Some(id) = exclude_id {
            query.and_where(Expr::col(Projects::Id).ne(id));
        }

        let sql = query.to_string(SqliteQueryBuilder);
        let row = sqlx::query(&sql).fetch_one(db).await?;
        let count: i64 = row.get(0);
        Ok(count == 0)
    }

    // Helper function to convert sqlx Row to Project
    fn from_row(row: &sqlx::sqlite::SqliteRow) -> Result<Project> {
        Ok(Project {
            id: row.get(Projects::Id.to_string().as_str()),
            name: row.get(Projects::Name.to_string().as_str()),
            shortcode: row.get(Projects::ShortCode.to_string().as_str()),
            callback_url: row.get(Projects::CallbackUrl.to_string().as_str()),
            simulation_mode: row.get(Projects::SimulationMode.to_string().as_str()),
            stk_delay: row.get::<i64, _>(Projects::StkDelay.to_string().as_str()) as u32,
            prefix: row.get(Projects::Prefix.to_string().as_str()),
            created_at: row.get(Projects::CreatedAt.to_string().as_str()),
        })
    }
}

// Additional convenience methods for common operations
impl Project {
    pub async fn create_simple(
        db: &SqlitePool,
        name: String,
        simulation_mode: String,
        stk_delay: u32,
    ) -> Result<ApiKey> {
        let data = CreateProject {
            name,
            shortcode: None,
            callback_url: None,
            simulation_mode,
            stk_delay,
            prefix: None,
            initial_users: None,
        };
        Self::create(db, data).await
    }

    // Create a project with callback URL
    pub async fn create_with_callback(
        db: &SqlitePool,
        name: String,
        callback_url: String,
        simulation_mode: String,
        stk_delay: u32,
    ) -> Result<ApiKey> {
        let data = CreateProject {
            name,
            shortcode: None,
            callback_url: Some(callback_url),
            simulation_mode,
            stk_delay,
            prefix: None,
            initial_users: None,
        };
        Self::create(db, data).await
    }

    // Create a project with initial users
    pub async fn create_with_users(
        db: &SqlitePool,
        name: String,
        simulation_mode: String,
        stk_delay: u32,
        initial_users: u32,
    ) -> Result<ApiKey> {
        let data = CreateProject {
            name,
            shortcode: None,
            callback_url: None,
            simulation_mode,
            stk_delay,
            prefix: None,
            initial_users: Some(initial_users),
        };
        Self::create(db, data).await
    }
}

#[tauri::command]
pub async fn create_project(
    state: State<'_, Database>,
    input: CreateProject,
) -> Result<ApiKey, String> {
    let db = &state.pool;
    Project::create(db, input)
        .await
        .map_err(|err| format!("Failed to create project: {}", err))
}

#[tauri::command]
pub async fn get_project(state: State<'_, Database>, id: i64) -> Result<ProjectDetails, String> {
    let db = &state.pool;

    // Build the SQL SELECT query for the 'projects' table.
    let sql_select_project = Query::select()
        .from(Projects::Table)
        .columns([
            Projects::Id,
            Projects::Name,
            Projects::ShortCode,
            Projects::CallbackUrl,
            Projects::SimulationMode,
            Projects::StkDelay,
            Projects::Prefix,
            Projects::CreatedAt,
        ])
        .and_where(Expr::col(Projects::Id).eq(id))
        .to_string(SqliteQueryBuilder);

    // Fetch the project details.
    let project_row = db
        .fetch_one(sql_select_project.as_str())
        .await
        .map_err(|err| format!("Failed to fetch project with ID {}: {}", id, err))?;

    // Extract project data from the row.
    let project_id: i64 = project_row.get(0);
    let name: String = project_row.get(1);
    let shortcode: Option<String> = project_row.get(2);
    let callback_url: Option<String> = project_row.get(3);
    let simulation_mode: String = project_row.get(4);
    let stk_delay: u32 = project_row.get(5);
    let prefix: Option<String> = project_row.get(6);
    let created_at: i64 = project_row.get(7);

    let created_at: DateTime<Utc> =
        DateTime::from_timestamp(created_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

    let api_key = ApiKey::read_by_project_id(&state.pool, project_id)
        .await
        .map_err(|err| {
            format!(
                "Failed to fetch API keys for project {}: {}",
                project_id, err
            )
        })?
        .ok_or_else(|| format!("Failed to fetch API keys for project {}", project_id))?;

    // Extract API key data from the row.
    let consumer_key: String = api_key.consumer_key;
    let consumer_secret: String = api_key.consumer_secret;
    let passkey: String = api_key.passkey;

    // Return the combined project details.
    Ok(ProjectDetails {
        id: project_id,
        name,
        shortcode,
        callback_url,
        stk_delay,
        created_at,
        simulation_mode,
        prefix,
        consumer_key,
        consumer_secret,
        passkey,
    })
}

#[tauri::command]
pub async fn get_projects(state: State<'_, Database>) -> Result<Vec<ProjectSummary>, String> {
    let db = &state.pool;

    // Build the SQL SELECT query for a summary of projects.
    let sql_select_projects = Query::select()
        .from(Projects::Table)
        .columns([
            Projects::Id,
            Projects::Name,
            Projects::SimulationMode,
            Projects::CreatedAt,
        ])
        .to_string(SqliteQueryBuilder);

    // Fetch all project summary rows.
    let rows = db
        .fetch_all(sql_select_projects.as_str())
        .await
        .map_err(|err| format!("Failed to fetch projects: {}", err))?;

    // Map the rows to `ProjectSummary` structs.
    let projects: Vec<ProjectSummary> = rows
        .iter()
        .map(|row| ProjectSummary {
            id: row.get(0),
            name: row.get(1),
            simulation_mode: row.get(2),
            created_at: row.get(3),
        })
        .collect();

    Ok(projects)
}

#[tauri::command]
pub async fn update_project(
    state: State<'_, Database>,
    id: i64,
    input: UpdateProject,
) -> Result<Option<Project>, String> {
    let db = &state.pool;
    Project::update(db, id, input)
        .await
        .map_err(|err| format!("Failed to update project {}: {}", id, err))
}

#[tauri::command]
pub async fn delete_project(state: State<'_, Database>, id: i64) -> Result<bool, String> {
    let db = &state.pool;
    Project::delete(db, id)
        .await
        .map_err(|e| format!("Failed to delete API keys for project {}: {}", id, e))
}
