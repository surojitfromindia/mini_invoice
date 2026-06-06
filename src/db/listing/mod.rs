use schemars::JsonSchema;
use sea_orm::{ConnectionTrait, PaginatorTrait, SelectorTrait};
use serde::Serialize;

use crate::errors::app_error::AppError;
use crate::errors::error_codes;

const MAX_PAGE_SIZE: u64 = 100;

// Centralized shared listing primitives so feature modules can compose filters,
// joins, and select clauses locally while page pagination behavior stays uniform.
#[derive(Debug)]
pub struct ValidatedPagePagination {
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageListResult<T> {
    pub rows: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PageMeta {
    pub page: u64,
    pub per_page: u64,
    pub total_rows: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl<T> PageListResult<T> {
    pub fn map_rows<U, F>(self, mut map_row: F) -> PageListResult<U>
    where
        F: FnMut(T) -> U,
    {
        PageListResult {
            rows: self.rows.into_iter().map(&mut map_row).collect(),
            meta: self.meta,
        }
    }
}

pub fn validate_page_pagination(
    page: u64,
    per_page: u64,
) -> Result<ValidatedPagePagination, AppError> {
    if page == 0 {
        return Err(invalid_pagination("page must be greater than zero"));
    }

    if per_page == 0 {
        return Err(invalid_pagination("perPage must be greater than zero"));
    }

    if per_page > MAX_PAGE_SIZE {
        return Err(invalid_pagination(format!(
            "perPage must be less than or equal to {MAX_PAGE_SIZE}"
        )));
    }

    Ok(ValidatedPagePagination { page, per_page })
}

pub async fn execute_page_query<'db, C, P, T>(
    db: &'db C,
    query: P,
    pagination: ValidatedPagePagination,
) -> Result<PageListResult<T>, AppError>
where
    C: ConnectionTrait,
    P: PaginatorTrait<'db, C> + Send,
    <P as PaginatorTrait<'db, C>>::Selector: SelectorTrait<Item = T> + Send + Sync + 'db,
{
    let paginator = query.paginate(db, pagination.per_page);
    let totals = paginator.num_items_and_pages().await?;
    let rows = paginator.fetch_page(pagination.page - 1).await?;

    Ok(PageListResult {
        rows,
        meta: PageMeta {
            page: pagination.page,
            per_page: pagination.per_page,
            total_rows: totals.number_of_items,
            total_pages: totals.number_of_pages,
            has_next: pagination.page < totals.number_of_pages,
            has_prev: pagination.page > 1,
        },
    })
}

fn invalid_pagination(message: impl Into<String>) -> AppError {
    AppError::BadRequest {
        code: error_codes::LISTING_INVALID_PAGINATION,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_pagination_rejects_zero_page() {
        let error = validate_page_pagination(0, 10).unwrap_err();

        assert_eq!(error.meta().code, error_codes::LISTING_INVALID_PAGINATION);
    }

    #[test]
    fn page_pagination_rejects_zero_page_size() {
        let error = validate_page_pagination(1, 0).unwrap_err();

        assert_eq!(error.meta().code, error_codes::LISTING_INVALID_PAGINATION);
    }

    #[test]
    fn page_list_result_serializes_camel_case_keys() {
        let result = PageListResult {
            rows: vec!["row"],
            meta: PageMeta {
                page: 1,
                per_page: 20,
                total_rows: 25,
                total_pages: 2,
                has_next: true,
                has_prev: false,
            },
        };

        let json = serde_json::to_value(result).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "rows": ["row"],
                "meta": {
                    "page": 1,
                    "perPage": 20,
                    "totalRows": 25,
                    "totalPages": 2,
                    "hasNext": true,
                    "hasPrev": false
                }
            })
        );
    }
}
