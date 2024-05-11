use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
        ALTER TABLE chats
        ADD COLUMN femboy_time TIME(0) NOT NULL DEFAULT '12:00 MSK';

        ALTER TABLE players
        ADD COLUMN femboy_wins INT8 NOT NULL DEFAULT 0;
        "#,
        )
        .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
        ALTER TABLE chats
        DROP COLUMN femboy_time;

        ALTER TABLE players
        DROP COLUMN femboy_wins;
        "#,
        )
        .await?;
        Ok(())
    }
}
