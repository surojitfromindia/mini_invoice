use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicIdResponse {
    pub public_id: String,
}

impl PublicIdResponse {
    pub fn new(public_id: impl Into<String>) -> Self {
        Self {
            public_id: public_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ActionStatusResponse {
    pub status: String,
}

impl ActionStatusResponse {
    pub fn new(status: impl Into<String>) -> Self {
        Self {
            status: status.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PagePaginationQuery {
    #[serde(default = "default_page")]
    #[serde(deserialize_with = "deserialize_u64")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    #[serde(deserialize_with = "deserialize_u64")]
    pub per_page: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageListResult<T> {
    pub rows: Vec<T>,
    pub meta: PageMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageMeta {
    pub page: u64,
    pub per_page: u64,
    pub total_rows: u64,
    pub total_pages: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

const fn default_page() -> u64 {
    1
}

const fn default_per_page() -> u64 {
    20
}

fn deserialize_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum U64Value {
        Number(u64),
        String(String),
    }

    match U64Value::deserialize(deserializer)? {
        U64Value::Number(value) => Ok(value),
        U64Value::String(value) => value.parse::<u64>().map_err(serde::de::Error::custom),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_pagination_defaults_are_stable() {
        let query: PagePaginationQuery = serde_json::from_str("{}").unwrap();

        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 20);
    }

    #[test]
    fn page_pagination_accepts_string_values() {
        let query: PagePaginationQuery = serde_json::from_value(serde_json::json!({
            "page": "1",
            "per_page": "20"
        }))
        .unwrap();

        assert_eq!(query.page, 1);
        assert_eq!(query.per_page, 20);
    }

    #[test]
    fn action_status_response_serializes_shape() {
        let response = ActionStatusResponse::new("accepted");

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json, serde_json::json!({ "status": "accepted" }));
    }
}
