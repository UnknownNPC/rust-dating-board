pub use sea_orm_migration::prelude::*;

mod m20230223_000001_create_user_table;
mod m20230223_000002_create_profile_table;
mod m20230223_000003_create_profilephoto_table;
mod m20230304_000004_create_city_table;
mod m20240408_000005_alter_profile_with_weight;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230223_000001_create_user_table::Migration),
            Box::new(m20230223_000002_create_profile_table::Migration),
            Box::new(m20230223_000003_create_profilephoto_table::Migration),
            Box::new(m20230304_000004_create_city_table::Migration),
            Box::new(m20240408_000005_alter_profile_with_weight::Migration),
        ]
    }
}
