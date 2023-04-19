use actix_utils::future::{ready, Ready};
use actix_web::http::header;
use actix_web::HttpMessage;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error,
};
use async_trait::async_trait;
use futures_util::future::LocalBoxFuture;

#[async_trait]
pub trait TokenChecker {
    /// This function will return option
    /// if the request token valid return
    /// Some with sized data to pass to the router
    /// otherwise retun None to response with status code 401
    /// Unauthorized
    ///
    /// This function returns the verifyed user ID
    async fn get_user_id(&self, request_token: &str) -> Option<u32>
    where
        Self: Sized;
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.

#[derive(Clone, Default)]
pub struct TokenAuth<F>(F);

impl<F> TokenAuth<F>
where
    F: TokenChecker,
{
    /// Construct `TokenAuth` middleware.
    pub fn new(finder: F) -> Self {
        Self(finder)
    }
}

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B, F> Transform<S, ServiceRequest> for TokenAuth<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
    S::Future: 'static,
    B: 'static,
    F: TokenChecker + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TokenAuthMiddleware<S, F>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TokenAuthMiddleware {
            service,
            token_finder: self.0.clone(),
        }))
    }
}

pub struct TokenAuthMiddleware<S, F> {
    service: S,
    token_finder: F,
}

impl<S, B, F> Service<ServiceRequest> for TokenAuthMiddleware<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + Clone + 'static,
    S::Future: 'static,
    F: TokenChecker + Clone + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let token_finder = self.token_finder.clone();

        Box::pin(async move {
            if let Some(token) = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|token| token.to_str().ok())
            {
                let token_data = token_finder.get_user_id(token).await;

                if let Some(data) = token_data {
                    req.extensions_mut().insert(data);
                    let res = service.call(req).await.unwrap();
                    return Ok(res);
                };
            }

            Err(ErrorUnauthorized("This Token is not valid"))
        })
    }
}
