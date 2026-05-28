// Shared helpers for small cross-cutting utility logic that does not fit a
// domain-specific module.
pub fn trim_and_filter_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}
