use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{IntoServiceInput, PublicIdResponse};
use crate::api::dto::item_dto::{
    CreateItemRequestDto, ItemListItemResponseDto, ItemListPageQueryDto,
};
use crate::app_state::AppState;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::resolver::item_payload_resolver::ItemPayloadResolver;
use crate::service::item_service::ItemService;
use aide::axum::ApiRouter;
use aide::axum::routing::ApiMethodDocs;
use aide::generate::in_context;
use aide::openapi::{Operation, ReferenceOr, Responses, StatusCode as OpenApiStatusCode};
use aide::{OperationInput, OperationOutput};
use axum::extract::Query as AxumQuery;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Query as AxumExtraQuery;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(
        Router::new().route("/", post(create_item_handler).get(list_items_page_handler)),
    )
    .api_route_docs("/", ApiMethodDocs::new("post", create_item_operation()))
    .api_route_docs("/", ApiMethodDocs::new("get", list_items_operation()))
}

fn create_item_operation() -> Operation {
    item_operation("createItem", "Create item", |op| {
        op.input::<Json<CreateItemRequestDto>>();
        op.response::<201, ApiResponse<PublicIdResponse>>();
    })
}

fn list_items_operation() -> Operation {
    item_operation("listItems", "List items", |op| {
        op.input::<AxumQuery<ItemListPageQueryDto>>();
        op.response::<200, ApiResponse<PageListResult<ItemListItemResponseDto>>>();
    })
}

fn item_operation(id: &str, summary: &str, docs: impl FnOnce(&mut OperationDoc)) -> Operation {
    let mut operation = Operation {
        operation_id: Some(id.to_owned()),
        summary: Some(summary.to_owned()),
        tags: vec!["item".to_owned()],
        responses: Some(Responses::default()),
        ..Operation::default()
    };

    in_context(|ctx| {
        docs(&mut OperationDoc {
            ctx,
            operation: &mut operation,
        })
    });
    operation
}

struct OperationDoc<'a> {
    ctx: &'a mut aide::generate::GenContext,
    operation: &'a mut Operation,
}

impl OperationDoc<'_> {
    fn input<T: OperationInput>(&mut self) {
        T::operation_input(self.ctx, self.operation);
    }

    fn response<const STATUS: u16, T: OperationOutput>(&mut self) {
        if let Some(response) = T::operation_response(self.ctx, self.operation) {
            self.operation
                .responses
                .get_or_insert_with(Responses::default)
                .responses
                .insert(OpenApiStatusCode::Code(STATUS), ReferenceOr::Item(response));
        }
    }
}

async fn create_item_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateItemRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = ItemPayloadResolver::create_item(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into_resolution_input(),
    )
    .await?;
    let public_id = ItemService::create_item(&ctx, resolved_payload).await?;

    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Item created",
        Some(StatusCode::CREATED),
    ))
}

async fn list_items_page_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    AxumExtraQuery(query): AxumExtraQuery<ItemListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<ItemListItemResponseDto>>, AppError> {
    let result = ItemService::list_items_page(&ctx, query.into_service_input()).await?;

    Ok(ApiResponse::success(
        ItemListItemResponseDto::page_from_service_output(result),
        "Items fetched",
        Some(StatusCode::OK),
    ))
}
