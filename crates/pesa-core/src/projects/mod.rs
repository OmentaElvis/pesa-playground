use chrono::{DateTime, Utc};

use sea_orm::prelude::DateTimeUtc;
use sea_orm::{ColumnTrait, ConnectionTrait, DbErr, EntityTrait, FromQueryResult, QueryFilter};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub mod db;
pub mod ui;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: u32,
    pub name: String,
    pub callback_url: Option<String>,
    pub simulation_mode: SimulationMode,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: DateTimeUtc,
}

#[derive(Display, EnumString, Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum SimulationMode {
    AlwaysSuccess,
    AlwaysFail,
    Random,
    Realistic,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateProject {
    pub business_id: u32,
    pub name: String,
    pub callback_url: Option<String>,
    pub simulation_mode: SimulationMode,
    pub stk_delay: u32,
    pub prefix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UpdateProject {
    pub name: Option<String>,
    pub callback_url: Option<String>,
    pub simulation_mode: Option<SimulationMode>,
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
    pub id: u32,
    pub name: String,
    pub callback_url: Option<String>,
    pub simulation_mode: SimulationMode,
    pub business_id: u32,
    pub stk_delay: u32,
    pub prefix: Option<String>,
    pub created_at: DateTime<Utc>,
    pub consumer_key: String,
    pub consumer_secret: String,
    pub passkey: String,
}

#[derive(Serialize, Deserialize, Debug, Default, FromQueryResult)]
pub struct ProjectSummary {
    pub id: u32,
    pub name: String,
    pub simulation_mode: String,
    pub business_id: u32,
    pub business_name: String,
    pub created_at: DateTime<Utc>,
    pub short_code: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectFilter {
    pub name: Option<String>,
    pub simulation_mode: Option<SimulationMode>,
    pub has_callback_url: Option<bool>,
    pub created_after: Option<i64>,
    pub created_before: Option<i64>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl Project {
    pub async fn get_by_id<C>(conn: &C, id: u32) -> Result<Option<Project>, DbErr>
    where
        C: ConnectionTrait,
    {
        let project = db::Entity::find_by_id(id).one(conn).await?;
        Ok(project.as_ref().map(|p| p.into()))
    }

    pub async fn get_by_name<C>(conn: &C, name: &str) -> Result<Option<Project>, DbErr>
    where
        C: ConnectionTrait,
    {
        let project = db::Entity::find()
            .filter(db::Column::Name.eq(name))
            .one(conn)
            .await?;
        Ok(project.as_ref().map(|p| p.into()))
    }
}

impl From<&db::Model> for Project {
    fn from(value: &db::Model) -> Self {
        Self {
            id: value.id,
            name: value.name.to_string(),
            callback_url: value.callback_url.clone(),
            simulation_mode: value
                .simulation_mode
                .parse()
                .unwrap_or(SimulationMode::Realistic),
            stk_delay: value.stk_delay,
            prefix: value.prefix.clone(),
            created_at: value.created_at,
        }
    }
}
