use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "role_permission")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub role_id: i32,
    #[sea_orm(primary_key)]
    pub permission_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Role,
    Permission,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Role => Entity::belongs_to(super::role::Entity)
                .from(Column::RoleId)
                .to(super::role::Column::Id)
                .into(),
            Self::Permission => Entity::belongs_to(super::permission::Entity)
                .from(Column::PermissionId)
                .to(super::permission::Column::Id)
                .into(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}
