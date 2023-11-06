use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "servers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub actual_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::User")]
    User,
}

impl Related<super::User> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Entity {
    pub fn find_by_actual_id(actual_id: String) -> Select<Entity> {
        Entity::find().filter(Column::ActualId.eq(actual_id))
    }
}

impl ActiveModelBehavior for ActiveModel {}
