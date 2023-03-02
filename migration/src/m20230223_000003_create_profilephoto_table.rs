use sea_orm_migration::prelude::*;
use super::m20230223_000002_create_profile_table::Profile;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProfilePhoto::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProfilePhoto::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ProfilePhoto::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProfilePhoto::Status).string().not_null())
                    .col(ColumnDef::new(ProfilePhoto::OriginalFileName).string().not_null())
                    .col(
                        ColumnDef::new(ProfilePhoto::ProfileId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-profilephoto-profile_id")
                            .from(ProfilePhoto::Table, ProfilePhoto::ProfileId)
                            .to(Profile::Table, Profile::Id)
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ProfilePhoto::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ProfilePhoto {
    Table,
    Id,
    CreatedAt,
    Status,
    OriginalFileName,
    ProfileId,
}
