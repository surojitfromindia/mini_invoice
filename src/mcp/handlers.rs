use crate::api::api_response::ApiResponse;
use crate::api::context::{authenticated_service_context, public_service_context};
use crate::api::dto::auth_dto::{AuthTokensResponseDto, LoginRequestDto, RefreshTokenRequestDto};
use crate::api::dto::branch_dto::{
    BranchListItemResponseDto, BranchListPageQueryDto, CreateBranchRequestDto,
};
use crate::api::dto::coa_dto::{ChartOfAccountsQueryDto, ChartOfAccountsResponseDto};
use crate::api::dto::common_dto::{ActionStatusResponse, IntoServiceInput, PublicIdResponse};
use crate::api::dto::item_dto::{
    CreateItemRequestDto, ItemListItemResponseDto, ItemListPageQueryDto,
};
use crate::api::dto::organization_dto::CreateOrganizationRequestDto;
use crate::api::dto::staff_dto::{
    AcceptStaffInvitationRequestDto, CreateStaffInvitationRequestDto,
    ResendStaffInvitationRequestDto, RevokeStaffInvitationRequestDto, StaffInvitationResponseDto,
};
use crate::api::dto::staff_role_dto::CreateStaffRoleRequestDto;
use crate::api::dto::unit_dto::{
    CreateUnitRequestDto, UnitListItemResponseDto, UnitListPageQueryDto,
};
use crate::api::dto::user_dto::CurrentUserResponseDto;
use crate::app_state::AppState;
use crate::auth::authorization_service::AuthorizationService;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::resolver::item_payload_resolver::ItemPayloadResolver;
use crate::resolver::public_id_resolver::PublicIdResolver;
use crate::resolver::staff_payload_resolver::{
    CreateStaffInvitationResolutionInput, StaffPayloadResolver,
};
use crate::service::auth_service::AuthService;
use crate::service::branch_service::BranchService;
use crate::service::coa_service::CoaService;
use crate::service::item_service::ItemService;
use crate::service::organization_service::OrganizationService;
use crate::service::staff_role_service::StaffRoleService;
use crate::service::staff_service::StaffService;
use crate::service::unit_service::UnitService;
use crate::service::user_service::UserService;
use axum::http::StatusCode;
use axum::http::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use rmcp::RoleServer;
use rmcp::model::CallToolResult;
use rmcp::service::RequestContext;
use std::env;

use super::result::{into_tool_result, invalid_arguments};

const MCP_REFRESH_TOKEN_HEADER: &str = "x-mini-invoice-refresh-token";
const MCP_REFRESH_TOKEN_ENV: &str = "MINI_INVOICE_MCP_REFRESH_TOKEN";

pub(super) async fn auth_login(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<LoginRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = public_service_context(app_state.clone(), request_headers(context)?);
        let (email, password) = payload.into_service_input();
        let tokens = AuthService::login_with_password(&ctx, email, password).await?;

        Ok::<_, AppError>(ApiResponse::success(
            AuthTokensResponseDto::from_service_output(tokens),
            "User logged-in",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn auth_refresh_token(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<RefreshTokenRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = public_service_context(app_state.clone(), request_headers(context)?);
        let tokens = AuthService::refresh_tokens(&ctx, payload.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            AuthTokensResponseDto::from_service_output(tokens),
            "Tokens refreshed",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn auth_logout(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
) -> CallToolResult {
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        AuthService::logout(&ctx).await?;

        Ok::<_, AppError>(ApiResponse::success(
            ActionStatusResponse {
                status: "logged_out".to_string(),
            },
            "User logged-out",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn user_get_current(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
) -> CallToolResult {
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let user = UserService::get_current_user_profile(&ctx).await?;

        Ok::<_, AppError>(ApiResponse::success(
            CurrentUserResponseDto::from_service_output(user),
            "Current user fetched",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn organization_create(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<CreateOrganizationRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let public_id =
            OrganizationService::create_organization(&ctx, payload.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            PublicIdResponse { public_id },
            "Organization created",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn chart_of_accounts_get(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    query: Result<ChartOfAccountsQueryDto, String>,
) -> CallToolResult {
    let query = match query {
        Ok(query) => query,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let view_mode = query.view.unwrap_or_default().into_service_input();
        let chart = CoaService::fetch_default_chart_of_accounts(&ctx, view_mode).await?;

        Ok::<_, AppError>(ApiResponse::success(
            ChartOfAccountsResponseDto::from_service_output(chart),
            "Chart of accounts fetched",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn branch_create(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<CreateBranchRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        AuthorizationService::require_all_permissions(&ctx, &[Permission::BranchCreate])?;
        let public_id = BranchService::create_branch(&ctx, payload.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            PublicIdResponse { public_id },
            "Branch created",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn branch_list(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    query: Result<BranchListPageQueryDto, String>,
) -> CallToolResult {
    let query = match query {
        Ok(query) => query,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let result = BranchService::list_branches_page(&ctx, query.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            BranchListItemResponseDto::page_from_service_output(result),
            "Branches fetched",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn item_create(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<CreateItemRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let organization_id = ctx.get_organization_id()?;
        let resolved_payload = ItemPayloadResolver::create_item(
            &ctx.app_state.primary_write_replica,
            organization_id,
            payload.into_resolution_input(),
        )
        .await?;
        let public_id = ItemService::create_item(&ctx, resolved_payload).await?;

        Ok::<_, AppError>(ApiResponse::success(
            PublicIdResponse { public_id },
            "Item created",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn item_list(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    query: Result<ItemListPageQueryDto, String>,
) -> CallToolResult {
    let query = match query {
        Ok(query) => query,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let result = ItemService::list_items_page(&ctx, query.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            ItemListItemResponseDto::page_from_service_output(result),
            "Items fetched",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn unit_create(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<CreateUnitRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let public_id = UnitService::create_unit(&ctx, payload.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            PublicIdResponse { public_id },
            "Unit created",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn unit_list(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    query: Result<UnitListPageQueryDto, String>,
) -> CallToolResult {
    let query = match query {
        Ok(query) => query,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        let result = UnitService::list_units_page(&ctx, query.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            UnitListItemResponseDto::page_from_service_output(result),
            "Units fetched",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn staff_role_create(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<CreateStaffRoleRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        AuthorizationService::require_permission(&ctx, Permission::StaffRoleCreate)?;
        let public_id =
            StaffRoleService::create_staff_role(&ctx, payload.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            PublicIdResponse { public_id },
            "Staff role created",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn staff_invitation_create(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<CreateStaffInvitationRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        AuthorizationService::require_any_permission(&ctx, &[Permission::StaffInvite])?;
        let organization_id = ctx.get_organization_id()?;
        let resolved_payload = StaffPayloadResolver::create_staff_invitation(
            &ctx.app_state.primary_write_replica,
            organization_id,
            CreateStaffInvitationResolutionInput {
                invitee_email: payload.invitee_email,
                invitee_first_name: payload.invitee_first_name,
                invitee_last_name: payload.invitee_last_name,
                role_public_id: payload.role_public_id,
                branch_public_ids: payload.branch_public_ids,
            },
        )
        .await?;
        let invitation = StaffService::create_staff_invitation(&ctx, resolved_payload).await?;

        Ok::<_, AppError>(ApiResponse::success(
            StaffInvitationResponseDto::from_service_output(invitation),
            "Staff invitation created",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn staff_invitation_accept(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<AcceptStaffInvitationRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = public_service_context(app_state.clone(), request_headers(context)?);
        StaffService::accept_staff_invitation(&ctx, payload.into_service_input()).await?;

        Ok::<_, AppError>(ApiResponse::success(
            ActionStatusResponse {
                status: "invitation_accepted".to_string(),
            },
            "Staff invitation accepted",
            Some(StatusCode::CREATED),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn staff_invitation_resend(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<ResendStaffInvitationRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        AuthorizationService::require_permission(&ctx, Permission::StaffInvitationResend)?;
        let organization_id = ctx.get_organization_id()?;
        let invitation_id = PublicIdResolver::staff_invitation_id(
            &ctx.app_state.primary_write_replica,
            organization_id,
            &payload.invitation_id,
        )
        .await?;
        let invitation = StaffService::resend_staff_invitation(&ctx, invitation_id).await?;

        Ok::<_, AppError>(ApiResponse::success(
            StaffInvitationResponseDto::from_service_output(invitation),
            "Staff invitation resent",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

pub(super) async fn staff_invitation_revoke(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
    payload: Result<RevokeStaffInvitationRequestDto, String>,
) -> CallToolResult {
    let payload = match payload {
        Ok(payload) => payload,
        Err(error) => return invalid_arguments(error),
    };
    let result = async {
        let ctx = authenticated_context(app_state, context).await?;
        AuthorizationService::require_permission(&ctx, Permission::StaffInvitationRevoke)?;
        let organization_id = ctx.get_organization_id()?;
        let invitation_id = PublicIdResolver::staff_invitation_id(
            &ctx.app_state.primary_write_replica,
            organization_id,
            &payload.invitation_id,
        )
        .await?;
        StaffService::revoke_staff_invitation(&ctx, invitation_id).await?;

        Ok::<_, AppError>(ApiResponse::success(
            ActionStatusResponse {
                status: "invitation_revoked".to_string(),
            },
            "Staff invitation revoked",
            Some(StatusCode::OK),
        ))
    }
    .await;

    into_tool_result(result)
}

async fn authenticated_context(
    app_state: &AppState,
    context: &RequestContext<RoleServer>,
) -> Result<crate::service::service_context::ServiceContext, AppError> {
    let headers = request_headers(context)?;
    match authenticated_service_context(app_state.clone(), headers).await {
        Ok(ctx) => Ok(ctx),
        Err(access_token_error) => {
            let Some(refresh_token) = mcp_refresh_token(headers)? else {
                return Err(access_token_error);
            };
            let public_ctx = public_service_context(app_state.clone(), headers);
            let access_token =
                AuthService::issue_access_token_from_refresh_token(&public_ctx, refresh_token)
                    .await?;
            let mut refreshed_headers = headers.clone();
            let authorization_value = HeaderValue::from_str(&format!("Bearer {access_token}"))
                .map_err(|_| {
                    AppError::InternalServer("Invalid generated access token".to_string())
                })?;
            refreshed_headers.insert(AUTHORIZATION, authorization_value);

            authenticated_service_context(app_state.clone(), &refreshed_headers).await
        }
    }
}

fn mcp_refresh_token(headers: &HeaderMap) -> Result<Option<String>, AppError> {
    if let Some(refresh_token) = headers.get(MCP_REFRESH_TOKEN_HEADER) {
        return refresh_token
            .to_str()
            .map(|token| Some(token.to_string()))
            .map_err(|_| AppError::InternalServer("Invalid MCP refresh token header".to_string()));
    }

    Ok(env::var(MCP_REFRESH_TOKEN_ENV)
        .ok()
        .filter(|token| !token.trim().is_empty()))
}

fn request_headers(context: &RequestContext<RoleServer>) -> Result<&HeaderMap, AppError> {
    context
        .extensions
        .get::<axum::http::request::Parts>()
        .map(|parts| &parts.headers)
        .ok_or_else(|| AppError::InternalServer("MCP HTTP request parts missing".to_string()))
}
