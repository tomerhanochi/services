use futures::Stream;
use htmlb::prelude::*;

trait IntoHtmlStream: IntoHtml {
    fn render_stream(&self) -> impl Stream;
}
