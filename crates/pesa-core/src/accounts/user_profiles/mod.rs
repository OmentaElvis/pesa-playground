use anyhow::Context;
use chrono::{DateTime, Duration, Utc};
use fake::{Fake, faker::name::en::Name};
use rand::{Rng, seq::SliceRandom};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DbErr, EntityTrait,
    FromQueryResult, QueryFilter, QuerySelect, RelationTrait, SelectColumns, prelude::DateTimeUtc,
};
use serde::{Deserialize, Serialize};

use crate::accounts::{self, Account, AccountType};

pub mod db;
pub mod ui;

#[derive(Serialize, FromQueryResult, Deserialize, Debug, Clone)]
pub struct User {
    pub account_id: u32,
    pub phone: String,
    pub name: String,
    pub pin: String,
    pub balance: i64,
    pub disabled: bool,
    pub created_at: DateTimeUtc,
    pub registered_at: DateTimeUtc,
    pub last_swap_date: Option<DateTimeUtc>,
    pub imsi: String,
}

impl User {
    pub async fn get_users<C>(conn: &C) -> anyhow::Result<Vec<User>>
    where
        C: ConnectionTrait,
    {
        let users: Vec<User> = crate::accounts::user_profiles::db::Entity::find()
            .join(sea_orm::JoinType::InnerJoin, db::Relation::Account.def())
            .select_column(db::Column::AccountId)
            .select_column(accounts::db::Column::Balance)
            .select_column(accounts::db::Column::AccountType)
            .select_column(accounts::db::Column::CreatedAt)
            .select_column(accounts::db::Column::Disabled)
            .into_model::<User>()
            .all(conn)
            .await?;

        Ok(users)
    }
    pub async fn create_from<C>(
        conn: &C,
        phone: String,
        name: String,
        pin: String,
        balance: i64,
    ) -> anyhow::Result<User>
    where
        C: ConnectionTrait,
    {
        let random_registration = Self::random_registration_date();
        let imsi = Self::generate_test_imsi();

        let user = User {
            account_id: 0,
            phone,
            name,
            pin,
            balance,
            disabled: false,
            created_at: Utc::now().to_utc(),
            registered_at: random_registration,
            last_swap_date: None,
            imsi,
        };

        user.create(conn).await
    }
    pub async fn create<C>(self, conn: &C) -> anyhow::Result<User>
    where
        C: ConnectionTrait,
    {
        let account = Account::create_account(conn, AccountType::User, self.balance)
            .await
            .context("Failed to create user account")?;

        let model = db::ActiveModel {
            account_id: Set(account.id),
            name: Set(self.name.to_string()),
            phone: Set(self.phone.to_string()),
            pin: Set(self.pin.to_string()),
            imsi: Set(self.imsi.to_string()),
            registered_at: Set(self.registered_at),
            last_swap_date: Set(self.last_swap_date),
        };

        model.insert(conn).await?;

        Ok(User {
            account_id: account.id,
            phone: self.phone,
            name: self.name,
            pin: self.pin,
            balance: account.balance,
            disabled: self.disabled,
            created_at: account.created_at,
            registered_at: self.registered_at,
            last_swap_date: self.last_swap_date,
            imsi: self.imsi,
        })
    }
    pub async fn update_by_id<C>(
        conn: &C,
        user_id: u32,
        name: Option<String>,
        pin: Option<String>,
        phone: Option<String>,
    ) -> anyhow::Result<()>
    where
        C: ConnectionTrait,
    {
        let user = db::Entity::find_by_id(user_id).one(conn).await?;

        if let Some(user) = user {
            let mut active_model: crate::accounts::user_profiles::db::ActiveModel = user.into();

            if let Some(name) = name {
                active_model.name = Set(name);
            }
            if let Some(pin) = pin {
                active_model.pin = Set(pin);
            }
            if let Some(number) = phone {
                active_model.phone = Set(number);
                active_model.last_swap_date = Set(Some(Utc::now().to_utc()));
            }

            active_model.update(conn).await?;
        }

        Ok(())
    }

    pub async fn get_user_by_phone<C>(conn: &C, phone: &str) -> Result<Option<User>, DbErr>
    where
        C: ConnectionTrait,
    {
        let user: Option<User> = crate::accounts::user_profiles::db::Entity::find()
            .filter(crate::accounts::user_profiles::db::Column::Phone.eq(phone))
            .join(sea_orm::JoinType::InnerJoin, db::Relation::Account.def())
            .select_column(db::Column::AccountId)
            .select_column(accounts::db::Column::Balance)
            .select_column(accounts::db::Column::AccountType)
            .select_column(accounts::db::Column::CreatedAt)
            .select_column(accounts::db::Column::Disabled)
            .into_model::<User>()
            .one(conn)
            .await?;

        Ok(user)
    }
    pub async fn find_by_id<C>(conn: &C, id: u32) -> Result<Option<User>, DbErr>
    where
        C: ConnectionTrait,
    {
        let user: Option<User> = crate::accounts::user_profiles::db::Entity::find_by_id(id)
            .join(sea_orm::JoinType::InnerJoin, db::Relation::Account.def())
            .select_column(db::Column::AccountId)
            .select_column(accounts::db::Column::Balance)
            .select_column(accounts::db::Column::AccountType)
            .select_column(accounts::db::Column::CreatedAt)
            .select_column(accounts::db::Column::Disabled)
            .into_model::<User>()
            .one(conn)
            .await?;

        Ok(user)
    }
    fn generate_phone_number(existing: &mut std::collections::HashSet<String>) -> String {
        loop {
            let suffix: u64 = (10_000_000..=99_999_999).fake();
            let phone = format!("2547{}", suffix);
            if !existing.contains(&phone) {
                existing.insert(phone.clone());
                return phone;
            }
        }
    }
    pub fn random_registration_date() -> DateTime<Utc> {
        let now = Utc::now();
        let two_years_ago = now - Duration::days(365 * 2);

        let seconds_range = now.timestamp() - two_years_ago.timestamp();
        let random_offset = rand::thread_rng().gen_range(0..=seconds_range);

        two_years_ago + Duration::seconds(random_offset)
    }

    fn generate_pin() -> String {
        format!("{:04}", (0..=9999).fake::<u16>()) // 4-digit PIN
    }

    pub fn generate_test_imsi() -> String {
        let mcc = "001"; // test MCC
        let mnc = "01"; // test MNC

        let mut rng = rand::thread_rng();
        let msin: String = (0..10).map(|_| rng.gen_range(0..10).to_string()).collect();

        format!("{}{}{}", mcc, mnc, msin)
    }

    pub fn generate() -> User {
        let mut set = std::collections::HashSet::new();

        let name: String = Name().fake();
        let phone = Self::generate_phone_number(&mut set);
        let pin = Self::generate_pin();
        // available balances to select in cents
        let balance = [
            250_000 * 100,
            1000 * 100,
            0,
            200 * 100,
            42000 * 100,
            14 * 100,
            120000 * 100,
            3_141_592 * 100,
        ]
        .choose(&mut rand::thread_rng())
        .unwrap();
        let imsi = Self::generate_test_imsi();
        let registered_at = Self::random_registration_date();

        User {
            phone,
            name,
            pin,
            account_id: 0,
            balance: *balance,
            disabled: false,
            created_at: Utc::now().to_utc(),
            imsi,
            registered_at,
            last_swap_date: None,
        }
    }

    pub fn generate_users(count: u32) -> Vec<User> {
        (0..count).map(|_| Self::generate()).collect()
    }

    pub async fn disable_user<C>(conn: &C, user_id: u32) -> anyhow::Result<()>
    where
        C: ConnectionTrait,
    {
        let account = accounts::db::Entity::find_by_id(user_id)
            .one(conn)
            .await
            .context(format!("Failed to get user account ({})", user_id))?;

        let mut account: accounts::db::ActiveModel = account
            .ok_or_else(|| anyhow::anyhow!("User ({}) not found.", user_id))?
            .into();
        account.disabled = Set(true);
        account
            .update(conn)
            .await
            .context("Failed to update user account")?;

        Ok(())
    }

    pub async fn delete_user<C>(conn: &C, user_id: u32) -> anyhow::Result<()>
    where
        C: ConnectionTrait,
    {
        // First delete the user_profile, which has a foreign key to account
        db::Entity::delete_by_id(user_id).exec(conn).await?;

        // Then delete the account
        accounts::db::Entity::delete_by_id(user_id).exec(conn).await?;

        Ok(())
    }
}
