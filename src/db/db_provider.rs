use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, QueryFilter, Set};
use sea_orm::{DbConn, EntityTrait};

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

    pub async fn find_user_by_id(&self, id: i64) -> Result<Option<UserModel>, DbErr> {
        user::Entity::find_by_id(id).one(&self.db_con).await
    }

    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<UserModel>, DbErr> {
        user::Entity::find()
            .filter(user::Column::Email.contains(email))
            .one(&self.db_con)
            .await
    }

    pub async fn add_user(
        &self,
        id: Option<i64>,
        name: &str,
        email: &str,
        provider_opt: Option<&str>,
    ) -> Result<UserModel, DbErr> {
        let user = user::ActiveModel {
            id: id.map_or(NotSet, |f| Set(f)),
            name: Set(name.to_string()),
            email: Set(email.to_string()),
            created_at: Set(Utc::now().naive_utc()),
            provider: Set(provider_opt.map(|f| f.to_string())),
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
            description: Set(String::from("description")),
            phone_number: Set(String::from("03333333")),
            user_id: Set(1),
            city: Set(String::from("Kiev")),
            status: Set(String::from("draft")),
            ..Default::default()
        };
        profile.insert(&self.db_con).await
    }
}
