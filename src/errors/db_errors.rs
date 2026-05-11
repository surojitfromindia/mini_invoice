use sea_orm::{DbErr, TransactionError};
use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};
impl From<DbErr> for AppError {
    fn from(err: DbErr) -> Self {
        AppError::Database(err)
    }
}

impl From<TransactionError<DbErr>> for AppError {
    fn from(err: TransactionError<DbErr>) -> Self {
        match err {
            TransactionError::Connection(conn_err) => AppError::Database(conn_err),
            TransactionError::Transaction(txn_err) => AppError::Database(txn_err),
        }
    }
}




impl From<TransactionError<AppError>> for AppError {
    fn from(err: TransactionError<AppError>) -> Self {
        match err {
            TransactionError::Connection(db_err) => {
                AppError::Database(db_err)
            }

            TransactionError::Transaction(app_err) => {
                app_err
            }
        }
    }
}

impl ErrorMetadata for DbErr {
    fn meta(&self) -> ErrorMeta {
        match self {
            DbErr::Query(sea_orm::RuntimeErr::SqlxError(er)) => {
                let db_error = er.as_database_error().and_then(|e| e.code());
                match db_error.as_deref() {
                    Some("23505") => ErrorMeta {
                        // PostgreSQL unique violation
                        code: "100.001.001",
                        message: "Record already exists".into(),
                        http_code: HttpErrorCode::Conflict,
                    },
                    Some("1062") => ErrorMeta {
                        // MySQL duplicate entry
                        code: "100.001.001",
                        message: "Record already exists".into(),
                        http_code: HttpErrorCode::Conflict,
                    },
                    Some("2067") => ErrorMeta {
                        // SQLite unique violation
                        code: "100.001.001",
                        message: "Record already exists".into(),
                        http_code: HttpErrorCode::Conflict,
                    },
                    _ => {
                        let message = er
                            .as_database_error()
                            .map(|e| e.message())
                            .unwrap_or("Database error");
                        ErrorMeta {
                            code: "100.000.000",
                            message: message.into(),
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
}