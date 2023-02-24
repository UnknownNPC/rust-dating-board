use sea_orm::ActiveValue::NotSet;
use sea_orm::{Database, DbConn, DbErr};
use sea_orm::{Set, ActiveModelTrait};
use chrono::Utc;

use super::profile;

struct Service;

impl Service {
    async fn establish_connection() -> Result<DbConn, DbErr> {
        let database_url = std::env::var("DATABASE_URL").unwrap();
        Database::connect(&database_url).await
    }

    async fn addProfile() {
        let post = profile::ActiveModel {
            id: NotSet,
            created_at: Set(Utc::now().timestamp().to_owned()),
            updated_at: Utc::now().timestamp(),
            name: String::from("name"),
            height: 165,
            cost_per_hour: 2000,
            description: String::from("description"),
            phone_number: String::from("03333333"),
            city: String::from("Kiev"),
            region: String::from("Central"),
            ..Default::default()
        };
        post.in
    }


}
