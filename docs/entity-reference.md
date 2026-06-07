# Backend Entity Reference

This document summarizes the SeaORM entities defined under `src/entity`.

## Shared Types

| Type | Rust type | Notes |
| --- | --- | --- |
| `PrimaryId` | `i32` | Internal database primary and foreign key identifier. |
| `PublicId` | `String` | External/public identifier used on public-facing records. |

## Shared Enum

### `GenericStatus`

Database enum: `generic_status`

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Deleted` | `deleted` |

Used by: `actors`, `user_credentials`, `login_logs`, `organizations_meta`, `staff_branches`, `staff_invitation_branches`, `staff_roles`, `units`, `unit_conversions`.

## User Entities

### `UserEntity`

Source: `src/entity/user_entity.rs`  
Table: `users`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `public_id` | `PublicId` | Unique |
| `first_name` | `String` | Required |
| `last_name` | `String` | Required |
| `email` | `String` | Unique |
| `email_verified` | `bool` | Required |
| `status` | `UserStatus` | Required |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `UserStatus` (`user_status`)

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Deleted` | `deleted` |

Aliases: `UserModel`, `UserEntity`.

### `UserCredentialsEntity`

Source: `src/entity/user_credentials_entity.rs`  
Table: `user_credentials`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `user_id` | `PrimaryId` | Primary key, no auto increment |
| `password_hash` | `String` | Text column |
| `refresh_token_hash` | `Option<String>` | Nullable text column |
| `failed_attempts` | `i16` | Required |
| `status` | `GenericStatus` | Required |
| `created_at` | `DateTimeUtc` | Required |
| `password_changed_at` | `Option<DateTimeUtc>` | Nullable |
| `last_login_at` | `Option<DateTimeUtc>` | Nullable |
| `refresh_token_expires_at` | `Option<DateTimeUtc>` | Nullable |

Aliases: `UserCredentialsModel`, `UserCredentialsEntity`.

## Actor and Client App Entities

### `ActorEntity`

Source: `src/entity/actor_entity.rs`  
Table: `actors`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `user_id` | `Option<PrimaryId>` | Nullable user actor reference |
| `public_user_id` | `Option<PublicId>` | Nullable public user reference |
| `client_app_id` | `Option<PrimaryId>` | Nullable client app actor reference |
| `public_client_app_id` | `Option<PublicId>` | Nullable public client app reference |
| `actor_type` | `ActorType` | Required |
| `status` | `GenericStatus` | Required |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `ActorType` (`actor_type`)

| Variant | Database value |
| --- | --- |
| `ClientApp` | `client_app` |
| `User` | `user` |
| `System` | `system` |

Aliases: `ActorModel`, `ActorEntity`.

### `ClientAppEntity`

Source: `src/entity/client_app/client_app_entity.rs`  
Table: `client_apps`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `public_id` | `PublicId` | Unique |
| `name` | `String` | Required |
| `client_secret` | `String` | Unique |
| `status` | `ClientAppStatus` | Required |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `ClientAppStatus` (`client_app_status`)

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Deleted` | `deleted` |

Aliases: `ClientAppModel`, `ClientAppEntity`.

## Authentication Log Entity

### `LoginLogEntity`

Source: `src/entity/login_log_entity.rs`  
Table: `login_logs`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `user_id` | `Option<PrimaryId>` | Nullable |
| `identifier` | `String` | Login identifier, currently email string |
| `created_at` | `DateTimeUtc` | Required |
| `event_type` | `SignInLogEventType` | Required |
| `status` | `GenericStatus` | Required |
| `request_context` | `RequestContext` | JSON query result |

Enum: `SignInLogEventType` (`sign_in_log_event_type`)

| Variant | Database value |
| --- | --- |
| `LoginSuccess` | `login_success` |
| `LoginFailure` | `login_failure` |
| `Logout` | `logout` |
| `RefreshToken` | `refresh_token` |

`RequestContext` fields:

| Field | Type |
| --- | --- |
| `ip_address` | `Option<String>` |
| `user_agent` | `Option<String>` |
| `device` | `Option<String>` |
| `os` | `Option<String>` |
| `browser` | `Option<String>` |

Aliases: `LoginLogModel`, `LoginLogEntity`.

## Organization Entities

### `OrganizationEntity`

Source: `src/entity/organization/organization_entity.rs`  
Table: `organizations`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `prime_user_id` | `PrimaryId` | Required |
| `public_id` | `PublicId` | Unique |
| `name_primary` | `String` | Required |
| `name_secondary` | `Option<String>` | Nullable |
| `status` | `OrganizationStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `OrganizationStatus` (`organization_status`)

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Deleted` | `deleted` |

Aliases: `OrganizationModel`, `OrganizationEntity`.

### `OrganizationMetaEntity`

Source: `src/entity/organization/organization_meta_entity.rs`  
Table: `organizations_meta`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `organization_id` | `PrimaryId` | Primary key, no auto increment |
| `country_iso_code` | `String` | Length 2 |
| `currency_iso_code` | `String` | Length 3 |
| `default_branch_id` | `Option<PrimaryId>` | Nullable |
| `status` | `GenericStatus` | Required |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Aliases: `OrganizationMetaModel`, `OrganizationMetaEntity`.

### `BranchEntity`

Source: `src/entity/organization/branch_entity.rs`  
Table: `branches`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `organization_id` | `PrimaryId` | Required |
| `public_id` | `PublicId` | Unique |
| `name_primary` | `String` | Required |
| `name_secondary` | `Option<String>` | Nullable |
| `is_primary` | `bool` | Required |
| `status` | `BranchStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `BranchStatus` (`branch_status`)

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Inactive` | `inactive` |
| `Deleted` | `deleted` |

Aliases: `BranchModel`, `BranchEntity`.

## Staff Entities

### `StaffEntity`

Source: `src/entity/staff/staff_entity.rs`  
Table: `staffs`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `user_id` | `PrimaryId` | Unique key `org_staff` with `organization_id` |
| `organization_id` | `PrimaryId` | Unique key `org_staff` with `user_id` |
| `public_id` | `PublicId` | Unique |
| `name_primary` | `String` | Required |
| `name_secondary` | `Option<String>` | Nullable |
| `role_id` | `PrimaryId` | Required |
| `is_default_organization` | `bool` | Required |
| `status` | `StaffStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `StaffStatus` (`staff_status`)

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Inactive` | `inactive` |
| `Deleted` | `deleted` |

Aliases: `StaffModel`, `StaffEntity`.

### `StaffRoleEntity`

Source: `src/entity/staff/staff_role_entity.rs`  
Table: `staff_roles`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `organization_id` | `PrimaryId` | Required |
| `public_id` | `PublicId` | Unique |
| `name_primary` | `String` | Required |
| `name_secondary` | `Option<String>` | Nullable |
| `permissions` | `String` | Required |
| `is_system_role` | `bool` | Required |
| `status` | `GenericStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Aliases: `StaffRoleModel`, `StaffRoleEntity`.

### `StaffBranch`

Source: `src/entity/staff/staff_branch_entity.rs`  
Table: `staff_branches`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `staff_id` | `PrimaryId` | Unique key `staff_branch_unique` with `branch_id` |
| `branch_id` | `PrimaryId` | Unique key `staff_branch_unique` with `staff_id` |
| `status` | `GenericStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

This module does not currently define public aliases for `Model` or `Entity`.

### `StaffInvitationEntity`

Source: `src/entity/staff/staff_invitation_entity.rs`  
Table: `staff_invitations`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `public_id` | `PublicId` | Unique |
| `organization_id` | `PrimaryId` | Required |
| `invitee_email` | `String` | Required |
| `invitee_first_name` | `String` | Required |
| `invitee_last_name` | `String` | Required |
| `invited_role_id` | `PrimaryId` | Required |
| `invitation_token_hash` | `String` | Required |
| `invitation_token_id` | `String` | Unique |
| `token_expires_at` | `DateTimeUtc` | Required |
| `accepted_at` | `Option<DateTimeUtc>` | Nullable |
| `status` | `StaffInvitationStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `StaffInvitationStatus` (`staff_invitation_status`)

| Variant | Database value |
| --- | --- |
| `Pending` | `pending` |
| `Accepted` | `accepted` |
| `Expired` | `expired` |
| `Revoked` | `revoked` |

Aliases: `StaffInvitationModel`, `StaffInvitationEntity`.

### `StaffInvitationBranch`

Source: `src/entity/staff/staff_invitation_branch_entity.rs`  
Table: `staff_invitation_branches`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `staff_invitation_id` | `PrimaryId` | Unique key `invitation_branch_unique` with `branch_id` |
| `branch_id` | `PrimaryId` | Unique key `invitation_branch_unique` with `staff_invitation_id` |
| `status` | `GenericStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

This module does not currently define public aliases for `Model` or `Entity`.

## Item and Unit Entities

### `ItemEntity`

Source: `src/entity/item/item_entity.rs`  
Table: `items`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `organization_id` | `PrimaryId` | Unique keys `org_item_sku`, `item_barcode` |
| `public_id` | `PublicId` | Unique |
| `sku` | `String` | Unique key `item_sku` |
| `barcode` | `Option<String>` | Unique key `item_barcode` with `organization_id` |
| `name_primary` | `String` | Required |
| `name_secondary` | `Option<String>` | Nullable |
| `description` | `Option<String>` | Nullable |
| `item_type` | `ItemType` | Required |
| `item_usage` | `ItemUsage` | Required |
| `base_unit_id` | `PrimaryId` | Required |
| `purchase_unit_id` | `Option<PrimaryId>` | Nullable |
| `sales_unit_id` | `Option<PrimaryId>` | Nullable |
| `default_purchase_price` | `Option<Decimal>` | Nullable |
| `default_sales_price` | `Option<Decimal>` | Nullable |
| `track_inventory` | `bool` | Required |
| `allow_negative_stock` | `bool` | Required |
| `reorder_level` | `Option<Decimal>` | Nullable |
| `status` | `ItemStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enums:

`ItemType` (`item_type`)

| Variant | Database value |
| --- | --- |
| `Product` | `product` |
| `Service` | `service` |

`ItemUsage` (`item_usage`)

| Variant | Database value |
| --- | --- |
| `Sales` | `sales` |
| `Purchase` | `purchase` |
| `Both` | `both` |

`ItemStatus` (`item_status`)

| Variant | Database value |
| --- | --- |
| `Active` | `active` |
| `Inactive` | `inactive` |
| `Deleted` | `deleted` |

Aliases: `ItemModel`, `ItemEntity`.

### `UnitEntity`

Source: `src/entity/item/unit_entity.rs`  
Table: `units`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `organization_id` | `PrimaryId` | Unique key `unit_code` with `code` |
| `public_id` | `PublicId` | Unique |
| `code` | `String` | Unique key `unit_code` with `organization_id` |
| `name_primary` | `String` | Required |
| `name_secondary` | `Option<String>` | Nullable |
| `symbol` | `Option<String>` | Nullable |
| `decimal_places` | `i16` | Required |
| `is_system_unit` | `bool` | Required |
| `status` | `GenericStatus` | Required |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Aliases: `UnitModel`, `UnitEntity`.

### `ItemUnitConversionEntity`

Source: `src/entity/item/item_unit_conversion_entity.rs`  
Table: `unit_conversions`

| Field | Type | Constraints / notes |
| --- | --- | --- |
| `id` | `PrimaryId` | Primary key |
| `organization_id` | `PrimaryId` | Unique key `unit_conversion_pair` |
| `public_id` | `PublicId` | Unique |
| `item_id` | `PrimaryId` | Unique key `unit_conversion_pair` |
| `from_unit_id` | `PrimaryId` | Unique key `unit_conversion_pair` |
| `to_unit_id` | `PrimaryId` | Unique key `unit_conversion_pair` |
| `conversion_rate` | `Decimal` | Required |
| `quantity_precision` | `i16` | Required |
| `rounding_mode` | `ConversionRoundingMode` | Required |
| `status` | `GenericStatus` | Required |
| `note` | `Option<String>` | Nullable |
| `created_by_actor_id` | `PrimaryId` | Required audit actor |
| `updated_by_actor_id` | `Option<PrimaryId>` | Nullable audit actor |
| `created_at` | `DateTimeUtc` | Required |
| `updated_at` | `DateTimeUtc` | Required |

Enum: `ConversionRoundingMode` (`unit_conversion_rounding_mode`)

| Variant | Database value |
| --- | --- |
| `None` | `none` |
| `Round` | `round` |
| `Floor` | `floor` |
| `Ceil` | `ceil` |

Aliases: `ItemUnitConversionModel`, `ItemUnitConversionEntity`.

## Notes

- Most organization-scoped business entities include `created_by_actor_id`, `updated_by_actor_id`, `created_at`, and `updated_at` audit fields.
- `status` fields are generally soft-delete lifecycle markers rather than hard deletion signals.
- SeaORM relation definitions are not present in these entity files; relationships are implied by `PrimaryId` reference fields.
