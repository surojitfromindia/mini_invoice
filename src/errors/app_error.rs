use crate::errors::error_meta::ErrorMeta;
use crate::errors::user_service_errors::UserServiceError;

#[derive(Debug)]
pub enum AppError {
    User(UserServiceError),
    DatabaseError(sea_orm::DbErr),
    InternalServerError(String),
}

impl AppError {
    pub fn get_meta(self) -> ErrorMeta {
        match self {
            AppError::User(data) => data.meta(),
            AppError::DatabaseError(error) => {
                match error {
                    sea_orm::DbErr::Query(sea_orm::RuntimeErr::SqlxError(er)) => {
                        let db_error = er.as_database_error().and_then(|e| e.code());
                        match db_error.as_deref() {
                            Some("23505") => ErrorMeta {
                                // PostgreSQL unique violation
                                code: "100.001.001",
                                message: "Record already exists",
                                http_code: HttpErrorCode::Conflict,
                            },
                            Some("1062") => ErrorMeta {
                                // MySQL duplicate entry
                                code: "100.001.001",
                                message: "Record already exists",
                                http_code: HttpErrorCode::Conflict,
                            },
                            Some("2067") => ErrorMeta {
                                // SQLite unique violation
                                code: "100.001.001",
                                message: "Record already exists",
                                http_code: HttpErrorCode::Conflict,
                            },
                            _ => ErrorMeta {
                                code: "100.000.000",
                                message: "Database error",
                                http_code: HttpErrorCode::InternalServerError,
                            },
                        }
                    }
                    _ => ErrorMeta {
                        code: "100.000.000",
                        message: "Database error",
                        http_code: HttpErrorCode::InternalServerError,
                    },
                }
            },
            AppError::InternalServerError(error_message)=>{
                ErrorMeta {
                    code: "100.000.000.00",
                    message: "",
                    http_code: HttpErrorCode::InternalServerError,
                }
            }
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

impl From<sea_orm::DbErr> for AppError {
    fn from(err: sea_orm::DbErr) -> Self {
        AppError::DatabaseError(err)
    }
}


