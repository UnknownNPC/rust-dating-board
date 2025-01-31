use super::m20230223_000001_create_user_table::User;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Profile::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Profile::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Profile::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Profile::UpdatedAt).timestamp().not_null())
                    .col(ColumnDef::new(Profile::Name).string().not_null())
                    .col(ColumnDef::new(Profile::Height).small_integer().not_null())
                    .col(ColumnDef::new(Profile::Description).string().not_null())
                    .col(ColumnDef::new(Profile::PhoneNumber).string().not_null())
                    .col(ColumnDef::new(Profile::City).string().not_null())
                    .col(ColumnDef::new(Profile::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Profile::Status).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-profile-user_id")
                            .from(Profile::Table, Profile::UserId)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-profile-phonenum")
                    .table(Profile::Table)
                    .col(Profile::PhoneNumber)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Profile::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Profile {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Name,
    Height,
    Weight,
    Description,
    PhoneNumber,
    City,
    UserId,
    Status,
    ViewCount
}
