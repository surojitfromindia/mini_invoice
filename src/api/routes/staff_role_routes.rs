use super::openapi_docs;
use crate::api::AuthorizedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{ActionStatusResponse, IntoServiceInput, PublicIdResponse};
use crate::api::dto::staff_role_dto::{
    CreateStaffRoleRequestDto, StaffRoleListItemResponseDto, StaffRoleListPageQueryDto,
    StaffRoleResponseDto, UpdateStaffRoleRequestDto,
};
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::service::staff_role_service::StaffRoleService;
use aide::axum::ApiRouter;
use axum::extract::Path;
use axum::extract::Query as AxumQuery;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Query as AxumExtraQuery;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(
        Router::new()
            .route(
                "/",
                post(create_staff_role_handler).get(list_staff_roles_page_handler),
            )
            .route(
                "/{public_id}",
                axum::routing::get(get_staff_role_handler)
                    .put(update_staff_role_handler)
                    .delete(delete_staff_role_handler),
            ),
    )
    .api_route_docs(
        "/",
        openapi_docs::method(
            "post",
            "staff_role",
            "createStaffRole",
            "Create staff role",
            |op| {
                op.input::<Json<CreateStaffRoleRequestDto>>();
                op.response::<201, ApiResponse<PublicIdResponse>>();
            },
        ),
    )
    .api_route_docs(
        "/",
        openapi_docs::method(
            "get",
            "staff_role",
            "listStaffRoles",
            "List staff roles",
            |op| {
                op.input::<AxumQuery<StaffRoleListPageQueryDto>>();
                op.response::<200, ApiResponse<PageListResult<StaffRoleListItemResponseDto>>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "get",
            "staff_role",
            "getStaffRole",
            "Get staff role",
            |op| {
                op.response::<200, ApiResponse<StaffRoleResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "put",
            "staff_role",
            "updateStaffRole",
            "Update staff role",
            |op| {
                op.input::<Json<UpdateStaffRoleRequestDto>>();
                op.response::<200, ApiResponse<StaffRoleResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "delete",
            "staff_role",
            "deleteStaffRole",
            "Delete staff role",
            |op| {
                op.response::<200, ApiResponse<ActionStatusResponse>>();
            },
        ),
    )
}

async fn create_staff_role_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<CreateStaffRoleRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRoleCreate)?;
    let role_public_id =
        StaffRoleService::create_staff_role(&ctx, payload.into_service_input()).await?;
    Ok(ApiResponse::success(
        PublicIdResponse {
            public_id: role_public_id,
        },
        "Staff role created",
        Some(StatusCode::CREATED),
    ))
}

async fn list_staff_roles_page_handler(
    authorized_ctx: AuthorizedContext,
    AxumExtraQuery(query): AxumExtraQuery<StaffRoleListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<StaffRoleListItemResponseDto>>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRoleRead)?;
    let result = StaffRoleService::list_staff_roles_page(&ctx, query.into_service_input()).await?;
    Ok(ApiResponse::success(
        StaffRoleListItemResponseDto::page_from_service_output(result),
        "Staff roles fetched",
        Some(StatusCode::OK),
    ))
}

async fn get_staff_role_handler(
    authorized_ctx: AuthorizedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<StaffRoleResponseDto>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRoleRead)?;
    let result = StaffRoleService::get_staff_role(&ctx, &public_id).await?;
    Ok(ApiResponse::success(
        StaffRoleResponseDto::from_service_output(result),
        "Staff role fetched",
        Some(StatusCode::OK),
    ))
}

async fn update_staff_role_handler(
    authorized_ctx: AuthorizedContext,
    Path(public_id): Path<String>,
    Json(payload): Json<UpdateStaffRoleRequestDto>,
) -> Result<ApiResponse<StaffRoleResponseDto>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRoleUpdate)?;
    let result =
        StaffRoleService::update_staff_role(&ctx, &public_id, payload.into_service_input()).await?;
    Ok(ApiResponse::success(
        StaffRoleResponseDto::from_service_output(result),
        "Staff role updated",
        Some(StatusCode::OK),
    ))
}

async fn delete_staff_role_handler(
    authorized_ctx: AuthorizedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRoleDelete)?;
    StaffRoleService::delete_staff_role(&ctx, &public_id).await?;
    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "deleted".to_string(),
        },
        "Staff role deleted",
        Some(StatusCode::OK),
    ))
}
