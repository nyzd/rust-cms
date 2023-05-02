use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub action: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        super::role_permission::Relation::Role.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::role_permission::Relation::Permission.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
