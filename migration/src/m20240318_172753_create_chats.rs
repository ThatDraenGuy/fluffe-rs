use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
        CREATE TABLE chats (
            id SERIAL8 PRIMARY KEY,
            telegram_id INT8 UNIQUE NOT NULL
        );
        CREATE TABLE players (
            id SERIAL8 PRIMARY KEY,
            user_id INT8 NOT NULL REFERENCES users(id),
            chat_id INT8 NOT NULL REFERENCES chats(id),
            coins INT8 NOT NULL DEFAULT 0,
            pets_received INT8 NOT NULL DEFAULT 0,
            pets_given INT8 NOT NULL DEFAULT 0,
            CONSTRAINT players_logical_key UNIQUE (user_id, chat_id)
        );
        "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
        DROP TABLE players;
        DROP TABLE chats;
        "#,
        )
        .await?;
        Ok(())
    }
}
