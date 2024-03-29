use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Message::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Message::SantaId)
                            .big_unsigned()
                    )
                    .col(
                        ColumnDef::new(Message::ChildId)
                            .big_unsigned()
                    )
                    .col(
                        ColumnDef::new(Message::Message)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Message::CreateDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Message {
    Table,
    Id,
    SantaId,
    ChildId,
    Message,
    CreateDate,
}
