use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, EntityTrait, FromQueryResult,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::auth::permission::{Permission, normalize_permission_codes, serialize_permission_codes};
use crate::db::listing::{PageListResult, execute_page_query, validate_page_pagination};
use crate::entity::staff::staff_role_entity as StaffRole;
use crate::entity::{GenericStatus, PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use crate::utils::misc_helpers::trim_and_filter_empty;

pub struct StaffRoleService;

pub struct CreateStaffRoleInput {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
}

pub struct UpdateStaffRoleInput {
    pub name_primary: Option<String>,
    pub name_secondary: Option<String>,
    pub permission_codes: Option<Vec<String>>,
    pub status: Option<GenericStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffRoleSortField {
    CreatedAt,
    NamePrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct StaffRoleListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub name: Option<String>,
    pub status: Option<GenericStatus>,
    pub sort: Option<StaffRoleSortField>,
    pub direction: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq, FromQueryResult)]
pub struct StaffRoleListItem {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: String,
    pub is_system_role: bool,
    pub status: GenericStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaffRoleDetail {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
    pub is_system_role: bool,
    pub status: GenericStatus,
}

pub struct DefaultOrganizationRoles {
    pub owner_role_id: PrimaryId,
}

impl StaffRoleService {
    pub async fn create_staff_role(
        ctx: &ServiceContext,
        payload: CreateStaffRoleInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let permission_codes = normalize_permission_codes(&payload.permission_codes)?;
        let role = Self::create_role(
            &ctx.app_state.primary_write_replica,
            actor_id,
            organization_id,
            payload.name_primary,
            payload.name_secondary,
            &permission_codes,
            false,
        )
        .await?;

        Ok(role.public_id)
    }

    pub async fn seed_default_roles(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
    ) -> Result<DefaultOrganizationRoles, AppError> {
        // Bootstrap a standard role set per organization so authorization can
        // stay data-driven while new organizations still start in a usable state.

        // owner with all permissions by default
        let owner_role = Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Owner".to_string(),
            None,
            &Permission::all_codes(),
            true,
        )
        .await?;

        // admin with selected permissions
        Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Admin".to_string(),
            None,
            &[
                Permission::BranchCreate.code().to_string(),
                Permission::StaffInvite.code().to_string(),
                Permission::StaffInvitationResend.code().to_string(),
                Permission::StaffInvitationRevoke.code().to_string(),
                Permission::StaffRoleCreate.code().to_string(),
                Permission::StaffRoleRead.code().to_string(),
                Permission::StaffRoleUpdate.code().to_string(),
                Permission::StaffRoleDelete.code().to_string(),
                Permission::StaffRead.code().to_string(),
                Permission::StaffUpdate.code().to_string(),
                Permission::StaffDelete.code().to_string(),
            ],
            true,
        )
        .await?;

        // manager.
        Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Manager".to_string(),
            None,
            &[
                Permission::BranchCreate.code().to_string(),
                Permission::StaffInvite.code().to_string(),
                Permission::StaffInvitationResend.code().to_string(),
                Permission::StaffInvitationRevoke.code().to_string(),
                Permission::StaffRead.code().to_string(),
                Permission::StaffUpdate.code().to_string(),
            ],
            true,
        )
        .await?;

        Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Staff".to_string(),
            None,
            &Vec::new(),
            true,
        )
        .await?;

        Ok(DefaultOrganizationRoles {
            owner_role_id: owner_role.id,
        })
    }

    pub async fn list_staff_roles_page(
        ctx: &ServiceContext,
        input: StaffRoleListPageInput,
    ) -> Result<PageListResult<StaffRoleListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(StaffRoleSortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query =
            Self::build_role_list_query(organization_id, input.name.as_deref(), input.status);
        let query = Self::apply_page_sort(query, sort_field, sort_direction);
        let query = Self::select_role_list_columns(query).into_model::<StaffRoleListItem>();

        execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await
    }

    pub async fn get_staff_role(
        ctx: &ServiceContext,
        public_id: &str,
    ) -> Result<StaffRoleDetail, AppError> {
        let organization_id = ctx.get_organization_id()?;
        let role = Self::find_role_by_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            public_id,
        )
        .await?;

        Ok(Self::map_role_detail(role))
    }

    pub async fn update_staff_role(
        ctx: &ServiceContext,
        public_id: &str,
        payload: UpdateStaffRoleInput,
    ) -> Result<StaffRoleDetail, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let existing = Self::find_role_by_public_id(
            &ctx.app_state.primary_write_replica,
            organization_id,
            public_id,
        )
        .await?;

        let mut updated = existing.into_active_model();
        if let Some(name_primary) = payload.name_primary {
            updated.name_primary = Set(name_primary);
        }
        if payload.name_secondary.is_some() {
            updated.name_secondary = Set(payload.name_secondary);
        }
        if let Some(permission_codes) = payload.permission_codes {
            let permission_codes = normalize_permission_codes(&permission_codes)?;
            updated.permissions = Set(serialize_permission_codes(&permission_codes)?);
        }
        if let Some(status) = payload.status {
            updated.status = Set(status);
        }
        updated.updated_by_actor_id = Set(Some(actor_id));
        updated.updated_at = Set(now);

        let role = updated.update(&ctx.app_state.primary_write_replica).await?;
        Ok(Self::map_role_detail(role))
    }

    pub async fn delete_staff_role(ctx: &ServiceContext, public_id: &str) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let role = Self::find_role_by_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            public_id,
        )
        .await?;

        if role.is_system_role {
            return Err(StaffServiceError::SystemRoleProtected.into());
        }

        let result = StaffRole::Entity::update_many()
            .col_expr(
                StaffRole::Column::Status,
                sea_orm::sea_query::Expr::value(GenericStatus::Deleted),
            )
            .col_expr(
                StaffRole::Column::UpdatedByActorId,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                StaffRole::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffRole::Column::Id.eq(role.id))
            .filter(StaffRole::Column::Status.ne(GenericStatus::Deleted))
            .exec(&ctx.app_state.primary_write_replica)
            .await?;

        if result.rows_affected == 0 {
            return Err(StaffServiceError::RoleNotFound.into());
        }

        Ok(())
    }

    async fn create_role(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        name_primary: String,
        name_secondary: Option<String>,
        permission_codes: &[String],
        is_system_role: bool,
    ) -> Result<StaffRole::Model, AppError> {
        let now = DateHelper::now().value();
        StaffRole::ActiveModel {
            status: Set(GenericStatus::Active),
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            name_primary: Set(name_primary),
            name_secondary: Set(name_secondary),
            permissions: Set(serialize_permission_codes(permission_codes)?),
            is_system_role: Set(is_system_role),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db_transaction)
        .await
        .map_err(Into::into)
    }

    async fn find_role_by_public_id(
        db: &impl ConnectionTrait,
        organization_id: PrimaryId,
        public_id: &str,
    ) -> Result<StaffRole::Model, AppError> {
        StaffRole::Entity::find()
            .filter(StaffRole::Column::OrganizationId.eq(organization_id))
            .filter(StaffRole::Column::PublicId.eq(public_id))
            .filter(StaffRole::Column::Status.ne(GenericStatus::Deleted))
            .one(db)
            .await?
            .ok_or_else(|| StaffServiceError::RoleNotFound.into())
    }

    fn build_role_list_query(
        organization_id: PrimaryId,
        name: Option<&str>,
        status: Option<GenericStatus>,
    ) -> sea_orm::Select<StaffRole::Entity> {
        let mut query =
            StaffRole::Entity::find().filter(StaffRole::Column::OrganizationId.eq(organization_id));

        if let Some(name) = trim_and_filter_empty(name) {
            query = query.filter(
                Condition::any()
                    .add(StaffRole::Column::NamePrimary.contains(name))
                    .add(StaffRole::Column::NameSecondary.contains(name)),
            );
        }

        if let Some(status) = status {
            query = query.filter(StaffRole::Column::Status.eq(status));
        } else {
            query = query.filter(StaffRole::Column::Status.ne(GenericStatus::Deleted));
        }

        query
    }

    fn apply_page_sort(
        query: sea_orm::Select<StaffRole::Entity>,
        sort_field: StaffRoleSortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<StaffRole::Entity> {
        match (sort_field, sort_direction) {
            (StaffRoleSortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(StaffRole::Column::CreatedAt)
                .order_by_asc(StaffRole::Column::Id),
            (StaffRoleSortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(StaffRole::Column::CreatedAt)
                .order_by_desc(StaffRole::Column::Id),
            (StaffRoleSortField::NamePrimary, SortDirection::Asc) => query
                .order_by_asc(StaffRole::Column::NamePrimary)
                .order_by_asc(StaffRole::Column::Id),
            (StaffRoleSortField::NamePrimary, SortDirection::Desc) => query
                .order_by_desc(StaffRole::Column::NamePrimary)
                .order_by_desc(StaffRole::Column::Id),
        }
    }

    fn select_role_list_columns<Q>(query: Q) -> Q
    where
        Q: QuerySelect<QueryStatement = sea_orm::sea_query::SelectStatement>,
    {
        query
            .select_only()
            .column(StaffRole::Column::PublicId)
            .column(StaffRole::Column::NamePrimary)
            .column(StaffRole::Column::NameSecondary)
            .column_as(StaffRole::Column::Permissions, "permission_codes")
            .column(StaffRole::Column::IsSystemRole)
            .column(StaffRole::Column::Status)
    }

    fn map_role_detail(role: StaffRole::Model) -> StaffRoleDetail {
        StaffRoleDetail {
            public_id: role.public_id,
            name_primary: role.name_primary,
            name_secondary: role.name_secondary,
            permission_codes: crate::auth::permission::deserialize_permission_codes(
                &role.permissions,
            ),
            is_system_role: role.is_system_role,
            status: role.status,
        }
    }
}
