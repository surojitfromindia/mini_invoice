use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::service::actor_service::ActorService;
use crate::service::service_context::{AuthContext, RequestContext, ServiceContext};
use crate::service::staff_service::StaffService;
use crate::utils::jwt_helpers::JwtHelpers;
use axum::extract::FromRequestParts;
use axum::http::header::HeaderMap;
use axum::http::request::Parts;

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

fn get_request_timezone(headers: &HeaderMap) -> String {
    headers
        .get("x-timezone")
        .or_else(|| headers.get("x-request-timezone"))
        .or_else(|| headers.get("timezone"))
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_owned())
        .unwrap_or_else(|| "UTC".to_owned())
}

fn build_request_context(headers: &HeaderMap) -> RequestContext {
    RequestContext {
        request_timezone: get_request_timezone(headers),
    }
}

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
            let public_id = claims.public_id;

            // get user actor.
            let user_actor =
                ActorService::get_user_actor(&state.primary_read_replica, &public_id).await?;
            let organization_id = match (user_actor.user_id, claims.organization_public_id) {
                (Some(user_id), Some(organization_public_id)) => {
                    let membership = StaffService::get_organization_for_user(
                        &ServiceContext::from_app_state(state.clone()),
                        user_id,
                        &organization_public_id,
                    )
                    .await
                    .map_err(|error| match error {
                        AppError::Staff(StaffServiceError::NotFound) => AppError::Unauthorized,
                        other => other,
                    })?;
                    Some(membership.id)
                }
                _ => None,
            };

            // build auth context
            let auth_context = AuthContext {
                actor_id: user_actor.id,
                user_id: user_actor.user_id,
                client_app_id: None,
                organization_id,
            };

            // build service context.
            let mut ctx = ServiceContext::from_app_state(state);
            ctx.set_auth(auth_context);
            ctx.set_request_context(build_request_context(&headers));

            Ok(AuthenticatedContext(ctx))
        }
    }
}
