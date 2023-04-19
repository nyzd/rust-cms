#[cfg(test)]
mod tests {
    use crate::token::token_middleware::TokenChecker;
    use crate::token::{TokenAuth, TokenGenerator};
    use actix_web::dev::Service;
    use actix_web::http::{header};
    use actix_web::test::{self, TestRequest};
    use actix_web::App;

    #[derive(Default, Clone)]
    struct FindToken;

    impl TokenChecker for FindToken {
        fn check_token(&self, _request_token: &str) -> bool {
            true
        }
    }

    #[actix_web::test]
    async fn test_token_generator() {
        let source: Vec<u8> = vec![1, 2, 3];

        let mut token_generator = TokenGenerator::new(&source);

        token_generator.generate();

        assert_eq!(token_generator.get_result().unwrap().len(), 64);
    }

    #[actix_web::test]
    async fn test_token_middleware() {
        let token_auth = TokenAuth::new(FindToken {});

        let app = test::init_service(App::new().wrap(token_auth)).await;

        let bad_req = TestRequest::get().to_request();

        let res = app.call(bad_req).await;

        assert!(res.is_err());

        let good_req = TestRequest::get().append_header((header::AUTHORIZATION, "secret-token")).to_request();

        let res = app.call(good_req).await;

        assert!(res.is_ok());
    }
}
