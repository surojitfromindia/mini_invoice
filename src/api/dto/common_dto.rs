use serde::{Deserialize, Deserializer, Serialize};

pub trait IntoServiceInput<T> {
    fn into_service_input(self) -> T;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicIdResponse {
    pub public_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionStatusResponse {
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "PagePaginationQueryRaw")]
pub struct PagePaginationQuery {
    pub page: u64,
    pub per_page: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}


#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct PagePaginationQueryRaw {
    #[serde(default = "default_page_value")]
    page: U64Value,
    #[serde(default = "default_per_page_value")]
    per_page: U64Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum U64Value {
    Number(u64),
    String(String),
}

impl U64Value {
    fn into_u64(self) -> Result<u64, String> {
        match self {
            U64Value::Number(value) => Ok(value),
            U64Value::String(value) => value.parse::<u64>().map_err(|error| error.to_string()),
        }
    }
}

impl TryFrom<PagePaginationQueryRaw> for PagePaginationQuery {
    type Error = String;

    fn try_from(value: PagePaginationQueryRaw) -> Result<Self, Self::Error> {
        Ok(Self {
            page: value.page.into_u64()?,
            per_page: value.per_page.into_u64()?,
        })
    }
}

const fn default_page_value() -> U64Value {
    U64Value::Number(1)
}

const fn default_per_page_value() -> U64Value {
    U64Value::Number(20)
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
        let response = ActionStatusResponse {
            status: "accepted".to_string(),
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json, serde_json::json!({ "status": "accepted" }));
    }

    #[test]
    fn public_id_response_serializes_camel_case_keys() {
        let response = PublicIdResponse {
            public_id: "pub_123".to_string(),
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(json, serde_json::json!({ "publicId": "pub_123" }));
    }
}
