use std::future::Future;
use std::sync::Arc;

use crate::api::dto::auth_dto::{LoginRequestDto, RefreshTokenRequestDto};
use crate::api::dto::branch_dto::{BranchListPageQueryDto, CreateBranchRequestDto};
use crate::api::dto::coa_dto::ChartOfAccountsQueryDto;
use crate::api::dto::item_dto::{CreateItemRequestDto, ItemListPageQueryDto};
use crate::api::dto::organization_dto::CreateOrganizationRequestDto;
use crate::api::dto::staff_dto::{
    AcceptStaffInvitationRequestDto, CreateStaffInvitationRequestDto,
    ResendStaffInvitationRequestDto, RevokeStaffInvitationRequestDto,
};
use crate::api::dto::staff_role_dto::CreateStaffRoleRequestDto;
use crate::api::dto::unit_dto::{CreateUnitRequestDto, UnitListPageQueryDto};
use crate::app_state::AppState;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, JsonObject, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo, Tool, ToolAnnotations,
};
use rmcp::service::{MaybeSendFuture, RequestContext};
use rmcp::{ErrorData as McpError, RoleServer, ServerHandler};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::handlers;
use super::result;

#[derive(Debug, Clone)]
pub(super) struct MiniInvoiceMcpServer {
    pub(super) app_state: AppState,
}

impl MiniInvoiceMcpServer {
    pub(super) fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

impl ServerHandler for MiniInvoiceMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Use Authorization: Bearer <accessToken> for authenticated tools. User account creation is intentionally not exposed over MCP.")
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, McpError>> + MaybeSendFuture + '_ {
        async move { Ok(ListToolsResult::with_all_items(tool_definitions())) }
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        tool_definitions()
            .into_iter()
            .find(|tool| tool.name == name)
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, McpError>> + MaybeSendFuture + '_ {
        async move {
            let result = match request.name.as_ref() {
                "health_check" => {
                    result::success(json!({ "status": "ok", "message": "I am fine!" }))
                }
                "auth_login" => {
                    handlers::auth_login(
                        &self.app_state,
                        &context,
                        parse_arguments::<LoginRequestDto>(request.arguments),
                    )
                    .await
                }
                "auth_refresh_token" => {
                    handlers::auth_refresh_token(
                        &self.app_state,
                        &context,
                        parse_arguments::<RefreshTokenRequestDto>(request.arguments),
                    )
                    .await
                }
                "auth_logout" => handlers::auth_logout(&self.app_state, &context).await,
                "user_get_current" => handlers::user_get_current(&self.app_state, &context).await,
                "organization_create" => {
                    handlers::organization_create(
                        &self.app_state,
                        &context,
                        parse_arguments::<CreateOrganizationRequestDto>(request.arguments),
                    )
                    .await
                }
                "chart_of_accounts_get" => {
                    handlers::chart_of_accounts_get(
                        &self.app_state,
                        &context,
                        parse_arguments::<ChartOfAccountsQueryDto>(request.arguments),
                    )
                    .await
                }
                "branch_create" => {
                    handlers::branch_create(
                        &self.app_state,
                        &context,
                        parse_arguments::<CreateBranchRequestDto>(request.arguments),
                    )
                    .await
                }
                "branch_list" => {
                    handlers::branch_list(
                        &self.app_state,
                        &context,
                        parse_arguments::<BranchListPageQueryDto>(request.arguments),
                    )
                    .await
                }
                "item_create" => {
                    handlers::item_create(
                        &self.app_state,
                        &context,
                        parse_arguments::<CreateItemRequestDto>(request.arguments),
                    )
                    .await
                }
                "item_list" => {
                    handlers::item_list(
                        &self.app_state,
                        &context,
                        parse_arguments::<ItemListPageQueryDto>(request.arguments),
                    )
                    .await
                }
                "unit_create" => {
                    handlers::unit_create(
                        &self.app_state,
                        &context,
                        parse_arguments::<CreateUnitRequestDto>(request.arguments),
                    )
                    .await
                }
                "unit_list" => {
                    handlers::unit_list(
                        &self.app_state,
                        &context,
                        parse_arguments::<UnitListPageQueryDto>(request.arguments),
                    )
                    .await
                }
                "staff_role_create" => {
                    handlers::staff_role_create(
                        &self.app_state,
                        &context,
                        parse_arguments::<CreateStaffRoleRequestDto>(request.arguments),
                    )
                    .await
                }
                "staff_invitation_create" => {
                    handlers::staff_invitation_create(
                        &self.app_state,
                        &context,
                        parse_arguments::<CreateStaffInvitationRequestDto>(request.arguments),
                    )
                    .await
                }
                "staff_invitation_accept" => {
                    handlers::staff_invitation_accept(
                        &self.app_state,
                        &context,
                        parse_arguments::<AcceptStaffInvitationRequestDto>(request.arguments),
                    )
                    .await
                }
                "staff_invitation_resend" => {
                    handlers::staff_invitation_resend(
                        &self.app_state,
                        &context,
                        parse_arguments::<ResendStaffInvitationRequestDto>(request.arguments),
                    )
                    .await
                }
                "staff_invitation_revoke" => {
                    handlers::staff_invitation_revoke(
                        &self.app_state,
                        &context,
                        parse_arguments::<RevokeStaffInvitationRequestDto>(request.arguments),
                    )
                    .await
                }
                _ => {
                    return Err(McpError::invalid_params(
                        format!("Unknown tool: {}", request.name),
                        None,
                    ));
                }
            };

            Ok(result)
        }
    }
}

fn parse_arguments<T: DeserializeOwned>(arguments: Option<JsonObject>) -> Result<T, String> {
    serde_json::from_value(Value::Object(arguments.unwrap_or_default()))
        .map_err(|error| error.to_string())
}

fn tool_definitions() -> Vec<Tool> {
    vec![
        read_tool(
            "health_check",
            "Health Check",
            "Check backend health.",
            empty_schema(),
        ),
        read_tool(
            "auth_login",
            "Login",
            "Login with email and password.",
            schema_for::<LoginRequestDto>(),
        ),
        read_tool(
            "auth_refresh_token",
            "Refresh Token",
            "Refresh access and refresh tokens.",
            schema_for::<RefreshTokenRequestDto>(),
        ),
        write_tool(
            "auth_logout",
            "Logout",
            "Logout the current authenticated user.",
            empty_schema(),
        ),
        read_tool(
            "user_get_current",
            "Get Current User",
            "Fetch the current authenticated user profile.",
            empty_schema(),
        ),
        write_tool(
            "organization_create",
            "Create Organization",
            "Create an organization for the authenticated user.",
            schema_for::<CreateOrganizationRequestDto>(),
        ),
        read_tool(
            "chart_of_accounts_get",
            "Get Chart Of Accounts",
            "Fetch the default chart of accounts in tree or flat view.",
            schema_for::<ChartOfAccountsQueryDto>(),
        ),
        write_tool(
            "branch_create",
            "Create Branch",
            "Create an organization branch.",
            schema_for::<CreateBranchRequestDto>(),
        ),
        read_tool(
            "branch_list",
            "List Branches",
            "List organization branches with pagination and filters.",
            schema_for::<BranchListPageQueryDto>(),
        ),
        write_tool(
            "item_create",
            "Create Item",
            "Create an inventory or service item.",
            schema_for::<CreateItemRequestDto>(),
        ),
        read_tool(
            "item_list",
            "List Items",
            "List organization items with pagination and filters.",
            schema_for::<ItemListPageQueryDto>(),
        ),
        write_tool(
            "unit_create",
            "Create Unit",
            "Create a custom organization unit.",
            schema_for::<CreateUnitRequestDto>(),
        ),
        read_tool(
            "unit_list",
            "List Units",
            "List organization units with pagination and filters.",
            schema_for::<UnitListPageQueryDto>(),
        ),
        write_tool(
            "staff_role_create",
            "Create Staff Role",
            "Create a staff role with permission codes.",
            schema_for::<CreateStaffRoleRequestDto>(),
        ),
        write_tool(
            "staff_invitation_create",
            "Create Staff Invitation",
            "Invite a staff member to the current organization.",
            schema_for::<CreateStaffInvitationRequestDto>(),
        ),
        write_tool(
            "staff_invitation_accept",
            "Accept Staff Invitation",
            "Accept a staff invitation using the invitation token and password.",
            schema_for::<AcceptStaffInvitationRequestDto>(),
        ),
        write_tool(
            "staff_invitation_resend",
            "Resend Staff Invitation",
            "Resend an existing staff invitation.",
            schema_for::<ResendStaffInvitationRequestDto>(),
        ),
        destructive_tool(
            "staff_invitation_revoke",
            "Revoke Staff Invitation",
            "Revoke an existing staff invitation.",
            schema_for::<RevokeStaffInvitationRequestDto>(),
        ),
    ]
}

fn read_tool(
    name: &'static str,
    title: &'static str,
    description: &'static str,
    input_schema: Arc<JsonObject>,
) -> Tool {
    tool(name, title, description, input_schema, true, false)
}

fn write_tool(
    name: &'static str,
    title: &'static str,
    description: &'static str,
    input_schema: Arc<JsonObject>,
) -> Tool {
    tool(name, title, description, input_schema, false, false)
}

fn destructive_tool(
    name: &'static str,
    title: &'static str,
    description: &'static str,
    input_schema: Arc<JsonObject>,
) -> Tool {
    tool(name, title, description, input_schema, false, true)
}

fn tool(
    name: &'static str,
    title: &'static str,
    description: &'static str,
    input_schema: Arc<JsonObject>,
    read_only: bool,
    destructive: bool,
) -> Tool {
    Tool::new(name, description, input_schema)
        .with_title(title)
        .with_annotations(
            ToolAnnotations::with_title(title)
                .read_only(read_only)
                .destructive(destructive),
        )
}

fn schema_for<T: JsonSchema>() -> Arc<JsonObject> {
    let schema = schemars::SchemaGenerator::default().into_root_schema_for::<T>();
    let value = serde_json::to_value(schema).expect("failed to serialize JSON schema");
    let object = value
        .as_object()
        .expect("root JSON schema must be an object")
        .clone();

    Arc::new(object)
}

fn empty_schema() -> Arc<JsonObject> {
    Arc::new(
        json!({
            "type": "object",
            "properties": {},
        })
        .as_object()
        .expect("empty schema must be an object")
        .clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_catalog_excludes_user_account_creation() {
        let tool_names = tool_definitions()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect::<Vec<_>>();

        assert!(!tool_names.contains(&"user_create_account".to_string()));
        assert!(!tool_names.contains(&"create_user_account".to_string()));
        assert!(!tool_names.contains(&"user_account_create".to_string()));
        assert!(tool_names.contains(&"user_get_current".to_string()));
        assert!(tool_names.contains(&"unit_create".to_string()));
        assert!(tool_names.contains(&"unit_list".to_string()));
    }
}
