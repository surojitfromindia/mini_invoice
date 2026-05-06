use crate::errors::app_error::AppError;

pub struct AuthService {}



impl AuthService {
    async fn login_with_password(email: String, password: String) -> Result<bool, AppError> {
        
        Ok(true)
    }
    
    async fn login_with_google(){
        
    }
    
    
    async fn login_with_microsoft(){
        
    }
    
}