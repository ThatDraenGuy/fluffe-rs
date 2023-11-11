use sea_orm::{entity::prelude::*, JoinType, QuerySelect};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "femboys")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i64,
    pub balance: i64,
    pub wins_num: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::User",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl Related<super::User> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Entity {
    pub fn find_by_server(server_actual_id: &str) -> Select<Self> {
        Entity::find()
            .inner_join(super::User)
            .join(JoinType::InnerJoin, super::user::Relation::Server.def())
            .filter(super::server::Column::ActualId.eq(server_actual_id))
    }

    pub fn find_by_user(server_actual_id: &str, user_discord_id: &str) -> Select<Self> {
        Self::find_by_server(server_actual_id)
            .filter(super::user::Column::DiscordId.eq(user_discord_id))
    }
}

impl ActiveModelBehavior for ActiveModel {}
