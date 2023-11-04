use sea_orm::{entity::prelude::*, Related};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub discord_id: String,
    pub server_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Server",
        from = "Column::ServerId",
        to = "super::server::Column::Id"
    )]
    Server,
    #[sea_orm(has_one = "super::Femboy")]
    Femboy,
}

impl Related<super::Server> for Entity {
    fn to() -> RelationDef {
        Relation::Server.def()
    }
}

impl Related<super::Femboy> for Entity {
    fn to() -> RelationDef {
        Relation::Femboy.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
