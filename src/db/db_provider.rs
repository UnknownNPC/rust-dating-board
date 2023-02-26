use chrono::Utc;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{ActiveModelTrait, Set};
use sea_orm::DbConn;

use super::profile;

pub struct DbProvider<'a> {
    pub db_con: &'a DbConn,
}

impl<'a> DbProvider<'a> {
    pub fn new(db_con: &'a DbConn) -> Self {
        DbProvider { db_con }
    }

    pub async fn add_profile(&self) {
        let post = profile::ActiveModel {
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
        let res = post.insert(self.db_con);
        println!("test");
        ()
    }
}
