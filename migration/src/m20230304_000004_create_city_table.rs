use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::ConnectionTrait;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(City::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(City::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(City::Name).string().not_null())
                    .col(ColumnDef::new(City::Status).string().not_null())
                    .to_owned(),
            ).await?;

        let db = manager.get_connection();

        db.execute_unprepared(
            "INSERT INTO CITY (NAME, STATUS) VALUES ('kiev', 'on'), ('lviv', 'on'), ('odesa', 'on'),
            ('ujgorod', 'on'), ('ivano-frankivsk', 'on'), ('chernivci', 'on'), ('ternopil', 'on'), ('luck', 'on'), 
            ('rivne', 'on'), ('hmelnickij', 'on'), ('jutomyr', 'on'), ('vinnica', 'on'), ('chernigiv', 'on'), 
            ('symu', 'on'), ('cherkasy', 'on'), ('poltava', 'on'), ('harkiv', 'on'), ('lugansk', 'on'), 
            ('doneck', 'on'), ('dnipro', 'on'), ('zaporija', 'on'), ('herson', 'on'), ('mykolaiv', 'on'), 
            ('kropivnyckij', 'on'), ('simferopol', 'on');"
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(City::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum City {
    Table,
    Id,
    Name,
    Status
}
