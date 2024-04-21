pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_users;
mod m20240318_172753_create_chats;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_users::Migration),
            Box::new(m20240318_172753_create_chats::Migration),
        ]
    }
}
