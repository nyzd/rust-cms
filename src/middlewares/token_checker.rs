use async_trait::async_trait;
use auth::token::{TokenChecker, TokenGenerator};
use entity::token::{self, Entity as TokenModel};
use entity::role::{self, Entity as RoleModel};
use entity::permission::{self, Entity as PermissionModel};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

#[derive(Clone)]
pub struct TokenValidator {
    db_connection: DatabaseConnection,
}

impl TokenValidator {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self {
            db_connection: conn,
        }
    }
}

#[derive(Clone)]
pub struct AuthResult {
    user_id: u32,
    permissions: Vec<String>,
}

#[async_trait]
impl TokenChecker<AuthResult> for TokenValidator {
    async fn get_user_id(&self, request_token: &str) -> Option<AuthResult> {
        let token_bytes = request_token.bytes().collect::<Vec<u8>>();

        // Hash the request token
        let mut token_generator = TokenGenerator::new(&token_bytes);
        token_generator.generate();

        let Ok(Some(token)) = TokenModel::find()
            .filter(token::Column::TokenHash.eq(token_generator.get_result()))
            .one(&self.db_connection)
            .await else {
                return None;
            };

        // Now get the permissions that user have
        // First we need to get the roles user have 
        let Ok(roles) = RoleModel::find()
            .filter(role::Column::UserId.eq(token.user_id))
            .find_also_related(PermissionModel)
            .all(&self.db_connection)
            .await else {
                return None;
            };

       let permissions = roles.into_iter()
           .map(|(role, permission)| permission.unwrap().action)
           .collect::<Vec<String>>();

        Some(AuthResult {
            user_id: token.user_id as u32,
            permissions,
        })
    }
}
