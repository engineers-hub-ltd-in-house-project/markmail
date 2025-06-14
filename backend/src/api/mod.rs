use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

use crate::{middleware::auth::auth_middleware, AppState};

pub mod auth;
pub mod campaigns;
pub mod email;
pub mod forms;
pub mod integrations;
pub mod markdown;
pub mod sequences;
pub mod subscribers;
pub mod templates;
pub mod users;

pub fn create_routes() -> Router<AppState> {
    // 公開ルート（認証不要）
    let public_routes = Router::new()
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/refresh", post(auth::refresh_token))
        // フォームの公開エンドポイント
        .route("/api/forms/:id/public", get(forms::get_public_form))
        .route("/api/forms/:id/submit", post(forms::submit_form));

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
        .route(
            "/api/campaigns/:id/subscribers",
            get(campaigns::get_campaign_subscribers),
        )
        // 購読者管理
        .nest("/api/subscribers", subscribers::router())
        // メール送信（開発環境のみ）
        .nest("/api/email", email::router())
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
        // フォーム管理
        .route("/api/forms", get(forms::get_forms))
        .route("/api/forms", post(forms::create_form))
        .route("/api/forms/:id", get(forms::get_form))
        .route("/api/forms/:id", put(forms::update_form))
        .route("/api/forms/:id", delete(forms::delete_form))
        .route(
            "/api/forms/:id/submissions",
            get(forms::get_form_submissions),
        )
        // シーケンス管理
        .route("/api/sequences", get(sequences::get_sequences))
        .route("/api/sequences", post(sequences::create_sequence))
        .route("/api/sequences/:id", get(sequences::get_sequence))
        .route("/api/sequences/:id", put(sequences::update_sequence))
        .route("/api/sequences/:id", delete(sequences::delete_sequence))
        .route(
            "/api/sequences/:id/full",
            get(sequences::get_sequence_with_steps),
        )
        .route(
            "/api/sequences/:id/steps",
            post(sequences::create_sequence_step),
        )
        .route(
            "/api/sequences/:sequence_id/steps/:step_id",
            put(sequences::update_sequence_step),
        )
        .route(
            "/api/sequences/:sequence_id/steps/:step_id",
            delete(sequences::delete_sequence_step),
        )
        .route(
            "/api/sequences/:id/activate",
            post(sequences::activate_sequence),
        )
        .route("/api/sequences/:id/pause", post(sequences::pause_sequence))
        // 認証ミドルウェアをレイヤーとして適用
        .layer(middleware::from_fn(auth_middleware));

    // ルートを結合
    Router::new().merge(public_routes).merge(protected_routes)
}
