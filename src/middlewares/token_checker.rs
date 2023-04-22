use async_trait::async_trait;
use auth::token::{TokenChecker, TokenGenerator};
use entity::token::{self, Entity as TokenModel};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

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


struct AuthResult {
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

        Some(AuthResult {
            user_id: token.user_id as u32,
            permissions: vec![]
        })
    }
}
