use sea_orm_migration::prelude::*;
use crate::m20230422_135423_create_permission::Permission;
use crate::m20230422_133556_create_role::Role;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RolePermission::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RolePermission::RoleId)
                            .integer()
                            .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-rolepermissions-role-id")
                            .from(RolePermission::Table, RolePermission::RoleId)
                            .to(Role::Table, Role::Id),
                    )
                    .col(
                        ColumnDef::new(RolePermission::PermissionId)
                            .integer()
                            .not_null()
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-rolepermissions-permission-id")
                            .from(RolePermission::Table, RolePermission::PermissionId)
                            .to(Permission::Table, Permission::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RolePermission::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum RolePermission {
    Table,
    RoleId,
    PermissionId
}
