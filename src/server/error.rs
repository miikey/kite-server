use std::fmt;

use actix_http::error::BlockingError;
use actix_http::http::StatusCode;
use actix_http::ResponseBuilder;
use actix_web::{error::ResponseError, HttpResponse};
use diesel::r2d2::PoolError;
use num_traits::cast::ToPrimitive;
use serde::export::Formatter;

use crate::user::error::*;

// Setting custom error
// See: https://actix.rs/docs/errors/
// fmt::Display
// See: https://doc.rust-lang.org/std/fmt/trait.Display.html


#[derive(Debug)]
pub enum ServerError{
    User(UserError),
    Pool(PoolError),
    Block,
}



impl From<UserError> for ServerError {
    fn from(user_error: UserError) -> Self {
        ServerError::User(user_error)
    }
}

impl From<PoolError> for ServerError {
    fn from(pool_error: PoolError) -> Self {
        ServerError::Pool(pool_error)
    }
}


impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServerError::User(user_error) => match user_error {
                UserError::OpError(e) => {
                    StatusCode::from_u16(e.to_u16().unwrap()).unwrap()
                },
                UserError::DBError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                _ => {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            },
            _ => {
                StatusCode::INTERNAL_SERVER_ERROR
            },
        }
    }
    fn error_response(&self) -> HttpResponse {
        ResponseBuilder::new(self.status_code()).body("")
    }
}


impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.status_code())
    }
}

// TODO: ServerError和connection error应该是全局的 error, 微信错误和用户错误放在 user 模块里面

