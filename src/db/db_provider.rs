
use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::DbConn;
use sea_orm::{ActiveModelTrait, DbErr, Set};

use super::profile::{self, Model as ProfileModel};
use super::user::{self, Model as UserModel};

#[derive(Clone)]
pub struct DbProvider {
    pub db_con: DbConn,
}

impl DbProvider {
    pub fn new(db_con: DbConn) -> Self {
        DbProvider { db_con }
    }

    pub async fn add_user(
        &self,
        id: Option<i64>,
        name: &str,
        email: &str,
    ) -> Result<UserModel, DbErr> {
        let user = user::ActiveModel {
            id: id.map_or(NotSet, |f| Set(f)),
            name: Set(name.to_string()),
            email: Set(email.to_string()),
            created_at: Set(Utc::now().naive_utc()),
            ..Default::default()
        };
        user.insert(&self.db_con).await
    }

    pub async fn add_profile(&self) -> Result<ProfileModel, DbErr> {
        let profile = profile::ActiveModel {
            id: NotSet,
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            name: Set(String::from("name")),
            height: Set(165),
            cost_per_hour: Set(2000),
            description: Set(String::from("description")),
            phone_number: Set(String::from("03333333")),
            user_id: Set(1),
            city: Set(String::from("Kiev")),
            region: Set(String::from("Central")),
            ..Default::default()
        };
        profile.insert(&self.db_con).await
    }
}
