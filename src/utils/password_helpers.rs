use crate::errors::app_error::AppError;


use argon2::{
    password_hash::PasswordHasher,
    Algorithm, Argon2, Params, Version
};


pub struct PasswordHelpers {}



impl PasswordHelpers {
    pub fn hash_plain_password(plain_password: String)-> Result<String, AppError> {
        let argon2 = Argon2::new_with_secret(
            b"secret pepper",
            Algorithm::default(),
            Version::default(),
            Params::default()
        ).map_err(|x|{
            AppError::InternalServerError(x.to_string())
        })?;
        let password_hash = argon2
            .hash_password(plain_password.as_bytes())
            .map_err(|e| AppError::InternalServerError(e.to_string()))?
            .to_string();

        Ok(password_hash)
    }
}