// Stable client-facing messages for internal failures.
// The raw underlying error is still logged server-side through tracing.
pub const INTERNAL_SERVER_ERROR: &str = "Internal server error";
pub const DATABASE_OPERATION_FAILED: &str = "Database operation failed";
