// ログ設定
// TODO: 構造化ログ、リクエスト/レスポンスログの実装

use tower_http::trace::{self, TraceLayer};
use tracing::Level;

pub fn logging_layer(
) -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
}
