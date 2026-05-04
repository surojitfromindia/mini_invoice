use crate::app_state::AppState;

pub struct ServiceContext {
    pub app_state: AppState,
}

#[derive(Clone)]
pub struct RequestContext {
    pub request_timezone: String,
}
