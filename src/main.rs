mod config;
mod db;
mod handlers;
mod models;

use axum::{
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Router,
};
use config::AppConfig;
use handlers::*;
use rust_embed::Embed;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

#[derive(Embed)]
#[folder = "ui/"]
pub struct WebAssets;

const INDEX_HTML: &str = concat!(
    include_str!("../ui/index_header.html"),
    include_str!("../ui/tabs/reception.html"),
    include_str!("../ui/tabs/history.html"),
    include_str!("../ui/tabs/physical.html"),
    include_str!("../ui/tabs/clinical.html"),
    include_str!("../ui/tabs/paraclinical.html"),
    include_str!("../ui/tabs/conclusion.html"),
    include_str!("../ui/tabs/export_csv.html"),
    include_str!("../ui/tabs/admin.html"),
    include_str!("../ui/tabs/print.html"),
    include_str!("../ui/index_footer.html"),
);

async fn index_ui_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn static_file_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    // Ưu tiên đọc trực tiếp từ thư mục ui/ trên ổ đĩa nếu tồn tại (hỗ trợ LIVE EDIT sửa file ăn ngay)
    let disk_path = std::path::Path::new("ui").join(&path);
    if disk_path.exists() && disk_path.is_file() {
        if let Ok(data) = tokio::fs::read(&disk_path).await {
            let mime = mime_guess::from_path(&disk_path).first_or_octet_stream();
            return (
                [
                    (axum::http::header::CONTENT_TYPE, mime.as_ref()),
                    (axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                ],
                data,
            )
                .into_response();
        }
    }

    match WebAssets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            (
                [
                    (axum::http::header::CONTENT_TYPE, mime.as_ref()),
                    (axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
                ],
                content.data,
            )
                .into_response()
        }
        None => (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = AppConfig::load();
    tracing::info!("Khởi tạo SQLite tại: {}", cfg.database_path);

    let pool = db::init_db(&cfg.database_path)
        .await
        .expect("Không thể khởi tạo cơ sở dữ liệu SQLite");

    let http_client = reqwest::Client::builder()
        .build()
        .expect("Không thể khởi tạo HTTP Client");

    let addr_str = format!("{}:{}", cfg.server_host, cfg.server_port);
    let addr: SocketAddr = addr_str
        .parse()
        .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 8080)));

    let state = Arc::new(AppState {
        db: RwLock::new(pool),
        config: RwLock::new(cfg.clone()),
        http_client,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index_ui_handler))
        .route("/static/*path", get(static_file_handler))
        // Auth & Quản lý tài khoản (Multi-Role)
        .route("/api/login", post(login_handler))
        .route("/api/me", get(me_handler))
        .route("/api/me/change-password", post(change_my_password_handler))
        .route("/api/me/profile", put(update_my_profile_handler))
        .route("/api/users", get(list_users_handler).post(create_user_handler))
        .route("/api/users/:id", put(update_user_handler))
        .route("/api/users/:id/toggle-lock", post(toggle_lock_user_handler))
        // Tích hợp API HIS
        .route("/api/his", get(fetch_his_patient_handler))
        .route("/api/his/mock", post(mock_his_api_handler))
        .route("/api/admin/his/test", post(test_his_connection_handler))
        // Quản lý Hồ sơ KSK
        .route("/api/records", get(list_records_handler).post(save_record_handler))
        .route("/api/records/by-ma-tn/:ma_tn", get(get_record_by_ma_tn_handler))
        .route("/api/records/:id", get(get_record_handler))
        .route("/api/records/:id/lock", post(lock_record_handler))
        .route("/api/records/:id/unlock", post(unlock_record_handler))
        .route("/api/records/:id/docx", get(export_record_docx_handler))
        // Trích xuất CSV
        .route("/api/export/csv", get(export_csv_handler))
        // Quản lý Danh mục
        .route("/api/catalogs", get(list_catalogs_handler))
        .route("/api/catalogs/versions", get(get_catalog_versions_handler))
        .route("/api/catalogs/bulk", post(bulk_import_catalogs_handler))
        .route("/api/catalogs/bulk-delete", post(bulk_delete_catalogs_handler))
        .route("/api/catalogs/bulk-status", post(bulk_status_catalogs_handler))
        .route("/api/catalogs/:id/set-default", post(set_default_catalog_handler))
        .route("/api/catalogs/:id/toggle-active", post(toggle_active_catalog_handler))
        .route("/api/catalogs/:id", delete(delete_catalog_handler).put(update_catalog_handler))
        // Admin Settings & DB Management
        .route("/api/admin/settings", get(get_admin_settings_handler).post(update_admin_settings_handler))
        .route("/api/admin/db/switch", post(switch_database_handler))
        .route("/api/admin/catalogs/reload-asset", post(reload_asset_catalogs_handler))
        .layer(cors)
        .with_state(state);

    tracing::info!("==============================================================");
    tracing::info!(" HỆ THỐNG KHÁM SỨC KHỎE (KSK) - SINGLE BINARY WEB SERVER ");
    tracing::info!(" Địa chỉ   : http://localhost:{}", cfg.server_port);
    tracing::info!(" Cơ sở     : {} ({})", cfg.ten_cskcb, cfg.ma_cskcb);
    tracing::info!("==============================================================");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Không thể mở cổng mạng cho Web Server");

    axum::serve(listener, app)
        .await
        .expect("Lỗi vận hành Axum Web Server");
}

// touch
