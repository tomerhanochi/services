use axum::response::{IntoResponse, Response};
use htmlb::IntoHtml;

pub(crate) struct Html<T>(pub T);
impl<T: IntoHtml> IntoResponse for Html<T> {
    fn into_response(self) -> Response {
        axum::response::Html(self.0.to_html()).into_response()
    }
}
