use crate::errors::error_meta::ErrorMeta;
use crate::errors::user_credential_service_errors::UserCredentialServiceError;
use crate::errors::user_service_errors::UserServiceError;
use sea_orm::{DbErr, TransactionError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    User(UserServiceError),
    UserCredential(UserCredentialServiceError),
    Unauthorized,
    DatabaseError(DbErr),
    InternalServerError(String),
}

impl AppError {
    pub fn get_meta(&self) -> ErrorMeta {
        match self {
            AppError::User(data) => data.meta(),
            AppError::UserCredential(data) => data.meta(),
            AppError::Unauthorized => ErrorMeta {
                code: "000.000.0001",
                message: "Invalid email or password".to_string(),
                http_code: HttpErrorCode::Unauthorized,
            },
            AppError::DatabaseError(error) => {
                match error {
                    DbErr::Query(sea_orm::RuntimeErr::SqlxError(er)) => {
                        let db_error = er.as_database_error().and_then(|e| e.code());
                        match db_error.as_deref() {
                            Some("23505") => ErrorMeta {
                                // PostgreSQL unique violation
                                code: "100.001.001",
                                message: "Record already exists".to_string(),
                                http_code: HttpErrorCode::Conflict,
                            },
                            Some("1062") => ErrorMeta {
                                // MySQL duplicate entry
                                code: "100.001.001",
                                message: "Record already exists".to_string(),
                                http_code: HttpErrorCode::Conflict,
                            },
                            Some("2067") => ErrorMeta {
                                // SQLite unique violation
                                code: "100.001.001",
                                message: "Record already exists".to_string(),
                                http_code: HttpErrorCode::Conflict,
                            },
                            _ => {
                                let message = er
                                    .as_database_error()
                                    .map(|e| e.message())
                                    .unwrap_or("Database error");
                                ErrorMeta {
                                    code: "100.000.000",
                                    message: message.to_string(),
                                    http_code: HttpErrorCode::InternalServerError,
                                }
                            }
                        }
                    }
                    other => ErrorMeta {
                        code: "100.000.500",
                        message: other.to_string(),
                        http_code: HttpErrorCode::InternalServerError,
                    },
                }
            }
            AppError::InternalServerError(error_message) => ErrorMeta {
                code: "100.000.000",
                message: error_message.clone(),
                http_code: HttpErrorCode::InternalServerError,
            },
        }
    }
}

pub enum HttpErrorCode {
    NotFound,
    Conflict,
    InternalServerError,
    Unauthorized,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.get_meta().message)
    }
}

impl From<UserCredentialServiceError> for AppError {
    fn from(err: UserCredentialServiceError) -> Self {
        AppError::UserCredential(err)
    }
}

impl From<UserServiceError> for AppError {
    fn from(err: UserServiceError) -> Self {
        AppError::User(err)
    }
}

impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<TransactionError<DbErr>> for AppError {
    fn from(err: TransactionError<DbErr>) -> Self {
        match err {
            TransactionError::Connection(conn_err) => AppError::DatabaseError(conn_err),
            TransactionError::Transaction(txn_err) => AppError::DatabaseError(txn_err),
        }
    }
}

impl From<TransactionError<AppError>> for AppError {
    fn from(err: TransactionError<AppError>) -> Self {
        err.into()
    }
}
