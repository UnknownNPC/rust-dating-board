use sea_orm_migration::prelude::*;

use crate::{
    m20230223_000001_create_user_table::User, m20230223_000002_create_profile_table::Profile,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Comment::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Comment::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Comment::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Comment::ProfileId).uuid().not_null())
                    .col(
                        ColumnDef::new(Comment::Status)
                            .enumeration(
                                CommentStatus::Enum,
                                [
                                    CommentStatus::Approved,
                                    CommentStatus::InReview,
                                    CommentStatus::Deleted,
                                ],
                            )
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Comment::CreatedAt)
                            .timestamp()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp))
                            .not_null(),
                    )
                    .col(ColumnDef::new(Comment::Text).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Comment::Table, Comment::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Comment::Table, Comment::ProfileId)
                            .to(Profile::Table, Profile::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Comment::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Comment {
    Table,
    Id,
    UserId,
    ProfileId,
    Status,
    CreatedAt,
    Text,
}

#[derive(Iden)]
pub enum CommentStatus {
    #[iden = "status"]
    Enum,
    #[iden = "deteled"]
    Deleted,
    #[iden = "approved"]
    Approved,
    #[iden = "in_review"]
    InReview,
}
