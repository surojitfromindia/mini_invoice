use sea_orm::{ConnectionTrait, PaginatorTrait, SelectorTrait};

use crate::api::dto::common_dto::{PageListResult, PageMeta};
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

pub fn validate_page_pagination(
    page: u64,
    per_page: u64,
) -> Result<ValidatedPagePagination, AppError> {
    if page == 0 {
        return Err(invalid_pagination("page must be greater than zero"));
    }

    if per_page == 0 {
        return Err(invalid_pagination("per_page must be greater than zero"));
    }

    if per_page > MAX_PAGE_SIZE {
        return Err(invalid_pagination(format!(
            "per_page must be less than or equal to {MAX_PAGE_SIZE}"
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
}
