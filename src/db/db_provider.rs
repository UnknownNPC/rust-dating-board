use std::collections::HashMap;

use chrono::Utc;
use futures::future::try_join_all;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{
    query::*, ActiveModelTrait, ColumnTrait, DbBackend, DbErr, Order, PaginatorTrait, QueryFilter,
    QueryOrder, Set, Statement, FromQueryResult,
};
use sea_orm::{DbConn, EntityTrait};

use super::city::{self};
use super::profile::{self, Model as ProfileModel};
use super::profile_photo::{self, Model as ProfilePhotoModel};
use super::user::{self, Model as UserModel};

#[derive(Clone)]
pub struct DbProvider {
    pub db_con: DbConn,
}

type TotalPages = u64;

impl DbProvider {
    pub fn new(db_con: DbConn) -> Self {
        DbProvider { db_con }
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

    pub async fn count_profile_photos(&self, profile_id: i64) -> Result<u64, DbErr> {
        profile_photo::Entity::find()
        .filter(profile_photo::Column::ProfileId.eq(profile_id))
        .filter(profile_photo::Column::Status.eq("active"))
        .count(&self.db_con).await
    }

    pub async fn find_active_profile_photo_with_profile_by_id_and_user_id(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<Option<(ProfilePhotoModel, ProfileModel)>, DbErr> {
        profile_photo::Entity::find_by_id(id)
            .filter(profile_photo::Column::Status.eq("active"))
            .find_also_related(profile::Entity)
            .filter(profile::Column::UserId.eq(user_id))
            .one(&self.db_con)
            .await
            .map(|res| res.map(|data| (data.0, data.1.unwrap())))
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

    pub async fn find_active_profile_by(&self, id: i64) -> Result<Option<ProfileModel>, DbErr> {
        profile::Entity::find_by_id(id)
            .filter(profile::Column::Status.eq("active"))
            .one(&self.db_con)
            .await
    }

    pub async fn find_active_profile_by_id_and_user_id(
        &self,
        id: i64,
        user_id: i64,
    ) -> Result<Option<ProfileModel>, DbErr> {
        profile::Entity::find_by_id(id)
            .filter(profile::Column::Status.eq("active"))
            .filter(profile::Column::UserId.eq(user_id))
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
            size: Set(file_size),
        };
        profile_photo.insert(&self.db_con).await
    }

    pub async fn update_profile_photo_with_delete_status(
        &self,
        model: &ProfilePhotoModel,
    ) -> Result<ProfilePhotoModel, DbErr> {
        let mut mutable: profile_photo::ActiveModel = model.to_owned().into();
        mutable.status = Set("deleted".to_owned());

        mutable.update(&self.db_con).await
    }

    pub async fn publish_profie(
        &self,
        model: &ProfileModel,
        name: &str,
        height: i16,
        city: &str,
        description: &str,
        phone_number: &str,
    ) -> Result<ProfileModel, DbErr> {
        let mut mutable: profile::ActiveModel = model.to_owned().into();
        mutable.name = Set(name.to_owned());
        mutable.height = Set(height);
        mutable.city = Set(city.to_owned());
        mutable.description = Set(description.to_owned());
        mutable.phone_number = Set(phone_number.to_owned());
        mutable.status = Set("active".to_owned());

        mutable.update(&self.db_con).await
    }

    pub async fn find_city_names(&self) -> Result<Vec<String>, DbErr> {
        let query_result = city::Entity::find()
            .select_only()
            .column(city::Column::Name)
            .filter(city::Column::Status.eq("on"))
            .into_model::<NameResult>()
            .all(&self.db_con)
            .await?;

        Ok(query_result.iter().map(|row| row.name.to_owned()).collect())
    }

    pub async fn all_user_profiles(&self, user_id: i64) -> Result<Vec<ProfileModel>, DbErr> {
        println!("[db_provider#all_user_profiles] User fetches all his profiles");
        profile::Entity::find()
            .filter(profile::Column::Status.eq("active"))
            .filter(profile::Column::UserId.eq(user_id))
            .order_by(profile::Column::CreatedAt, Order::Desc)
            .all(&self.db_con)
            .await
    }

    pub async fn search_profiles(
        &self,
        text: &str,
        limit: i64,
    ) -> Result<Vec<ProfileModel>, DbErr> {
        println!(
            "[db_provider#search_profiles] User search for profiles: {}",
            text
        );

        profile_photo::Entity::find()
            .from_raw_sql(Statement::from_sql_and_values(
                DbBackend::Postgres,
                "SELECT * FROM profile WHERE to_tsvector(phone_number) || to_tsvector(description) || to_tsvector(name) @@ plainto_tsquery($1) 
                AND status = 'active' order by created_at desc limit $2;",
                [text.into(), limit.into()],
            ))
            .into_model::<ProfileModel>()
            .all(&self.db_con)
            .await
    }

    pub async fn profiles_pagination(
        &self,
        number_of_entities: u64,
        page_opt: &Option<u64>,
        city_opt: &Option<String>,
    ) -> Result<(TotalPages, Vec<ProfileModel>), DbErr> {
        let query = profile::Entity::find()
            .filter(profile::Column::Status.eq("active"))
            .apply_if(city_opt.to_owned(), |query, v| {
                query.filter(profile::Column::City.eq(v))
            })
            .order_by(profile::Column::CreatedAt, Order::Desc)
            .paginate(&self.db_con, number_of_entities);

        let total_pages = query.num_pages().await.unwrap();

        let query_page = page_opt.map(|f| if f > 0 { f - 1 } else { f }).unwrap_or(0);
        println!(
            "[db_providing#find_pagination] Fetching page: [{}]. City: [{}]. Total num of pages: [{}]",
            query_page + 1,
            city_opt.as_deref().unwrap_or_default(),
            total_pages
        );
        let profiles = query.fetch_page(query_page).await;

        profiles.map(|data| (total_pages, data))
    }

    pub async fn find_first_profile_photos_for(
        &self,
        profile_ids: &Vec<i64>,
    ) -> Result<HashMap<i64, Option<ProfilePhotoModel>>, DbErr> {
        let profile_photo_str_ids: Vec<String> =
            profile_ids.clone().iter().map(|f| f.to_string()).collect();

        if profile_ids.is_empty() {
            Ok(HashMap::new())
        } else {
            let query = format!(
                "SELECT DISTINCT ON (profile_id) * FROM profile_photo WHERE status = 'active' and profile_id IN ({})",
                profile_photo_str_ids.join(",")
            );

            let profile_photos_res = profile_photo::Entity::find()
                .from_raw_sql(Statement::from_sql_and_values(
                    DbBackend::Postgres,
                    query.as_str(),
                    [],
                ))
                .into_model::<ProfilePhotoModel>()
                .all(&self.db_con)
                .await;

            profile_photos_res.map(|profile_photos| {
                let search = profile_ids
                    .iter()
                    .map(|profile_id_ref| {
                        let profile_id = profile_id_ref.to_owned();
                        let profile_photo_opt = profile_photos
                            .iter()
                            .find(|photo| photo.profile_id == profile_id)
                            .map(|f| f.to_owned());

                        (profile_id, profile_photo_opt)
                    })
                    .collect();
                search
            })
        }
    }

    pub async fn delete_profile_and_photos(
        &self,
        profile_model: &ProfileModel,
        profole_photos: &Vec<ProfilePhotoModel>,
    ) -> Result<(), DbErr> {
        let mut mutable_profile: profile::ActiveModel = profile_model.to_owned().into();
        mutable_profile.status = Set("deleted".to_owned());

        mutable_profile.update(&self.db_con).await?;

        let update_photos_futs = profole_photos
            .into_iter()
            .map(|f| async { self.update_profile_photo_with_delete_status(f).await })
            .collect::<Vec<_>>();
        try_join_all(update_photos_futs).await?;

        Ok(())
    }
}

#[derive(Debug, FromQueryResult)]
struct NameResult {
    name: String,
}