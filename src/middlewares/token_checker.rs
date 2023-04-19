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

#[async_trait]
impl TokenChecker for TokenValidator {
    async fn get_user_id(&self, request_token: &str) -> Option<u32> {
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

        Some(token.user_id as u32)
    }
}
