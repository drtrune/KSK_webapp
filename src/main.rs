#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod db;
mod gui;
mod handlers;
mod models;
mod updater;

use axum::{routing::post, Router};
use config::AppConfig;
use gui::AppLaunchCtx;
use handlers::{mock_his_api_handler, push_his_patient_handler, AppState};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

fn main() {
    updater::cleanup_old_exe_on_startup();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = AppConfig::load();
    let rt = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Không thể khởi tạo Tokio Runtime"),
    );

    let pool = rt
        .block_on(db::init_db(&cfg.database_path))
        .expect("Không thể khởi tạo cơ sở dữ liệu SQLite");

    let http_client = reqwest::Client::builder()
        .build()
        .expect("Không thể khởi tạo HTTP Client");

    let (his_tx, _his_rx) = broadcast::channel::<String>(32);

    let state = Arc::new(AppState {
        db: RwLock::new(pool),
        config: RwLock::new(cfg.clone()),
        http_client,
        his_push_tx: Some(his_tx.clone()),
    });

    // Khởi chạy Local IPC Receiver (127.0.0.1:18080) để nhận dữ liệu từ AutoHotkey v2 UIA Bridge
    let ipc_state = state.clone();
    let ipc_port = cfg.local_ipc_port;
    rt.spawn(async move {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = Router::new()
            .route("/api/his/push", post(push_his_patient_handler))
            .route("/api/his/mock", post(mock_his_api_handler))
            .layer(cors)
            .with_state(ipc_state);

        let addr = SocketAddr::from(([127, 0, 0, 1], ipc_port));
        if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
            tracing::info!("Local HIS Bridge IPC đang lắng nghe tại http://{}", addr);
            let _ = axum::serve(listener, app).await;
        } else {
            tracing::warn!("Cổng Local IPC {} đã được tiến trình khác sử dụng", ipc_port);
        }
    });

    let launch_ctx = AppLaunchCtx {
        rt,
        state,
        his_tx,
        initial_cfg: cfg.clone(),
    };

    let win_cfg = dioxus::desktop::Config::new().with_window(
        dioxus::desktop::WindowBuilder::new()
            .with_title(format!("Quản Lý Khám Sức Khỏe (KSK) - {}", cfg.ten_cskcb))
            .with_inner_size(dioxus::desktop::LogicalSize::new(1360.0, 840.0))
            .with_min_inner_size(dioxus::desktop::LogicalSize::new(1024.0, 680.0)),
    );

    dioxus::LaunchBuilder::desktop()
        .with_context(launch_ctx)
        .with_cfg(win_cfg)
        .launch(gui::App);
}
