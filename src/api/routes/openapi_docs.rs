use aide::axum::routing::ApiMethodDocs;
use aide::generate::{GenContext, in_context};
use aide::openapi::{Operation, ReferenceOr, Responses, StatusCode as OpenApiStatusCode};
use aide::{OperationInput, OperationOutput};

pub fn method(
    method: &'static str,
    tag: &str,
    id: &str,
    summary: &str,
    docs: impl FnOnce(&mut OperationDoc),
) -> ApiMethodDocs {
    ApiMethodDocs::new(method, operation(tag, id, summary, docs))
}

fn operation(
    tag: &str,
    id: &str,
    summary: &str,
    docs: impl FnOnce(&mut OperationDoc),
) -> Operation {
    let mut operation = Operation {
        operation_id: Some(id.to_owned()),
        summary: Some(summary.to_owned()),
        tags: vec![tag.to_owned()],
        responses: Some(Responses::default()),
        ..Operation::default()
    };

    in_context(|ctx| {
        docs(&mut OperationDoc {
            ctx,
            operation: &mut operation,
        })
    });
    operation
}

pub struct OperationDoc<'a> {
    ctx: &'a mut GenContext,
    operation: &'a mut Operation,
}

impl OperationDoc<'_> {
    pub fn input<T: OperationInput>(&mut self) {
        T::operation_input(self.ctx, self.operation);
    }

    pub fn response<const STATUS: u16, T: OperationOutput>(&mut self) {
        if let Some(response) = T::operation_response(self.ctx, self.operation) {
            self.operation
                .responses
                .get_or_insert_with(Responses::default)
                .responses
                .insert(OpenApiStatusCode::Code(STATUS), ReferenceOr::Item(response));
        }
    }
}
