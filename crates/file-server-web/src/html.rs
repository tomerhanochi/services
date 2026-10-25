use axum::response::{IntoResponse, Response};
use htmlb::prelude::IntoHtml;

pub(crate) struct Html<T>(pub T);
impl<T: IntoHtml> IntoResponse for Html<T> {
    #[hotpath::measure(impl_type = "Html")]
    fn into_response(self) -> Response {
        axum::response::Html(self.0.to_html()).into_response()
    }
}
