use chrono::{DateTime, Utc};
use fake::faker::name::en::Name;
use fake::rand::rng;
use fake::rand::seq::IndexedRandom;
use fake::Fake;
use sea_query::{ColumnDef, Expr, Iden, Table};
use sea_query::{Query, SqliteQueryBuilder};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use sqlx::{Executor, SqlitePool};
use tauri::State;

use crate::db::Database;

#[derive(Iden)]
pub enum Users {
    Table,
    Id,
    ProjectId,
    Phone,
    Name,
    Balance,
    Status,
    CreatedAt,
    Pin,
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub id: u32,
    pub project_id: i64,
    pub phone: String,
    pub name: String,
    pub balance: f64,
    pub pin: String,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub async fn init_table(db: &SqlitePool) -> anyhow::Result<()> {
        let users_sql = {
            Table::create()
                .table(Users::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Users::Id)
                        .integer()
                        .not_null()
                        .primary_key()
                        .auto_increment(),
                )
                .col(ColumnDef::new(Users::ProjectId).integer().not_null())
                .col(ColumnDef::new(Users::Phone).text().not_null())
                .col(ColumnDef::new(Users::Name).text())
                .col(ColumnDef::new(Users::Pin).text().not_null().default("0000"))
                .col(ColumnDef::new(Users::Balance).float().not_null().default(0))
                .col(
                    ColumnDef::new(Users::Status)
                        .text()
                        .not_null()
                        .default("active"),
                )
                .col(
                    ColumnDef::new(Users::CreatedAt)
                        .integer()
                        .default(Expr::cust("(strftime('%s', 'now'))")),
                )
                .index(
                    sea_query::Index::create()
                        .name("idx_users_project_phone")
                        .col(Users::ProjectId)
                        .col(Users::Phone)
                        .unique(),
                )
                .to_string(SqliteQueryBuilder)
        };

        db.execute(users_sql.as_str()).await?;
        Ok(())
    }
    fn generate_fake_phone(existing: &mut std::collections::HashSet<String>) -> String {
        loop {
            let suffix: u64 = (10_000_000..=99_999_999).fake(); // 9-digit Safaricom-like
            let phone = format!("2547{}", suffix);
            if !existing.contains(&phone) {
                existing.insert(phone.clone());
                return phone;
            }
        }
    }

    fn generate_fake_pin() -> String {
        format!("{:04}", (0..=9999).fake::<u16>()) // 4-digit PIN
    }

    pub fn generate() -> User {
        let mut set = std::collections::HashSet::new();
        let mut rng = rng();

        let name: String = Name().fake();
        let phone = Self::generate_fake_phone(&mut set);
        let balance = [0, 5000, 10000, 25000, 100000]
            .choose(&mut rng)
            .copied()
            .unwrap_or(0);
        let pin = Self::generate_fake_pin();

        User {
            phone,
            name,
            balance: balance as f64,
            pin,
            id: 0,
            project_id: 0,
            status: None,
            created_at: DateTime::UNIX_EPOCH,
        }
    }
    pub fn generate_users(count: u32) -> Vec<User> {
        (0..count).map(|_| Self::generate()).collect()
    }

    pub async fn create(db: &SqlitePool, user: &User) -> anyhow::Result<i64> {
        let sql = {
            Query::insert()
                .into_table(Users::Table)
                .columns([
                    Users::ProjectId,
                    Users::Phone,
                    Users::Name,
                    Users::Balance,
                    Users::Status,
                    Users::Pin,
                ])
                .values([
                    user.project_id.to_string().into(),
                    user.phone.to_string().into(),
                    user.name.to_string().into(),
                    user.balance.into(),
                    user.status.clone().unwrap_or("active".to_string()).into(),
                    user.pin.to_string().into(),
                ])?
                .returning_col(Users::Id)
                .to_string(SqliteQueryBuilder)
        };

        let row = db.fetch_one(sql.as_str()).await?;

        let id: i64 = row.get(0);

        Ok(id)
    }

    pub async fn get_user_by_id(db: &SqlitePool, user_id: i64) -> anyhow::Result<Option<User>> {
        let sql = {
            Query::select()
                .columns([
                    Users::Id,
                    Users::ProjectId,
                    Users::Phone,
                    Users::Name,
                    Users::Balance,
                    Users::Pin,
                    Users::Status,
                    Users::CreatedAt,
                ])
                .from(Users::Table)
                .and_where(Expr::col(Users::ProjectId).eq(user_id))
                .to_string(SqliteQueryBuilder)
        };

        let row = db.fetch_one(sql.as_str()).await?;

        let id: u32 = row.get(0);
        let project_id: i64 = row.get(1);
        let phone: String = row.get(2);
        let name: String = row.get(3);
        let balance: f64 = row.get(4);
        let pin: String = row.get(5);
        let status: Option<String> = row.get(6);
        let created_at: i64 = row.get(7);
        let created_at = DateTime::from_timestamp(created_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

        Ok(Some(User {
            id,
            project_id,
            phone,
            name,
            balance,
            pin,
            status,
            created_at,
        }))
    }
    pub async fn get_user_by_phone(db: &SqlitePool, phone: String) -> anyhow::Result<Option<User>> {
        let sql = {
            Query::select()
                .columns([
                    Users::Id,
                    Users::ProjectId,
                    Users::Phone,
                    Users::Name,
                    Users::Balance,
                    Users::Pin,
                    Users::Status,
                    Users::CreatedAt,
                ])
                .from(Users::Table)
                .and_where(Expr::col(Users::Phone).eq(phone.as_str()))
                .to_string(SqliteQueryBuilder)
        };

        let row = db.fetch_one(sql.as_str()).await?;

        let id: u32 = row.get(0);
        let project_id: i64 = row.get(1);
        let phone: String = row.get(2);
        let name: String = row.get(3);
        let balance: f64 = row.get(4);
        let pin: String = row.get(5);
        let status: Option<String> = row.get(6);
        let created_at: i64 = row.get(7);
        let created_at = DateTime::from_timestamp(created_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

        Ok(Some(User {
            id,
            project_id,
            phone,
            name,
            balance,
            pin,
            status,
            created_at,
        }))
    }

    pub async fn list_by_project(db: &SqlitePool, project_id: i64) -> anyhow::Result<Vec<User>> {
        let sql = {
            Query::select()
                .from(Users::Table)
                .columns([
                    Users::Id,
                    Users::ProjectId,
                    Users::Phone,
                    Users::Name,
                    Users::Balance,
                    Users::Pin,
                    Users::Status,
                    Users::CreatedAt,
                ])
                .and_where(Expr::col(Users::ProjectId).eq(project_id))
                .to_string(SqliteQueryBuilder)
        };
        let rows = db.fetch_all(sql.as_str()).await?;
        let users: Vec<User> = rows
            .iter()
            .map(|row| {
                let id: u32 = row.get(0);
                let project_id: i64 = row.get(1);
                let phone: String = row.get(2);
                let name: String = row.get(3);
                let balance: f64 = row.get(4);
                let pin: String = row.get(5);
                let status: Option<String> = row.get(6);
                let created_at: i64 = row.get(7);
                let created_at =
                    DateTime::from_timestamp(created_at, 0).unwrap_or(DateTime::UNIX_EPOCH);

                User {
                    id,
                    project_id,
                    phone,
                    name,
                    balance,
                    pin,
                    status,
                    created_at,
                }
            })
            .collect();

        Ok(users)
    }

    pub async fn update_by_id(
        db: &SqlitePool,
        user_id: i64,
        name: Option<&str>,
        balance: Option<f64>,
        pin: Option<&str>,
        status: Option<&str>,
    ) -> anyhow::Result<()> {
        let sql = {
            let mut sql = Query::update();

            if let Some(name) = name {
                sql.value(Users::Name, name);
            }

            if let Some(balance) = balance {
                sql.value(Users::Balance, balance);
            }

            if let Some(pin) = pin {
                sql.value(Users::Pin, pin);
            }

            if let Some(status) = status {
                sql.value(Users::Status, status);
            }

            sql.table(Users::Table);
            sql.and_where(Expr::col(Users::Id).eq(user_id));
            sql.to_string(SqliteQueryBuilder)
        };

        db.execute(sql.as_str()).await?;

        Ok(())
    }

    pub async fn delete(db: &SqlitePool, user_id: i64) -> anyhow::Result<()> {
        let sql = {
            Query::delete()
                .from_table(Users::Table)
                .and_where(Expr::col(Users::Id).eq(user_id))
                .to_string(SqliteQueryBuilder)
        };

        db.execute(sql.as_str()).await?;
        Ok(())
    }
}

#[tauri::command]
pub async fn get_users(state: State<'_, Database>, project_id: i64) -> Result<Vec<User>, String> {
    let users = User::list_by_project(&state.pool, project_id)
        .await
        .map_err(|err| format!("Failed to get users: {}", err))?;

    Ok(users)
}
#[tauri::command]
pub async fn create_user(
    state: State<'_, Database>,
    project_id: i64,
    name: String,
    phone: String,
    balance: f64,
    pin: String,
) -> Result<i64, String> {
    let user = User {
        id: 0,
        project_id,
        phone,
        name,
        balance,
        pin,
        status: None,
        created_at: DateTime::UNIX_EPOCH,
    };

    let id = User::create(&state.pool, &user)
        .await
        .map_err(|err| format!("Failed to create user: {}", err))?;

    Ok(id)
}
#[tauri::command]
pub async fn remove_user(state: State<'_, Database>, user_id: i64) -> Result<(), String> {
    User::delete(&state.pool, user_id)
        .await
        .map_err(|err| format!("Failed to delete user: {}", err))?;

    Ok(())
}
#[tauri::command]
pub async fn get_user(state: State<'_, Database>, user_id: i64) -> Result<Option<User>, String> {
    let user = User::get_user_by_id(&state.pool, user_id)
        .await
        .map_err(|err| format!("Failed to get user: {}", err))?;

    Ok(user)
}
#[tauri::command]
pub async fn generate_user() -> Result<Value, String> {
    let user = User::generate();
    Ok(json!({
        "name": user.name,
        "phone": user.phone,
        "balance": user.balance,
        "pin": user.pin,
    }))
}

#[tauri::command]
pub async fn update_user(
    state: State<'_, Database>,
    user_id: i64,
    name: Option<&str>,
    balance: Option<f64>,
    pin: Option<&str>,
    status: Option<&str>,
) -> Result<(), String> {
    User::update_by_id(&state.pool, user_id, name, balance, pin, status)
        .await
        .map_err(|err| format!("Failed to update user: {}", err))?;

    Ok(())
}
