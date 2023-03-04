use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, QueryFilter, Set};
use sea_orm::{DbConn, EntityTrait};

use super::profile::{self, Model as ProfileModel};
use super::profile_photo::{self, Model as ProfilePhotoModel};
use super::user::{self, Model as UserModel};
use super::city::{self, Model as CityModel};

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
            .filter(user::Column::Email.eq(email))
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

    pub async fn find_draft_profile_for(
        &self,
        user_id: i64,
    ) -> Result<Option<ProfileModel>, DbErr> {
        profile::Entity::find()
            .filter(profile::Column::UserId.eq(user_id))
            .filter(profile::Column::Status.eq("draft"))
            .one(&self.db_con)
            .await
    }

    pub async fn find_all_profile_photos_for(
        &self,
        profile_id: i64,
    ) -> Result<Vec<ProfilePhotoModel>, DbErr> {
        profile_photo::Entity::find()
            .filter(profile_photo::Column::ProfileId.eq(profile_id))
            .filter(profile_photo::Column::Status.eq("active"))
            .all(&self.db_con)
            .await
    }

    pub async fn add_draft_profile_for(&self, user_id: i64) -> Result<ProfileModel, DbErr> {
        let profile = profile::ActiveModel {
            id: NotSet,
            created_at: Set(Utc::now().naive_utc()),
            updated_at: Set(Utc::now().naive_utc()),
            name: Set(String::from("")),
            height: Set(0),
            description: Set(String::from("")),
            phone_number: Set(String::from("")),
            user_id: Set(user_id),
            city: Set(String::from("")),
            status: Set(String::from("draft")),
            ..Default::default()
        };
        profile.insert(&self.db_con).await
    }

    pub async fn add_profile_photo(
        &self,
        profile_id: i64,
        file_name: &str,
        file_size: i64,
    ) -> Result<ProfilePhotoModel, DbErr> {
        let profile_photo = profile_photo::ActiveModel {
            id: NotSet,
            created_at: Set(Utc::now().naive_utc()),
            status: Set(String::from("active")),
            profile_id: Set(profile_id),
            file_name: Set(file_name.to_string()),
            size: Set(file_size)
        };
        profile_photo.insert(&self.db_con).await
    }

    pub async fn update_profile_photo_with_delete_status(
        &self,
        model: ProfilePhotoModel,
    ) -> Result<ProfilePhotoModel, DbErr> {
        let mut mutable: profile_photo::ActiveModel = model.into();
        mutable.status = Set("deleted".to_owned());

        mutable.update(&self.db_con).await
    }

    pub async fn find_cities_on(&self) -> Result<Vec<CityModel>, DbErr> {
        city::Entity::find().filter(city::Column::Status.eq("on")).all(&self.db_con).await
    }

}
