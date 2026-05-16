use crate::errors::app_error::AppError;

use crate::config::settings::Settings;

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};

pub struct PasswordHelpers {}

impl PasswordHelpers {
    pub fn hash_login_password(
        settings: &Settings,
        plain_password: &str,
    ) -> Result<String, AppError> {
        let argon2 = Self::init_argon(&settings.login_secret_pepper)?;
        let password_hash = argon2
            .hash_password(plain_password.as_bytes())
            .map_err(|e| AppError::InternalServer(e.to_string()))?
            .to_string();

        Ok(password_hash)
    }

    pub fn verify_login_password(
        settings: &Settings,
        plain_password: &str,
        password_hash: &str,
    ) -> Result<bool, AppError> {
        let argon2 = Self::init_argon(&settings.login_secret_pepper)?;
        let parsed_hash = PasswordHash::new(&password_hash)
            .map_err(|x| AppError::InternalServer(x.to_string()))?;
        let res = argon2
            .verify_password(plain_password.as_bytes(), &parsed_hash)
            .is_ok();
        Ok(res)
    }

    fn init_argon(pepper: &str) -> Result<Argon2<'_>, AppError> {
        Argon2::new_with_secret(
            pepper.as_bytes(),
            Algorithm::default(),
            Version::default(),
            Params::default(),
        )
        .map_err(|x| AppError::InternalServer(x.to_string()))
    }
}
