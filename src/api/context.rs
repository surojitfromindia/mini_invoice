use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::service_context::{AuthContext, ServiceContext};
use crate::utils::jwt_helpers::JwtHelpers;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use crate::service::actor_service::ActorService;

pub struct PublicContext(pub ServiceContext);

impl FromRequestParts<AppState> for PublicContext {
    type Rejection = AppError;

    fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = state.clone();
        async move {
            let ctx = ServiceContext::from_app_state(state);
            Ok(PublicContext(ctx))
        }
    }
}

pub struct AuthenticatedContext(pub ServiceContext);

impl FromRequestParts<AppState> for AuthenticatedContext {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = state.clone();
        let headers = parts.headers.clone();

        async move {
            let token = headers
                .get("authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .ok_or(AppError::Unauthorized)?;

            // verify access token.
            let jwt = JwtHelpers::new(&state.settings);
            let claims = jwt.verify_access_token(token)?;
            let public_id =claims.public_id;

            // get user actor.
            let user_actor = ActorService::get_user_actor(
                &state.primary_read_replica,
                &public_id,
            ).await?;

            // todo: later fetch user staff/ or app.
            // build auth context
            let auth_context = AuthContext{
                actor_id: user_actor.id,
                user_id: user_actor.user_id,
                client_app_id: None,
                organization_id: None,
            };

            // build service context.
            let mut ctx = ServiceContext::from_app_state(state);
            ctx.set_auth(auth_context);
            Ok(AuthenticatedContext(ctx))
        }
    }
}
