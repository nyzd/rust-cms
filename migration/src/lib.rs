pub use sea_orm_migration::prelude::*;

mod m20230418_101322_create_user_table;
mod m20230418_111519_create_post;
mod m20230418_121601_create_token;
mod m20230420_093824_create_email_verification;
mod m20230422_133556_create_role;
mod m20230422_135423_create_permission;
mod m20230422_140203_create_role_permissions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230418_101322_create_user_table::Migration),
            Box::new(m20230418_111519_create_post::Migration),
            Box::new(m20230418_121601_create_token::Migration),
            Box::new(m20230420_093824_create_email_verification::Migration),
            Box::new(m20230422_133556_create_role::Migration),
            Box::new(m20230422_135423_create_permission::Migration),
            Box::new(m20230422_140203_create_role_permissions::Migration),
        ]
    }
}
