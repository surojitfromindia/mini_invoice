use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use crate::app_state::AppState;
use crate::entity::{ActorPrimaryId, ClientAppPrimaryId, OrganizationPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
#[derive(Clone)]
pub struct ServiceContext {
    pub app_state: AppState,
    pub auth: Option<AuthContext>,
}

impl ServiceContext {

    pub fn from_app_state(app_state: AppState)->Self{
        Self {
            app_state,
            auth : None,
        }
    }
    pub fn get_actor_id(&self) -> Result<ActorPrimaryId, AppError> {
        if let Some(org_auth) = &self.auth {
            return Ok(org_auth.actor_id);
        }
        Err(AppError::ActorIdNotFound)
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

