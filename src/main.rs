mod config;
mod crypto;
mod db;
mod group_import;
mod handlers;
mod models;
mod pdf_print;
mod templates;

use axum::{
    response::{Html, IntoResponse},
    routing::{delete, get, post, put},
    Router,
};
use config::AppConfig;
use handlers::*;
use rust_embed::Embed;
use std::{
    fs::OpenOptions,
    io::Write,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
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
    include_str!("../ui/tabs/group.html"),
    include_str!("../ui/tabs/export_csv.html"),
    include_str!("../ui/tabs/admin.html"),
    include_str!("../ui/tabs/print.html"),
    include_str!("../ui/index_footer.html"),
);

async fn http_logging_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();
    let resp = next.run(req).await;
    if path.starts_with("/api/") {
        let status = resp.status();
        let ms = start.elapsed().as_millis();
        if status.is_server_error() {
            tracing::error!("{} {} -> {} ({}ms)", method, path, status.as_u16(), ms);
        } else if status.is_client_error() {
            tracing::warn!("{} {} -> {} ({}ms)", method, path, status.as_u16(), ms);
        } else {
            tracing::info!("{} {} -> {} ({}ms)", method, path, status.as_u16(), ms);
        }
    }
    resp
}

async fn index_ui_handler() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn static_file_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let disk_path = std::path::Path::new("ui").join(&path);
    if disk_path.exists() && disk_path.is_file() {
        if let Ok(data) = tokio::fs::read(&disk_path).await {
            let mime = mime_guess::from_path(&disk_path).first_or_octet_stream();
            return (
                [
                    (axum::http::header::CONTENT_TYPE, mime.as_ref()),
                    (
                        axum::http::header::CACHE_CONTROL,
                        "no-cache, no-store, must-revalidate",
                    ),
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
                    (
                        axum::http::header::CACHE_CONTROL,
                        "no-cache, no-store, must-revalidate",
                    ),
                ],
                content.data,
            )
                .into_response()
        }
        None => (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

fn setup_logging_and_panic_hook() {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ksk_app.log")
        .ok();
    let log_mutex = Arc::new(Mutex::new(log_file));

    let panic_log = log_mutex.clone();
    std::panic::set_hook(Box::new(move |info| {
        let bt = std::backtrace::Backtrace::force_capture();
        let msg = format!(
            "\n[{}] PANIC: {}\nBacktrace:\n{}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            info,
            bt
        );
        eprintln!("{}", msg);
        if let Ok(mut guard) = panic_log.lock() {
            if let Some(ref mut f) = *guard {
                let _ = f.write_all(msg.as_bytes());
                let _ = f.flush();
            }
        }
    }));

    let writer_log = log_mutex;
    let make_writer = move || LogFileWriter(writer_log.clone());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with_writer(make_writer)
        .with_ansi(false)
        .try_init();
}

struct LogFileWriter(Arc<Mutex<Option<std::fs::File>>>);

impl Write for LogFileWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = std::io::stderr().write_all(buf);
        if let Ok(mut guard) = self.0.lock() {
            if let Some(ref mut f) = *guard {
                let _ = f.write_all(buf);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let _ = std::io::stderr().flush();
        if let Ok(mut guard) = self.0.lock() {
            if let Some(ref mut f) = *guard {
                let _ = f.flush();
            }
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    setup_logging_and_panic_hook();

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
        master_key: RwLock::new(None),
        http_client,
        his_push_tx: None,
    });

    // Tác vụ sao lưu database tự động trên máy chủ theo chu kỳ
    let backup_state = state.clone();
    tokio::spawn(async move {
        loop {
            let (enabled, interval_mins, bk_dir, keep_cnt) = {
                let c = backup_state.config.read().await;
                (
                    c.auto_backup_enabled,
                    c.backup_interval_minutes.max(5),
                    c.backup_dir_path.clone(),
                    c.backup_keep_count.max(3),
                )
            };
            tokio::time::sleep(Duration::from_secs(interval_mins * 60)).await;
            if !enabled {
                continue;
            }
            let db_guard = backup_state.db.read().await;
            if let Ok(info) = db::create_server_backup(&*db_guard, &bk_dir, "auto").await {
                tracing::info!("Auto-backup server DB: {}", info.filename);
                let all = db::list_server_backups(&bk_dir);
                let autos: Vec<_> = all
                    .into_iter()
                    .filter(|b| b.filename.contains("_auto"))
                    .collect();
                for old in autos.into_iter().skip(keep_cnt) {
                    let _ = db::delete_server_backup(&bk_dir, &old.filename);
                }
            }
        }
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(index_ui_handler))
        .route("/static/*path", get(static_file_handler))
        // Setup & Auth
        .route("/api/setup/status", get(setup_status_handler))
        .route("/api/setup/init", post(setup_init_handler))
        .route("/api/auth/recover", post(recover_admin_handler))
        .route("/api/login", post(login_handler))
        .route("/api/me", get(me_handler))
        .route("/api/me/change-password", post(change_my_password_handler))
        .route("/api/me/profile", put(update_my_profile_handler))
        .route("/api/users", get(list_users_handler).post(create_user_handler))
        .route("/api/users/:id", put(update_user_handler))
        .route("/api/users/:id/toggle-lock", post(toggle_lock_user_handler))
        // Tích hợp HIS (Chế độ 1: API HIS, Chế độ 2: AHK Bridge Push)
        .route("/api/his", get(fetch_his_patient_handler))
        .route("/api/his/push", post(push_his_patient_handler))
        .route("/api/his/mock", post(mock_his_api_handler))
        .route("/api/admin/his/test", post(test_his_connection_handler))
        // Metadata 6 Mẫu Phiếu KSK
        .route("/api/templates", get(get_templates_metadata_handler))
        // Quản lý Hồ sơ KSK & In PDF A4
        .route("/api/records", get(list_records_handler).post(save_record_handler))
        .route("/api/records/by-ma-tn/:ma_tn", get(get_record_by_ma_tn_handler))
        .route("/api/records/:id", get(get_record_handler))
        .route("/api/records/:id/complete", post(complete_record_handler))
        .route("/api/records/:id/lock", post(lock_record_handler))
        .route("/api/records/:id/unlock", post(unlock_record_handler))
        .route("/api/records/:id/templates", put(update_record_templates_handler))
        .route("/api/records/:id/pdf", get(export_record_pdf_handler))
        .route("/api/records/:id/docx", get(export_record_docx_handler))
        // Khám đoàn (Công ty, Hợp đồng, Import & In cả đoàn)
        .route("/api/companies", get(list_companies_handler).post(create_company_handler))
        .route("/api/companies/:id", put(update_company_handler))
        .route("/api/companies/:id/toggle-active", post(toggle_company_active_handler))
        .route("/api/contracts", get(list_contracts_handler).post(create_contract_handler))
        .route("/api/contracts/sample-csv", get(download_sample_csv_handler))
        .route("/api/contracts/:id", put(update_contract_handler))
        .route("/api/contracts/:id/records", get(list_contract_records_handler))
        .route("/api/contracts/:id/pdf", get(export_contract_pdf_handler))
        .route("/api/contracts/:id/import-preview", post(import_contract_preview_handler))
        .route("/api/contracts/:id/import-commit", post(import_contract_commit_handler))
        // Trích xuất & API Fetch dữ liệu (CSV / JSON)
        .route("/api/export/csv", get(export_csv_handler))
        .route(
            "/api/export/api-config",
            get(get_export_api_config_handler).post(save_export_api_config_handler),
        )
        .route("/api/export/records", get(fetch_export_records_handler))
        // Quản lý Danh mục
        .route("/api/catalogs", get(list_catalogs_handler))
        .route("/api/catalogs/versions", get(get_catalog_versions_handler))
        .route("/api/catalogs/bulk", post(bulk_import_catalogs_handler))
        .route("/api/catalogs/bulk-delete", post(bulk_delete_catalogs_handler))
        .route("/api/catalogs/bulk-status", post(bulk_status_catalogs_handler))
        .route("/api/catalogs/:id/set-default", post(set_default_catalog_handler))
        .route("/api/catalogs/:id/toggle-active", post(toggle_active_catalog_handler))
        .route("/api/catalogs/:id", delete(delete_catalog_handler).put(update_catalog_handler))
        // Admin Settings & Sao lưu DB trên Máy chủ
        .route(
            "/api/admin/settings",
            get(get_admin_settings_handler).post(update_admin_settings_handler),
        )
        .route("/api/admin/db/switch", post(switch_database_handler))
        .route("/api/admin/db/backups", get(list_db_backups_handler))
        .route("/api/admin/db/backup", post(create_db_backup_handler))
        .route("/api/admin/db/restore", post(restore_db_backup_handler))
        .route(
            "/api/admin/db/backups/:filename",
            get(download_db_backup_handler).delete(delete_db_backup_handler),
        )
        .route("/api/admin/catalogs/reload-asset", post(reload_asset_catalogs_handler))
        .route("/api/phan-loai/tree", get(get_phan_loai_tree_handler))
        .route("/api/admin/phan-loai/item", post(upsert_phan_loai_item_handler))
        .route("/api/admin/phan-loai/reload-xlsx", post(reload_phan_loai_xlsx_handler))
        .layer(axum::middleware::from_fn(http_logging_middleware))
        .layer(cors)
        .with_state(state);

    tracing::info!("==============================================================");
    tracing::info!(" HỆ THỐNG KHÁM SỨC KHỎE (KSK) - SINGLE BINARY WEB SERVER ");
    tracing::info!(" Địa chỉ   : http://{}:{}", cfg.server_host, cfg.server_port);
    tracing::info!(" Cơ sở     : {} ({})", cfg.ten_cskcb, cfg.ma_cskcb);
    tracing::info!("==============================================================");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Không thể mở cổng mạng cho Web Server");

    axum::serve(listener, app)
        .await
        .expect("Lỗi vận hành Axum Web Server");
}
