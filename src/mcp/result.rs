use crate::errors::app_error::AppError;
use rmcp::model::CallToolResult;
use serde::Serialize;
use serde_json::json;

pub(super) fn success<T: Serialize>(value: T) -> CallToolResult {
    let structured_content = serde_json::to_value(value).unwrap_or_else(|error| {
        json!({
            "success": false,
            "message": "Failed to serialize tool response",
            "error": error.to_string(),
        })
    });

    CallToolResult::structured(structured_content)
}

pub(super) fn app_error(error: AppError) -> CallToolResult {
    let meta = error.meta();

    CallToolResult::structured_error(json!({
        "success": false,
        "message": meta.message,
        "error": {
            "code": meta.code,
            "message": meta.message,
        },
        "httpStatus": meta.http_code.as_status().as_u16(),
    }))
}

pub(super) fn invalid_arguments(error: impl ToString) -> CallToolResult {
    CallToolResult::structured_error(json!({
        "success": false,
        "message": "Invalid tool arguments",
        "error": {
            "code": "MCP_INVALID_ARGUMENTS",
            "message": error.to_string(),
        },
        "httpStatus": 400,
    }))
}

pub(super) fn into_tool_result<T: Serialize>(result: Result<T, AppError>) -> CallToolResult {
    result.map(success).unwrap_or_else(app_error)
}
