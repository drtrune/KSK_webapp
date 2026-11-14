#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod crypto;
mod db;
mod gui;
mod handlers;
mod models;
mod updater;

use axum::{routing::post, Router};
use config::AppConfig;
use gui::AppLaunchCtx;
use handlers::{mock_his_api_handler, push_his_patient_handler, AppState};
use std::{
    fs::OpenOptions,
    io::Write,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::{broadcast, RwLock};
use tower_http::cors::{Any, CorsLayer};

fn setup_logging_and_panic_hook() {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ksk_app.log")
        .ok();
    let log_mutex = Arc::new(Mutex::new(log_file));

    // Ghi toàn bộ panic (kể cả trong Dioxus Scope) ra file ksk_app.log
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

    // Ghi tracing log ra ksk_app.log
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

fn main() {
    setup_logging_and_panic_hook();
    updater::cleanup_old_exe_on_startup();

    tracing::info!("=== Khoi dong KSK Native Desktop (GPUI) ===");

    let cfg = AppConfig::load();
    tracing::info!("Database path: {}", cfg.database_path);

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
        master_key: RwLock::new(None),
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
            tracing::info!("Local HIS Bridge IPC dang lang nghe tai http://{}", addr);
            let _ = axum::serve(listener, app).await;
        } else {
            tracing::warn!("Cong Local IPC {} da duoc tien trinh khac su dung", ipc_port);
        }
    });

    let launch_ctx = AppLaunchCtx {
        rt: rt.handle().clone(),
        state,
        his_tx,
        initial_cfg: cfg.clone(),
    };

    let window_title = format!("Quản Lý Khám Sức Khỏe (KSK) - {}", cfg.ten_cskcb);

    use gpui::AppContext as _;
    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);

        let bounds = gpui::Bounds::centered(
            None,
            gpui::size(gpui::px(1360.0), gpui::px(840.0)),
            cx,
        );
        let options = gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(window_title.clone().into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let ctx = launch_ctx.clone();
        match gpui_kit::open_window(options, cx, move |window, cx| {
            tracing::info!("Tao KskGpuiApp view...");
            let app_view = cx.new(|cx| gui::KskGpuiApp::new(ctx, window, cx));
            cx.new(|cx| gpui_component::Root::new(app_view, window, cx))
        }) {
            Ok(_) => tracing::info!("Da mo cua so GPUI thanh cong!"),
            Err(e) => tracing::error!("Khong the mo cua so GPUI: {:?}", e),
        }
        cx.activate(true);
    });
    tracing::info!("GPUI Application::run da ket thuc.");

    drop(rt);
}
