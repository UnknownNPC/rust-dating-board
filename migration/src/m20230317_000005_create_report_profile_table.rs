use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ReportProfile::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ReportProfile::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ReportProfile::CreatedAt)
                            .timestamp()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ReportProfile::Status).string().not_null())
                    .col(ColumnDef::new(ReportProfile::Text).string().not_null())
                    .col(ColumnDef::new(ReportProfile::UserId).big_integer())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ReportProfile::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ReportProfile {
    Table,
    Id,
    CreatedAt,
    Status,
    Text,
    UserId,
}
