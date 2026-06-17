use sea_orm::entity::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, FromQueryResult, QueryFilter,
    QueryOrder, QuerySelect, Set, TransactionTrait,
};

use crate::db::listing::{PageListResult, execute_page_query, validate_page_pagination};
use crate::entity::party::party_accounting_profile_entity as PartyAccountingProfile;
use crate::entity::party::party_address_entity::{self as PartyAddress, PartyAddressType};
use crate::entity::party::party_contact_entity as PartyContact;
use crate::entity::party::party_entity::{
    self as Party, PartyKind, PartySource, PartyStatus, PartyType,
};
use crate::entity::{PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::service::auto_number_service::AutoNumberService;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use crate::utils::misc_helpers::trim_and_filter_empty;

pub struct CreatePartyAddressInput {
    pub address_type: PartyAddressType,
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state_region: Option<String>,
    pub postal_code: Option<String>,
    pub country_iso_code: Option<String>,
    pub is_default_billing: bool,
    pub is_default_shipping: bool,
}

pub struct CreatePartyContactInput {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub designation: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

pub struct CreatePartyAccountingProfileInput {
    pub default_sales_account_id: Option<PrimaryId>,
    pub default_purchase_account_id: Option<PrimaryId>,
    pub default_receivable_account_id: Option<PrimaryId>,
    pub default_payable_account_id: Option<PrimaryId>,
    pub default_output_tax_account_id: Option<PrimaryId>,
    pub default_input_tax_account_id: Option<PrimaryId>,
}

pub struct CreatePartyInput {
    pub branch_id: PrimaryId,
    pub party_type: PartyType,
    pub party_kind: PartyKind,
    pub status: Option<PartyStatus>,
    pub source: Option<PartySource>,
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_no: Option<String>,
    pub tax_treatment: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
    pub allow_credit: bool,
    pub notes: Option<String>,
    pub addresses: Vec<CreatePartyAddressInput>,
    pub contacts: Vec<CreatePartyContactInput>,
    pub accounting_profile: Option<CreatePartyAccountingProfileInput>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartySortField {
    CreatedAt,
    DisplayName,
    Code,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct PartyListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub party_type: Option<PartyType>,
    pub status: Option<PartyStatus>,
    pub sort: Option<PartySortField>,
    pub direction: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq, FromQueryResult)]
pub struct PartyListItem {
    pub public_id: String,
    pub code: String,
    pub party_type: PartyType,
    pub party_kind: PartyKind,
    pub status: PartyStatus,
    pub source: PartySource,
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_no: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub allow_credit: bool,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
}

pub struct PartyService;

impl PartyService {
    pub async fn create_party(
        ctx: &ServiceContext,
        payload: CreatePartyInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();

        let txn = ctx.app_state.primary_write_replica.begin().await?;
        let public_id = IdGenerator::generate_general_id();
        let auto_number = AutoNumberService::allocate_one_for_target_in_transaction(
            &txn,
            actor_id,
            organization_id,
            payload.branch_id,
            Self::series_key_for_party_type(&payload.party_type),
            public_id.clone(),
        )
        .await?;

        let party = Party::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(public_id),
            code: Set(auto_number.formatted_number),
            party_type: Set(payload.party_type),
            party_kind: Set(payload.party_kind),
            status: Set(payload.status.unwrap_or(PartyStatus::Active)),
            source: Set(payload.source.unwrap_or(PartySource::Manual)),
            display_name: Set(payload.display_name),
            name_primary: Set(payload.name_primary),
            name_secondary: Set(payload.name_secondary),
            legal_name: Set(payload.legal_name),
            phone: Set(payload.phone),
            email: Set(payload.email),
            tax_no: Set(payload.tax_no),
            tax_treatment: Set(payload.tax_treatment),
            country_iso_code: Set(payload.country_iso_code),
            currency_iso_code: Set(payload.currency_iso_code),
            payment_terms_days: Set(payload.payment_terms_days),
            credit_limit: Set(payload.credit_limit),
            allow_credit: Set(payload.allow_credit),
            notes: Set(payload.notes),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        for address in payload.addresses {
            PartyAddress::ActiveModel {
                party_id: Set(party.id),
                address_type: Set(address.address_type),
                label: Set(address.label),
                line1: Set(address.line1),
                line2: Set(address.line2),
                city: Set(address.city),
                state_region: Set(address.state_region),
                postal_code: Set(address.postal_code),
                country_iso_code: Set(address.country_iso_code),
                is_default_billing: Set(address.is_default_billing),
                is_default_shipping: Set(address.is_default_shipping),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        for contact in payload.contacts {
            PartyContact::ActiveModel {
                party_id: Set(party.id),
                name_primary: Set(contact.name_primary),
                name_secondary: Set(contact.name_secondary),
                designation: Set(contact.designation),
                phone: Set(contact.phone),
                email: Set(contact.email),
                is_primary: Set(contact.is_primary),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        if let Some(accounting_profile) = payload.accounting_profile {
            PartyAccountingProfile::ActiveModel {
                party_id: Set(party.id),
                organization_id: Set(organization_id),
                default_sales_account_id: Set(accounting_profile.default_sales_account_id),
                default_purchase_account_id: Set(accounting_profile.default_purchase_account_id),
                default_receivable_account_id: Set(
                    accounting_profile.default_receivable_account_id,
                ),
                default_payable_account_id: Set(accounting_profile.default_payable_account_id),
                default_output_tax_account_id: Set(
                    accounting_profile.default_output_tax_account_id,
                ),
                default_input_tax_account_id: Set(accounting_profile.default_input_tax_account_id),
                created_by_actor_id: Set(actor_id),
                updated_by_actor_id: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }

        txn.commit().await?;

        Ok(party.public_id)
    }

    fn series_key_for_party_type(party_type: &PartyType) -> &'static str {
        match party_type {
            PartyType::Customer => "customer",
            PartyType::Vendor => "vendor",
            PartyType::Both => "party",
        }
    }

    pub async fn list_parties_page(
        ctx: &ServiceContext,
        input: PartyListPageInput,
    ) -> Result<PageListResult<PartyListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(PartySortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query = Self::build_party_list_query(
            organization_id,
            input.name.as_deref(),
            input.phone.as_deref(),
            input.party_type,
            input.status,
        );
        let query = Self::apply_page_sort(query, sort_field, sort_direction);
        let query = Self::select_party_list_columns(query).into_model::<PartyListItem>();

        execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await
    }

    fn build_party_list_query(
        organization_id: PrimaryId,
        name: Option<&str>,
        phone: Option<&str>,
        party_type: Option<PartyType>,
        status: Option<PartyStatus>,
    ) -> sea_orm::Select<Party::Entity> {
        let mut query =
            Party::Entity::find().filter(Party::Column::OrganizationId.eq(organization_id));

        if let Some(name) = trim_and_filter_empty(name) {
            query = query.filter(
                Condition::any()
                    .add(Party::Column::DisplayName.contains(name))
                    .add(Party::Column::NamePrimary.contains(name))
                    .add(Party::Column::NameSecondary.contains(name)),
            );
        }

        if let Some(phone) = trim_and_filter_empty(phone) {
            query = query.filter(Party::Column::Phone.contains(phone));
        }

        if let Some(party_type) = party_type {
            query = query.filter(Party::Column::PartyType.eq(party_type));
        }

        if let Some(status) = status {
            query = query.filter(Party::Column::Status.eq(status));
        } else {
            query = query.filter(Party::Column::Status.ne(PartyStatus::Deleted));
        }

        query
    }

    fn apply_page_sort(
        query: sea_orm::Select<Party::Entity>,
        sort_field: PartySortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<Party::Entity> {
        match (sort_field, sort_direction) {
            (PartySortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(Party::Column::CreatedAt)
                .order_by_asc(Party::Column::Id),
            (PartySortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(Party::Column::CreatedAt)
                .order_by_desc(Party::Column::Id),
            (PartySortField::DisplayName, SortDirection::Asc) => query
                .order_by_asc(Party::Column::DisplayName)
                .order_by_asc(Party::Column::Id),
            (PartySortField::DisplayName, SortDirection::Desc) => query
                .order_by_desc(Party::Column::DisplayName)
                .order_by_desc(Party::Column::Id),
            (PartySortField::Code, SortDirection::Asc) => query
                .order_by_asc(Party::Column::Code)
                .order_by_asc(Party::Column::Id),
            (PartySortField::Code, SortDirection::Desc) => query
                .order_by_desc(Party::Column::Code)
                .order_by_desc(Party::Column::Id),
        }
    }

    fn select_party_list_columns<Q>(query: Q) -> Q
    where
        Q: QuerySelect<QueryStatement = sea_orm::sea_query::SelectStatement>,
    {
        query
            .select_only()
            .column(Party::Column::PublicId)
            .column(Party::Column::Code)
            .column(Party::Column::PartyType)
            .column(Party::Column::PartyKind)
            .column(Party::Column::Status)
            .column(Party::Column::Source)
            .column(Party::Column::DisplayName)
            .column(Party::Column::NamePrimary)
            .column(Party::Column::NameSecondary)
            .column(Party::Column::LegalName)
            .column(Party::Column::Phone)
            .column(Party::Column::Email)
            .column(Party::Column::TaxNo)
            .column(Party::Column::CountryIsoCode)
            .column(Party::Column::CurrencyIsoCode)
            .column(Party::Column::AllowCredit)
            .column(Party::Column::PaymentTermsDays)
            .column(Party::Column::CreditLimit)
    }
}
