use crate::app_state::AppState;
use crate::entity::{ActorPrimaryId, ClientAppPrimaryId, OrganizationPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
#[derive(Clone)]
pub struct ServiceContext {
    pub app_state: AppState,
    auth: Option<AuthContext>,
    request_context: Option<RequestContext>
}

impl ServiceContext {
    pub fn from_app_state(app_state: AppState) -> Self {
        Self {
            app_state,
            auth: None,
            request_context: None
        }
    }

    pub fn get_actor_id(&self) -> Result<ActorPrimaryId, AppError> {
        if let Some(auth) = &self.auth {
            return Ok(auth.actor_id);
        }
        Err(AppError::ActorIdNotFound)
    }

    pub fn get_user_id(&self) -> Result<UserPrimaryId, AppError> {
        if let Some(auth) = &self.auth {
            if let Some(user_id) = auth.user_id {
                return Ok(user_id);
            }
        }
        Err(AppError::UserIdNotFound)
    }

    pub fn set_auth(&mut self, auth: AuthContext) {
        self.auth = Some(auth);
    }
    
    pub fn set_request_context(&mut self, request_context: RequestContext) {
        self.request_context = Some(request_context);
    }
    
}

#[derive(Clone)]
pub struct RequestContext {
    pub request_timezone: String,
}

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub actor_id: ActorPrimaryId,
    pub user_id: Option<UserPrimaryId>,
    pub client_app_id: Option<ClientAppPrimaryId>,
    pub organization_id: Option<OrganizationPrimaryId>,
}
