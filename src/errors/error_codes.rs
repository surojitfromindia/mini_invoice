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
pub const USER_EMAIL_ALREADY_EXISTS: &str = "010.000.0001";
pub const USER_NOT_FOUND: &str = "010.000.0002";
pub const USER_CREDENTIAL_NOT_FOUND: &str = "010.000.0003";

// Organization domain
pub const ORGANIZATION_NOT_FOUND: &str = "100.000.0002";
pub const BRANCH_NOT_FOUND: &str = "101.000.0003";
pub const PRIMARY_BRANCH_NOT_CONFIGURED: &str = "101.000.0004";

// Staff domain
pub const STAFF_NOT_FOUND: &str = "200.000.0002";
pub const STAFF_INVITATION_NOT_FOUND: &str = "200.000.0003";
pub const STAFF_INVITATION_EXPIRED: &str = "200.000.0004";
pub const STAFF_INVITATION_ALREADY_USED: &str = "200.000.0005";
pub const STAFF_ROLE_NOT_FOUND: &str = "200.000.0006";
pub const STAFF_INVALID_PERMISSION: &str = "200.000.0007";
pub const STAFF_ROLE_SYSTEM_PROTECTED: &str = "200.000.0008";

// Item domain
pub const ITEM_UNIT_NOT_FOUND: &str = "300.000.0001";
pub const ITEM_REQUIRED_UNIT: &str = "300.000.0002";
pub const ITEM_INVALID_SKU: &str = "300.000.0003";
pub const ITEM_INVALID_NAME: &str = "300.000.0004";
pub const ITEM_NOT_FOUND: &str = "300.000.0005";

// Auto number domain
pub const AUTO_NUMBER_SERIES_NOT_FOUND: &str = "400.000.0001";
pub const AUTO_NUMBER_INVALID_QUANTITY: &str = "400.000.0002";
pub const AUTO_NUMBER_INVALID_SERIES_KEY: &str = "400.000.0003";
pub const AUTO_NUMBER_INVALID_CONFIG: &str = "400.000.0004";

// Party domain
pub const PARTY_NOT_FOUND: &str = "500.000.0001";
pub const PARTY_ACCOUNT_NOT_FOUND: &str = "500.000.0002";
pub const PARTY_ACCOUNT_NOT_POSTING: &str = "500.000.0003";

// Chart of accounts domain
pub const COA_ACCOUNT_NOT_FOUND: &str = "600.000.0001";
pub const COA_PARENT_ACCOUNT_NOT_FOUND: &str = "600.000.0002";
pub const COA_PARENT_ACCOUNT_INVALID: &str = "600.000.0003";
pub const COA_SYSTEM_ACCOUNT_PROTECTED: &str = "600.000.0004";
pub const COA_ACCOUNT_HAS_CHILDREN: &str = "600.000.0005";
pub const COA_ACCOUNT_IN_USE: &str = "600.000.0006";

// Database and unexpected internal failures
pub const LISTING_INVALID_PAGINATION: &str = "900.000.0001";
pub const DATABASE_DUPLICATE_RECORD: &str = "900.001.0001";
pub const DATABASE_OPERATION_FAILED: &str = "900.001.0002";
pub const INTERNAL_SERVER_ERROR: &str = "900.000.0001";
