use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(User::Id)
                            .big_unsigned()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(User::ChatId)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::Child)
                            .big_unsigned()
                    )
                    .col(
                        ColumnDef::new(User::Santa)
                            .big_unsigned()
                    )
                    .col(
                        ColumnDef::new(User::Nickname)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::Username)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::State)
                            .text(),
                    )
                    .col(
                        ColumnDef::new(User::WishText)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::City)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(User::CreateDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum User {
    Table,
    Id,
    ChatId,
    Nickname,
    Username,
    State,
    WishText,
    City,
    Child,
    Santa,
    CreateDate,
}
