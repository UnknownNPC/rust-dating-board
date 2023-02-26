use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::{Database, DbConn, DbErr};

use super::profile;

pub struct Service;

impl Service {
    async fn establish_connection() -> Result<DbConn, DbErr> {
        let database_url = std::env::var("DATABASE_URL").unwrap();
        Database::connect(&database_url).await
    }

    pub async fn addProfile() {
        let db = Self::establish_connection().await.unwrap();
        let post = profile::ActiveModel {
            id: NotSet,
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            name: Set(String::from("name")),
            height: Set(165),
            cost_per_hour: Set(2000),
            description: Set(String::from("description")),
            phone_number: Set(String::from("03333333")),
            city: Set(String::from("Kiev")),
            region: Set(String::from("Central")),
            ..Default::default()
        };
        post.insert(&db).await;
    }
}
