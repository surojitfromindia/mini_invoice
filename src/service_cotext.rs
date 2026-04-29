use crate::app_state::AppState;
use crate::request_context::RequestContext;

pub struct ServiceContext {
    pub app_state: AppState,
    pub request_context: RequestContext,
}

impl ServiceContext {
    pub fn new(app_state: AppState, request_context: RequestContext) -> ServiceContext {
        ServiceContext {
            app_state,
            request_context,
        }
    }
}
