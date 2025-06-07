use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{middleware::auth::auth_middleware, AppState};

pub mod auth;
pub mod campaigns;
pub mod integrations;
pub mod markdown;
pub mod subscribers;
pub mod templates;
pub mod users;

pub fn create_routes() -> Router<AppState> {
    // 公開ルート（認証不要）
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh_token));

    // 保護されたルート（認証必要）
    let protected_routes = Router::new()
        // ユーザー管理
        .route("/api/users/profile", get(users::get_profile))
        .route("/api/users/profile", put(users::update_profile))
        // テンプレート管理
        .route("/api/templates", get(templates::list_templates))
        .route("/api/templates", post(templates::create_template))
        .route("/api/templates/:id", get(templates::get_template))
        .route("/api/templates/:id", put(templates::update_template))
        .route("/api/templates/:id", delete(templates::delete_template))
        .route(
            "/api/templates/:id/preview",
            post(templates::preview_template),
        )
        // キャンペーン管理
        .route("/api/campaigns", get(campaigns::list_campaigns))
        .route("/api/campaigns", post(campaigns::create_campaign))
        .route("/api/campaigns/:id", get(campaigns::get_campaign))
        .route("/api/campaigns/:id", put(campaigns::update_campaign))
        .route("/api/campaigns/:id", delete(campaigns::delete_campaign))
        .route("/api/campaigns/:id/send", post(campaigns::send_campaign))
        .route(
            "/api/campaigns/:id/schedule",
            post(campaigns::schedule_campaign),
        )
        .route(
            "/api/campaigns/:id/preview",
            get(campaigns::preview_campaign),
        )
        // 購読者管理
        .nest("/api/subscribers", subscribers::router())
        // マークダウン処理
        .route("/api/markdown/render", post(markdown::render_markdown))
        .route("/api/markdown/validate", post(markdown::validate_markdown))
        // GitHub連携
        .route(
            "/api/integrations/github/repos",
            get(integrations::github_repos),
        )
        .route(
            "/api/integrations/github/import",
            post(integrations::import_from_github),
        )
        // 認証ミドルウェアをレイヤーとして適用
        .layer(middleware::from_fn(auth_middleware));

    // ルートを結合
    Router::new().merge(public_routes).merge(protected_routes)
}
