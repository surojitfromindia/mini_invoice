// Central error code registry.
// Keep every public-facing API error code here so new modules do not invent
// overlapping values independently.

// Authentication and authorization
pub const INVALID_CREDENTIALS: &str = "000.000.0001";
pub const UNAUTHORIZED: &str = "000.000.0002";
pub const FORBIDDEN: &str = "000.000.0003";

// JWT and token lifecycle
pub const JWT_INVALID_TOKEN: &str = "001.000.0001";
pub const JWT_CANNOT_GENERATE_TOKEN: &str = "001.000.0002";
pub const JWT_INVALID_TOKEN_TYPE: &str = "001.000.0003";

// Request/service context invariants
pub const ACTOR_ID_NOT_FOUND: &str = "001.000.001";
pub const USER_ID_NOT_FOUND: &str = "001.000.002";
pub const ORGANIZATION_ID_NOT_FOUND: &str = "001.000.003";

// User domain
pub const USER_EMAIL_ALREADY_EXISTS: &str = "100.000.0001";
pub const USER_NOT_FOUND: &str = "100.000.0002";

// User credential domain
pub const USER_CREDENTIAL_NOT_FOUND: &str = "101.000.0001";

// Organization domain
pub const ORGANIZATION_NOT_FOUND: &str = "102.000.0002";
pub const ORGANIZATION_BRANCH_NOT_FOUND: &str = "102.000.0003";
pub const ORGANIZATION_PRIMARY_BRANCH_NOT_CONFIGURED: &str = "102.000.0004";

// Staff domain
pub const STAFF_NOT_FOUND: &str = "103.000.0002";
pub const STAFF_INVITATION_NOT_FOUND: &str = "103.000.0003";
pub const STAFF_INVITATION_EXPIRED: &str = "103.000.0004";
pub const STAFF_INVITATION_ALREADY_USED: &str = "103.000.0005";
pub const STAFF_ROLE_NOT_FOUND: &str = "103.000.0006";
pub const STAFF_INVALID_PERMISSION: &str = "103.000.0007";

// Database and unexpected internal failures
pub const DATABASE_DUPLICATE_RECORD: &str = "900.001.0001";
pub const DATABASE_OPERATION_FAILED: &str = "900.001.0002";
pub const INTERNAL_SERVER_ERROR: &str = "900.000.0001";
