use crate::errors::error_meta::ErrorMeta;
use crate::errors::user_service_errors::UserServiceError;

#[derive(Debug)]
pub enum AppError {
    User(UserServiceError),
    Internal,
    DatabaseError(sea_orm::DbErr),
}

impl AppError {
    pub fn get_meta(self) -> ErrorMeta {
        match self {
            AppError::User(data) => data.meta(),
            AppError::Internal => ErrorMeta {
                code: "101.000.000",
                message: "",
                http_code: HttpErrorCode::InternalServerError,
            },
            AppError::DatabaseError(error) => ErrorMeta {
                code: "100.000.000",
                message: "Database error",
                http_code: HttpErrorCode::InternalServerError,
            },
        }
    }
}

pub enum HttpErrorCode {
    NotFound,
    Conflict,
    InternalServerError,
}

impl From<UserServiceError> for AppError {
    fn from(err: UserServiceError) -> Self {
        AppError::User(err)
    }
}
