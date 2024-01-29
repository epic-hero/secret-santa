pub use sea_orm_migration::prelude::*;

mod m20231017_000001_create_user_table;
mod m20231109_000002_create_messages_table;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231017_000001_create_user_table::Migration),
            Box::new(m20231109_000002_create_messages_table::Migration),
        ]
    }
}
