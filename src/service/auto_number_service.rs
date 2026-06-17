use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Datelike, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, FromQueryResult,
    IntoActiveModel, QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait,
};

use crate::db::listing::{PageListResult, execute_page_query, validate_page_pagination};
use crate::entity::auto_number::auto_number_allocation_entity::{
    self as AutoNumberAllocation, AutoNumberAllocationStatus,
};
use crate::entity::auto_number::auto_number_counter_entity as AutoNumberCounter;
use crate::entity::auto_number::auto_number_series_entity::{
    self as AutoNumberSeries, AutoNumberResetPolicy, AutoNumberSeriesModel, AutoNumberStatus,
};
use crate::entity::organization::branch_entity as Branch;
use crate::entity::{PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::error_codes;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoNumberRequest {
    pub branch_id: PrimaryId,
    pub series_key: String,
    pub quantity: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoNumberAllocationResult {
    pub allocation_public_id: PublicId,
    pub branch_id: PrimaryId,
    pub series_key: String,
    pub sequence_number: i64,
    pub formatted_number: String,
}

pub struct CreateAutoNumberSeriesInput {
    pub branch_id: PrimaryId,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicy,
    pub status: Option<AutoNumberStatus>,
}

pub struct UpdateAutoNumberSeriesInput {
    pub branch_id: Option<PrimaryId>,
    pub series_key: Option<String>,
    pub prefix_template: Option<String>,
    pub suffix_template: Option<String>,
    pub padding_width: Option<i16>,
    pub start_number: Option<i64>,
    pub increment_by: Option<i16>,
    pub reset_policy: Option<AutoNumberResetPolicy>,
    pub status: Option<AutoNumberStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoNumberSeriesSortField {
    CreatedAt,
    SeriesKey,
    Branch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct AutoNumberSeriesListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub branch_id: Option<PrimaryId>,
    pub series_key: Option<String>,
    pub status: Option<AutoNumberStatus>,
    pub sort: Option<AutoNumberSeriesSortField>,
    pub direction: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq, FromQueryResult)]
pub struct AutoNumberSeriesListItemRaw {
    pub public_id: PublicId,
    pub branch_id: PrimaryId,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicy,
    pub status: AutoNumberStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoNumberSeriesListItem {
    pub public_id: PublicId,
    pub branch_public_id: PublicId,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicy,
    pub status: AutoNumberStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AutoNumberSeriesDetail {
    pub public_id: PublicId,
    pub branch_public_id: PublicId,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicy,
    pub status: AutoNumberStatus,
}

pub struct AutoNumberService;

impl AutoNumberService {
    pub async fn create_series(
        ctx: &ServiceContext,
        payload: CreateAutoNumberSeriesInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let series_key = Self::normalize_series_key(payload.series_key)?;

        let series = AutoNumberSeries::ActiveModel {
            organization_id: Set(organization_id),
            branch_id: Set(payload.branch_id),
            series_key: Set(series_key),
            public_id: Set(IdGenerator::generate_general_id()),
            prefix_template: Set(payload.prefix_template),
            suffix_template: Set(payload.suffix_template),
            padding_width: Set(payload.padding_width),
            start_number: Set(payload.start_number),
            increment_by: Set(payload.increment_by),
            reset_policy: Set(payload.reset_policy),
            status: Set(payload.status.unwrap_or(AutoNumberStatus::Active)),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        let model_for_validation = AutoNumberSeriesModel {
            id: 0,
            organization_id,
            branch_id: payload.branch_id,
            series_key: series.series_key.as_ref().clone(),
            public_id: String::new(),
            prefix_template: series.prefix_template.as_ref().clone(),
            suffix_template: series.suffix_template.as_ref().clone(),
            padding_width: *series.padding_width.as_ref(),
            start_number: *series.start_number.as_ref(),
            increment_by: *series.increment_by.as_ref(),
            reset_policy: series.reset_policy.as_ref().clone(),
            status: series.status.as_ref().clone(),
            created_by_actor_id: actor_id,
            updated_by_actor_id: None,
            created_at: now,
            updated_at: now,
        };
        Self::validate_series_config(&model_for_validation)?;

        let series = series.insert(&ctx.app_state.primary_write_replica).await?;

        Ok(series.public_id)
    }

    pub async fn list_series_page(
        ctx: &ServiceContext,
        input: AutoNumberSeriesListPageInput,
    ) -> Result<PageListResult<AutoNumberSeriesListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(AutoNumberSeriesSortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query = Self::build_series_list_query(
            organization_id,
            input.branch_id,
            input.series_key.as_deref(),
            input.status,
        );
        let query = Self::apply_series_page_sort(query, sort_field, sort_direction);
        let query =
            Self::select_series_list_columns(query).into_model::<AutoNumberSeriesListItemRaw>();

        let result =
            execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await?;
        let branch_public_ids = Self::branch_public_ids_for_rows(
            &ctx.app_state.primary_read_replica,
            organization_id,
            result.rows.iter().map(|row| row.branch_id).collect(),
        )
        .await?;

        Ok(result.map_rows(|row| Self::map_series_list_item(row, &branch_public_ids)))
    }

    pub async fn get_series(
        ctx: &ServiceContext,
        public_id: &str,
    ) -> Result<AutoNumberSeriesDetail, AppError> {
        let organization_id = ctx.get_organization_id()?;
        let series = Self::find_series_by_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            public_id,
        )
        .await?;
        let branch_public_id = Self::branch_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            series.branch_id,
        )
        .await?;

        Ok(Self::map_series_detail(series, branch_public_id))
    }

    pub async fn update_series(
        ctx: &ServiceContext,
        public_id: &str,
        payload: UpdateAutoNumberSeriesInput,
    ) -> Result<AutoNumberSeriesDetail, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let existing = Self::find_series_by_public_id(
            &ctx.app_state.primary_write_replica,
            organization_id,
            public_id,
        )
        .await?;

        let mut updated = existing.clone().into_active_model();
        if let Some(branch_id) = payload.branch_id {
            updated.branch_id = Set(branch_id);
        }
        if let Some(series_key) = payload.series_key {
            updated.series_key = Set(Self::normalize_series_key(series_key)?);
        }
        if let Some(prefix_template) = payload.prefix_template {
            updated.prefix_template = Set(prefix_template);
        }
        if payload.suffix_template.is_some() {
            updated.suffix_template = Set(payload.suffix_template);
        }
        if let Some(padding_width) = payload.padding_width {
            updated.padding_width = Set(padding_width);
        }
        if let Some(start_number) = payload.start_number {
            updated.start_number = Set(start_number);
        }
        if let Some(increment_by) = payload.increment_by {
            updated.increment_by = Set(increment_by);
        }
        if let Some(reset_policy) = payload.reset_policy {
            updated.reset_policy = Set(reset_policy);
        }
        if let Some(status) = payload.status {
            updated.status = Set(status);
        }
        updated.updated_by_actor_id = Set(Some(actor_id));
        updated.updated_at = Set(now);

        let validation_model = AutoNumberSeriesModel {
            id: existing.id,
            organization_id,
            branch_id: *updated.branch_id.as_ref(),
            series_key: updated.series_key.as_ref().clone(),
            public_id: existing.public_id,
            prefix_template: updated.prefix_template.as_ref().clone(),
            suffix_template: updated.suffix_template.as_ref().clone(),
            padding_width: *updated.padding_width.as_ref(),
            start_number: *updated.start_number.as_ref(),
            increment_by: *updated.increment_by.as_ref(),
            reset_policy: updated.reset_policy.as_ref().clone(),
            status: updated.status.as_ref().clone(),
            created_by_actor_id: existing.created_by_actor_id,
            updated_by_actor_id: Some(actor_id),
            created_at: existing.created_at,
            updated_at: now,
        };
        Self::validate_series_config(&validation_model)?;

        let updated = updated.update(&ctx.app_state.primary_write_replica).await?;
        let branch_public_id = Self::branch_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            updated.branch_id,
        )
        .await?;

        Ok(Self::map_series_detail(updated, branch_public_id))
    }

    pub async fn delete_series(ctx: &ServiceContext, public_id: &str) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();

        let result = AutoNumberSeries::Entity::update_many()
            .col_expr(
                AutoNumberSeries::Column::Status,
                sea_orm::sea_query::Expr::value(AutoNumberStatus::Deleted),
            )
            .col_expr(
                AutoNumberSeries::Column::UpdatedByActorId,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                AutoNumberSeries::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(AutoNumberSeries::Column::OrganizationId.eq(organization_id))
            .filter(AutoNumberSeries::Column::PublicId.eq(public_id))
            .filter(AutoNumberSeries::Column::Status.ne(AutoNumberStatus::Deleted))
            .exec(&ctx.app_state.primary_write_replica)
            .await?;

        if result.rows_affected == 0 {
            return Err(Self::series_not_found());
        }

        Ok(())
    }

    pub async fn allocate_one(
        ctx: &ServiceContext,
        branch_id: PrimaryId,
        series_key: impl Into<String>,
    ) -> Result<AutoNumberAllocationResult, AppError> {
        let mut allocations = Self::allocate_many(
            ctx,
            vec![AutoNumberRequest {
                branch_id,
                series_key: series_key.into(),
                quantity: 1,
            }],
        )
        .await?;

        allocations
            .pop()
            .ok_or_else(|| AppError::InternalServer("Auto number allocation failed".into()))
    }

    pub async fn allocate_many(
        ctx: &ServiceContext,
        requests: Vec<AutoNumberRequest>,
    ) -> Result<Vec<AutoNumberAllocationResult>, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let txn = ctx.app_state.primary_write_replica.begin().await?;

        let allocations =
            Self::allocate_many_in_transaction(&txn, actor_id, organization_id, requests, None)
                .await?;

        txn.commit().await?;

        Ok(allocations)
    }

    pub async fn allocate_one_for_target_in_transaction(
        txn: &DatabaseTransaction,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        branch_id: PrimaryId,
        series_key: impl Into<String>,
        target_public_id: PublicId,
    ) -> Result<AutoNumberAllocationResult, AppError> {
        let mut allocations = Self::allocate_many_in_transaction(
            txn,
            actor_id,
            organization_id,
            vec![AutoNumberRequest {
                branch_id,
                series_key: series_key.into(),
                quantity: 1,
            }],
            Some(target_public_id),
        )
        .await?;

        allocations
            .pop()
            .ok_or_else(|| AppError::InternalServer("Auto number allocation failed".into()))
    }

    async fn allocate_many_in_transaction(
        txn: &DatabaseTransaction,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        requests: Vec<AutoNumberRequest>,
        target_public_id: Option<PublicId>,
    ) -> Result<Vec<AutoNumberAllocationResult>, AppError> {
        let grouped_requests = Self::group_requests(requests)?;
        let now = DateHelper::now().value();
        let mut allocations = Vec::new();

        for ((branch_id, series_key), quantity) in grouped_requests {
            let series =
                Self::lock_active_series(txn, organization_id, branch_id, &series_key).await?;
            Self::validate_series_config(&series)?;

            let period_key = Self::period_key(&series.reset_policy, now);
            let rendered_prefix =
                Self::render_template(&series.prefix_template, &series.reset_policy, now);
            let rendered_suffix = series
                .suffix_template
                .as_deref()
                .map(|template| Self::render_template(template, &series.reset_policy, now))
                .unwrap_or_default();

            let counter = Self::lock_or_create_counter(txn, &series, &period_key, now).await?;
            let first_number = counter.next_number;
            let last_number =
                first_number + ((quantity as i64 - 1) * i64::from(series.increment_by));
            let next_number = last_number + i64::from(series.increment_by);

            for offset in 0..quantity {
                let sequence_number =
                    first_number + i64::from(offset) * i64::from(series.increment_by);
                let formatted_number = Self::format_number(
                    &rendered_prefix,
                    sequence_number,
                    series.padding_width,
                    &rendered_suffix,
                )?;
                let allocation = AutoNumberAllocation::ActiveModel {
                    public_id: Set(IdGenerator::generate_general_id()),
                    organization_id: Set(organization_id),
                    branch_id: Set(branch_id),
                    series_id: Set(series.id),
                    series_key: Set(series_key.clone()),
                    period_key: Set(period_key.clone()),
                    sequence_number: Set(sequence_number),
                    formatted_number: Set(formatted_number),
                    target_public_id: Set(target_public_id.clone()),
                    status: Set(AutoNumberAllocationStatus::Committed),
                    created_by_actor_id: Set(actor_id),
                    created_at: Set(now),
                    ..Default::default()
                }
                .insert(txn)
                .await?;

                allocations.push(AutoNumberAllocationResult {
                    allocation_public_id: allocation.public_id,
                    branch_id,
                    series_key: series_key.clone(),
                    sequence_number,
                    formatted_number: allocation.formatted_number,
                });
            }

            let mut counter_active: AutoNumberCounter::ActiveModel = counter.into();
            counter_active.next_number = Set(next_number);
            counter_active.last_issued_number = Set(Some(last_number));
            counter_active.updated_at = Set(now);
            counter_active.update(txn).await?;
        }

        Ok(allocations)
    }

    fn group_requests(
        requests: Vec<AutoNumberRequest>,
    ) -> Result<BTreeMap<(PrimaryId, String), u32>, AppError> {
        let mut grouped_requests = BTreeMap::new();

        for request in requests {
            if request.quantity == 0 {
                return Err(Self::bad_request(
                    error_codes::AUTO_NUMBER_INVALID_QUANTITY,
                    "Auto number quantity must be greater than zero",
                ));
            }

            let series_key = request.series_key.trim().to_string();
            if series_key.is_empty() {
                return Err(Self::bad_request(
                    error_codes::AUTO_NUMBER_INVALID_SERIES_KEY,
                    "Auto number series key is required",
                ));
            }

            grouped_requests
                .entry((request.branch_id, series_key))
                .and_modify(|quantity| *quantity += request.quantity)
                .or_insert(request.quantity);
        }

        if grouped_requests.is_empty() {
            return Err(Self::bad_request(
                error_codes::AUTO_NUMBER_INVALID_QUANTITY,
                "At least one auto number request is required",
            ));
        }

        Ok(grouped_requests)
    }

    async fn lock_active_series(
        txn: &DatabaseTransaction,
        organization_id: PrimaryId,
        branch_id: PrimaryId,
        series_key: &str,
    ) -> Result<AutoNumberSeriesModel, AppError> {
        AutoNumberSeries::Entity::find()
            .filter(AutoNumberSeries::Column::OrganizationId.eq(organization_id))
            .filter(AutoNumberSeries::Column::BranchId.eq(branch_id))
            .filter(AutoNumberSeries::Column::SeriesKey.eq(series_key))
            .filter(AutoNumberSeries::Column::Status.eq(AutoNumberStatus::Active))
            .lock_exclusive()
            .one(txn)
            .await?
            .ok_or_else(|| {
                Self::bad_request(
                    error_codes::AUTO_NUMBER_SERIES_NOT_FOUND,
                    "Active auto number series not found",
                )
            })
    }

    async fn lock_or_create_counter(
        txn: &DatabaseTransaction,
        series: &AutoNumberSeriesModel,
        period_key: &str,
        now: DateTime<Utc>,
    ) -> Result<AutoNumberCounter::Model, AppError> {
        if let Some(counter) = AutoNumberCounter::Entity::find()
            .filter(AutoNumberCounter::Column::SeriesId.eq(series.id))
            .filter(AutoNumberCounter::Column::PeriodKey.eq(period_key))
            .order_by_asc(AutoNumberCounter::Column::Id)
            .lock_exclusive()
            .one(txn)
            .await?
        {
            return Ok(counter);
        }

        AutoNumberCounter::ActiveModel {
            series_id: Set(series.id),
            period_key: Set(period_key.to_string()),
            next_number: Set(series.start_number),
            last_issued_number: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(txn)
        .await
        .map_err(AppError::from)
    }

    fn validate_series_config(series: &AutoNumberSeriesModel) -> Result<(), AppError> {
        if series.padding_width <= 0 {
            return Err(Self::bad_request(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number padding width must be greater than zero",
            ));
        }
        if series.start_number <= 0 {
            return Err(Self::bad_request(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number start number must be greater than zero",
            ));
        }
        if series.increment_by <= 0 {
            return Err(Self::bad_request(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number increment must be greater than zero",
            ));
        }

        let template = format!(
            "{}{}",
            series.prefix_template,
            series.suffix_template.as_deref().unwrap_or_default()
        );
        let required_token = match series.reset_policy {
            AutoNumberResetPolicy::Never => None,
            AutoNumberResetPolicy::Monthly => Some("{YYYYMM}"),
            AutoNumberResetPolicy::CalendarYear => Some("{YYYY}"),
            AutoNumberResetPolicy::FiscalYear => Some("{FY}"),
        };

        if let Some(required_token) = required_token {
            if !template.contains(required_token) {
                return Err(Self::bad_request(
                    error_codes::AUTO_NUMBER_INVALID_CONFIG,
                    "Auto number reset policy requires a matching period token",
                ));
            }
        }

        Ok(())
    }

    fn period_key(reset_policy: &AutoNumberResetPolicy, now: DateTime<Utc>) -> String {
        match reset_policy {
            AutoNumberResetPolicy::Never => "lifetime".to_string(),
            AutoNumberResetPolicy::Monthly => format!("{:04}-{:02}", now.year(), now.month()),
            AutoNumberResetPolicy::CalendarYear => now.year().to_string(),
            AutoNumberResetPolicy::FiscalYear => Self::fiscal_year(now),
        }
    }

    fn render_template(
        template: &str,
        reset_policy: &AutoNumberResetPolicy,
        now: DateTime<Utc>,
    ) -> String {
        let fiscal_year = Self::fiscal_year(now);
        let rendered = template
            .replace("{FY}", &fiscal_year)
            .replace("{YYYYMM}", &format!("{:04}{:02}", now.year(), now.month()))
            .replace("{YYYY}", &now.year().to_string());

        match reset_policy {
            AutoNumberResetPolicy::Never => rendered,
            AutoNumberResetPolicy::Monthly => rendered,
            AutoNumberResetPolicy::CalendarYear => rendered,
            AutoNumberResetPolicy::FiscalYear => rendered,
        }
    }

    fn fiscal_year(now: DateTime<Utc>) -> String {
        let start_year = if now.month() >= 4 {
            now.year()
        } else {
            now.year() - 1
        };
        format!("{}-{}", start_year, start_year + 1)
    }

    fn format_number(
        prefix: &str,
        sequence_number: i64,
        padding_width: i16,
        suffix: &str,
    ) -> Result<String, AppError> {
        let padding_width = usize::try_from(padding_width).map_err(|_| {
            Self::bad_request(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number padding width is invalid",
            )
        })?;

        Ok(format!(
            "{prefix}{sequence_number:0padding_width$}{suffix}",
            padding_width = padding_width
        ))
    }

    async fn find_series_by_public_id(
        db: &impl sea_orm::ConnectionTrait,
        organization_id: PrimaryId,
        public_id: &str,
    ) -> Result<AutoNumberSeriesModel, AppError> {
        AutoNumberSeries::Entity::find()
            .filter(AutoNumberSeries::Column::OrganizationId.eq(organization_id))
            .filter(AutoNumberSeries::Column::PublicId.eq(public_id))
            .filter(AutoNumberSeries::Column::Status.ne(AutoNumberStatus::Deleted))
            .one(db)
            .await?
            .ok_or_else(Self::series_not_found)
    }

    fn build_series_list_query(
        organization_id: PrimaryId,
        branch_id: Option<PrimaryId>,
        series_key: Option<&str>,
        status: Option<AutoNumberStatus>,
    ) -> sea_orm::Select<AutoNumberSeries::Entity> {
        let mut query = AutoNumberSeries::Entity::find()
            .filter(AutoNumberSeries::Column::OrganizationId.eq(organization_id));

        if let Some(branch_id) = branch_id {
            query = query.filter(AutoNumberSeries::Column::BranchId.eq(branch_id));
        }

        if let Some(series_key) = series_key.map(str::trim).filter(|value| !value.is_empty()) {
            query = query.filter(AutoNumberSeries::Column::SeriesKey.contains(series_key));
        }

        if let Some(status) = status {
            query = query.filter(AutoNumberSeries::Column::Status.eq(status));
        } else {
            query = query.filter(AutoNumberSeries::Column::Status.ne(AutoNumberStatus::Deleted));
        }

        query
    }

    fn apply_series_page_sort(
        query: sea_orm::Select<AutoNumberSeries::Entity>,
        sort_field: AutoNumberSeriesSortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<AutoNumberSeries::Entity> {
        match (sort_field, sort_direction) {
            (AutoNumberSeriesSortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(AutoNumberSeries::Column::CreatedAt)
                .order_by_asc(AutoNumberSeries::Column::Id),
            (AutoNumberSeriesSortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(AutoNumberSeries::Column::CreatedAt)
                .order_by_desc(AutoNumberSeries::Column::Id),
            (AutoNumberSeriesSortField::SeriesKey, SortDirection::Asc) => query
                .order_by_asc(AutoNumberSeries::Column::SeriesKey)
                .order_by_asc(AutoNumberSeries::Column::Id),
            (AutoNumberSeriesSortField::SeriesKey, SortDirection::Desc) => query
                .order_by_desc(AutoNumberSeries::Column::SeriesKey)
                .order_by_desc(AutoNumberSeries::Column::Id),
            (AutoNumberSeriesSortField::Branch, SortDirection::Asc) => query
                .order_by_asc(AutoNumberSeries::Column::BranchId)
                .order_by_asc(AutoNumberSeries::Column::Id),
            (AutoNumberSeriesSortField::Branch, SortDirection::Desc) => query
                .order_by_desc(AutoNumberSeries::Column::BranchId)
                .order_by_desc(AutoNumberSeries::Column::Id),
        }
    }

    fn select_series_list_columns<Q>(query: Q) -> Q
    where
        Q: QuerySelect<QueryStatement = sea_orm::sea_query::SelectStatement>,
    {
        query
            .select_only()
            .column(AutoNumberSeries::Column::PublicId)
            .column(AutoNumberSeries::Column::BranchId)
            .column(AutoNumberSeries::Column::SeriesKey)
            .column(AutoNumberSeries::Column::PrefixTemplate)
            .column(AutoNumberSeries::Column::SuffixTemplate)
            .column(AutoNumberSeries::Column::PaddingWidth)
            .column(AutoNumberSeries::Column::StartNumber)
            .column(AutoNumberSeries::Column::IncrementBy)
            .column(AutoNumberSeries::Column::ResetPolicy)
            .column(AutoNumberSeries::Column::Status)
    }

    async fn branch_public_ids_for_rows(
        db: &impl sea_orm::ConnectionTrait,
        organization_id: PrimaryId,
        branch_ids: Vec<PrimaryId>,
    ) -> Result<HashMap<PrimaryId, PublicId>, AppError> {
        let branches = Branch::Entity::find()
            .filter(Branch::Column::OrganizationId.eq(organization_id))
            .filter(Branch::Column::Id.is_in(branch_ids))
            .all(db)
            .await?;

        Ok(branches
            .into_iter()
            .map(|branch| (branch.id, branch.public_id))
            .collect())
    }

    async fn branch_public_id(
        db: &impl sea_orm::ConnectionTrait,
        organization_id: PrimaryId,
        branch_id: PrimaryId,
    ) -> Result<PublicId, AppError> {
        Self::branch_public_ids_for_rows(db, organization_id, vec![branch_id])
            .await?
            .remove(&branch_id)
            .ok_or_else(|| AppError::InternalServer("Auto number branch not found".into()))
    }

    fn map_series_list_item(
        row: AutoNumberSeriesListItemRaw,
        branch_public_ids: &HashMap<PrimaryId, PublicId>,
    ) -> AutoNumberSeriesListItem {
        AutoNumberSeriesListItem {
            branch_public_id: branch_public_ids
                .get(&row.branch_id)
                .cloned()
                .unwrap_or_default(),
            public_id: row.public_id,
            series_key: row.series_key,
            prefix_template: row.prefix_template,
            suffix_template: row.suffix_template,
            padding_width: row.padding_width,
            start_number: row.start_number,
            increment_by: row.increment_by,
            reset_policy: row.reset_policy,
            status: row.status,
        }
    }

    fn map_series_detail(
        series: AutoNumberSeriesModel,
        branch_public_id: PublicId,
    ) -> AutoNumberSeriesDetail {
        AutoNumberSeriesDetail {
            public_id: series.public_id,
            branch_public_id,
            series_key: series.series_key,
            prefix_template: series.prefix_template,
            suffix_template: series.suffix_template,
            padding_width: series.padding_width,
            start_number: series.start_number,
            increment_by: series.increment_by,
            reset_policy: series.reset_policy,
            status: series.status,
        }
    }

    fn normalize_series_key(series_key: impl Into<String>) -> Result<String, AppError> {
        let series_key = series_key.into().trim().to_string();
        if series_key.is_empty() {
            return Err(Self::bad_request(
                error_codes::AUTO_NUMBER_INVALID_SERIES_KEY,
                "Auto number series key is required",
            ));
        }

        Ok(series_key)
    }

    fn series_not_found() -> AppError {
        Self::bad_request(
            error_codes::AUTO_NUMBER_SERIES_NOT_FOUND,
            "Auto number series not found",
        )
    }

    fn bad_request(code: &'static str, message: impl Into<String>) -> AppError {
        AppError::BadRequest {
            code,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::{AutoNumberResetPolicy, AutoNumberService};

    #[test]
    fn fiscal_year_starts_in_april() {
        let march = Utc.with_ymd_and_hms(2026, 3, 31, 0, 0, 0).unwrap();
        let april = Utc.with_ymd_and_hms(2026, 4, 1, 0, 0, 0).unwrap();

        assert_eq!(AutoNumberService::fiscal_year(march), "2025-2026");
        assert_eq!(AutoNumberService::fiscal_year(april), "2026-2027");
    }

    #[test]
    fn renders_supported_period_tokens() {
        let now = Utc.with_ymd_and_hms(2026, 6, 17, 0, 0, 0).unwrap();

        let rendered = AutoNumberService::render_template(
            "INV-{FY}-{YYYY}-{YYYYMM}-",
            &AutoNumberResetPolicy::FiscalYear,
            now,
        );

        assert_eq!(rendered, "INV-2026-2027-2026-202606-");
    }

    #[test]
    fn formats_padded_number() {
        let number = AutoNumberService::format_number("CUS-", 7, 4, "").unwrap();

        assert_eq!(number, "CUS-0007");
    }
}
