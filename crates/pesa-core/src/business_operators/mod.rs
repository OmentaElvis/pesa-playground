use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, FromQueryResult,
    QueryFilter,
};
use serde::{Deserialize, Serialize};

pub mod db;
pub mod ui;

#[derive(FromQueryResult, Debug, Serialize, Deserialize, Clone)]
pub struct BusinessOperator {
    pub id: u32,
    pub username: String,
    pub password: String,
    pub business_id: u32,
}

impl From<db::Model> for BusinessOperator {
    fn from(value: db::Model) -> Self {
        BusinessOperator {
            id: value.id,
            username: value.username,
            password: value.password,
            business_id: value.business_id,
        }
    }
}

impl BusinessOperator {
    pub async fn create<C>(
        db: &C,
        username: String,
        password: String,
        business_id: u32,
    ) -> anyhow::Result<Self>
    where
        C: ConnectionTrait,
    {
        let new_operator = db::ActiveModel {
            username: Set(username),
            password: Set(password),
            business_id: Set(business_id),
            ..Default::default()
        };
        let operator = new_operator.insert(db).await?;

        Ok(operator.into())
    }

    pub async fn find_by_business<C>(
        db: &C,
        username: String,
        business_id: u32,
    ) -> anyhow::Result<Option<Self>>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .filter(db::Column::Username.eq(username))
            .into_model::<BusinessOperator>()
            .one(db)
            .await?;

        Ok(model)
    }

    pub async fn get_business_operators<C>(
        db: &C,
        business_id: u32,
    ) -> anyhow::Result<Vec<BusinessOperator>>
    where
        C: ConnectionTrait,
    {
        let models = db::Entity::find()
            .filter(db::Column::BusinessId.eq(business_id))
            .into_model::<BusinessOperator>()
            .all(db)
            .await?;

        Ok(models)
    }

    pub async fn get_operator<C>(db: &C, operator: u32) -> anyhow::Result<Option<BusinessOperator>>
    where
        C: ConnectionTrait,
    {
        let model = db::Entity::find_by_id(operator)
            .into_model::<BusinessOperator>()
            .one(db)
            .await?;

        Ok(model)
    }

    pub async fn delete_operator<C>(db: &C, operator: u32) -> anyhow::Result<bool>
    where
        C: ConnectionTrait,
    {
        let result = db::Entity::delete_by_id(operator).exec(db).await?;
        Ok(result.rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::business::{Business, CreateBusiness};
    use crate::tests::TestDb;

    async fn setup_test_business(db: &TestDb) -> u32 {
        let input = CreateBusiness {
            name: "Test Business".to_string(),
            short_code: "123456".to_string(),
            initial_working_balance: 0.0,
            initial_utility_balance: 0.0,
        };
        let business = Business::create(&db.conn, input).await.unwrap();
        business.id
    }

    #[tokio::test]
    async fn test_create_operator_success() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;

        let operator = BusinessOperator::create(
            &db.conn,
            "testuser".to_string(),
            "pass123".to_string(),
            business_id,
        )
        .await
        .unwrap();

        assert_eq!(operator.username, "testuser");
        assert_eq!(operator.business_id, business_id);
    }

    #[tokio::test]
    async fn test_operator_username_collision_same_business() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;

        BusinessOperator::create(
            &db.conn,
            "collison".to_string(),
            "pass".to_string(),
            business_id,
        )
        .await
        .unwrap();

        // This should probably fail if we want unique usernames per business
        let result = BusinessOperator::create(
            &db.conn,
            "collison".to_string(),
            "pass2".to_string(),
            business_id,
        )
        .await;

        // If it succeeds, we have a bug in logic/schema
        assert!(
            result.is_err(),
            "Should not allow duplicate usernames in same business"
        );
    }

    #[tokio::test]
    async fn test_operator_username_collision_different_businesses() {
        let db = TestDb::in_memory().await.unwrap();
        let biz1 = setup_test_business(&db).await;
        let biz2 = Business::create(
            &db.conn,
            CreateBusiness {
                name: "Biz 2".to_string(),
                short_code: "654321".to_string(),
                initial_working_balance: 0.0,
                initial_utility_balance: 0.0,
            },
        )
        .await
        .unwrap()
        .id;

        BusinessOperator::create(&db.conn, "sharedname".to_string(), "pass".to_string(), biz1)
            .await
            .unwrap();

        // This should succeed
        let result =
            BusinessOperator::create(&db.conn, "sharedname".to_string(), "pass".to_string(), biz2)
                .await;
        assert!(
            result.is_ok(),
            "Should allow same username in different businesses"
        );
    }

    #[tokio::test]
    async fn test_find_by_business_not_found() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;

        let result =
            BusinessOperator::find_by_business(&db.conn, "nonexistent".to_string(), business_id)
                .await
                .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_business_operators() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;
        // Business::create already creates 'admin'
        BusinessOperator::create(
            &db.conn,
            "user2".to_string(),
            "pass".to_string(),
            business_id,
        )
        .await
        .unwrap();

        let operators = BusinessOperator::get_business_operators(&db.conn, business_id)
            .await
            .unwrap();
        assert_eq!(operators.len(), 2); // admin + user2
    }

    #[tokio::test]
    async fn test_delete_operator() {
        let db = TestDb::in_memory().await.unwrap();
        let business_id = setup_test_business(&db).await;
        let operator = BusinessOperator::create(
            &db.conn,
            "delme".to_string(),
            "pass".to_string(),
            business_id,
        )
        .await
        .unwrap();

        let result = BusinessOperator::delete_operator(&db.conn, operator.id)
            .await
            .unwrap();
        assert!(result);

        let verify = BusinessOperator::get_operator(&db.conn, operator.id)
            .await
            .unwrap();
        assert!(verify.is_none());
    }
}
