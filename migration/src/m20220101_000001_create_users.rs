use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
        CREATE TABLE users (
            id SERIAL8 PRIMARY KEY,
            telegram_id VARCHAR UNIQUE NOT NULL CHECK(telegram_id ~ '^\d*$') ,
            username VARCHAR UNIQUE NULL
        );
        "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
        DROP TABLE users;
        "#,
            )
            .await?;
        Ok(())
    }
}
