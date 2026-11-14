pub mod components;
pub mod sidebar;
pub mod tab_export_admin;
pub mod tabs_exam;

use crate::config::AppConfig;
use crate::db::{get_db_info, hash_password, init_db, seed_catalogs_from_asset};
use crate::handlers::{
    generate_csv_bytes, generate_docx_bytes, query_filtered_records, SharedState,
};
use crate::models::*;
use crate::updater::{self, CURRENT_VERSION};
use components::map_get;
use dioxus::prelude::*;
use serde_json::{json, Map, Value};
use sidebar::Sidebar;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use tab_export_admin::{TabAdmin, TabExportCsv};
use tabs_exam::{
    TabClinical, TabConclusion, TabHistory, TabParaclinical, TabPhysical, TabReception,
};
use tokio::runtime::Handle;
use tokio::sync::broadcast;

const STYLES_CSS: &str = include_str!("styles.css");

#[derive(Clone)]
pub struct AppLaunchCtx {
    pub rt: Handle,
    pub state: SharedState,
    pub his_tx: broadcast::Sender<String>,
    pub initial_cfg: AppConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppTab {
    Reception,
    History,
    Physical,
    Clinical,
    Paraclinical,
    Conclusion,
    ExportCsv,
    Admin,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KskRecordSummary {
    pub id: i64,
    pub ma_tiep_nhan: String,
    pub ho_ten: String,
    pub ngay_vao: String,
    pub status: i64,
    pub phan_loai_sk: String,
}

fn section_role_and_fields(section: &str) -> (&'static str, &'static [&'static str]) {
    match section {
        "reception" => ("reception", FIELDS_RECEPTION),
        "history" => ("tiensu", FIELDS_TIENSU),
        "physical" => ("bs_theluc", FIELDS_BS_THELUC),
        "noi" => ("bs_noi", FIELDS_BS_NOI),
        "ngoai" => ("bs_ngoai", FIELDS_BS_NGOAI),
        "dalieu" => ("bs_dalieu", FIELDS_BS_DALIEU),
        "san" => ("bs_san", FIELDS_BS_SAN),
        "mat" => ("bs_mat", FIELDS_BS_MAT),
        "tmh" => ("bs_tmh", FIELDS_BS_TMH),
        "rhm" => ("bs_rhm", FIELDS_BS_RHM),
        "cls" => ("bs_cls", FIELDS_BS_CLS),
        "ketluan" => ("bs_ketluan", FIELDS_BS_KETLUAN),
        _ => ("admin", &[]),
    }
}

fn run_sync<F, T>(rt: &Handle, fut: F) -> T
where
    F: std::future::Future<Output = T> + Send,
    T: Send,
{
    if tokio::runtime::Handle::try_current().is_ok() {
        let rt_clone = rt.clone();
        std::thread::scope(|s| {
            s.spawn(|| rt_clone.block_on(fut))
                .join()
                .expect("Async thread panicked")
        })
    } else {
        rt.block_on(fut)
    }
}

fn load_catalogs_sync(rt: &Handle, state: &SharedState) -> HashMap<String, Vec<CatalogItem>> {
    let types = [
        "DAN_TOC",
        "NGHE_NGHIEP",
        "TINH_TP",
        "XA_PHUONG",
        "DOI_TUONG",
        "NGUON_CHI_TRA",
        "NOI_CAP",
        "ICD10",
    ];
    let mut map = HashMap::new();
    run_sync(rt, async {
        let db = state.db.read().await;
        for t in types {
            let items: Vec<CatalogItem> = sqlx::query_as(
                "SELECT id, cat_type, code, name, parent_code, is_active, is_default FROM catalogs WHERE cat_type = ? ORDER BY is_default DESC, name ASC",
            )
            .bind(t)
            .fetch_all(&*db)
            .await
            .unwrap_or_default();
            map.insert(t.to_string(), items);
        }
    });
    map
}

#[allow(non_snake_case)]
pub fn App() -> Element {
    let ctx = use_context::<AppLaunchCtx>();
    let today = chrono::Local::now().format("%d/%m/%Y").to_string();

    let mut cfg = use_signal(|| ctx.initial_cfg.clone());
    let mut current_user = use_signal(|| None::<User>);
    let mut login_username = use_signal(String::new);
    let mut login_password = use_signal(String::new);
    let mut login_error = use_signal(|| None::<String>);

    let mut show_profile_modal = use_signal(|| false);
    let mut profile_full_name = use_signal(String::new);
    let mut profile_old_pw = use_signal(String::new);
    let mut profile_new_pw = use_signal(String::new);

    let update_available = use_signal(|| {
        if ctx.initial_cfg.auto_check_update {
            updater::check_for_update(&ctx.initial_cfg.update_dir_path).unwrap_or(None)
        } else {
            None
        }
    });

    let mut active_tab = use_signal(|| AppTab::Reception);
    let sidebar_open = use_signal(|| true);
    let filter_from_date = use_signal(|| today.clone());
    let filter_to_date = use_signal(|| today.clone());
    let filter_status = use_signal(|| None::<i64>);
    let filter_q = use_signal(String::new);

    let mut records = use_signal(Vec::<KskRecordSummary>::new);
    let mut selected_record_id = use_signal(|| None::<i64>);
    let mut selected_record_status = use_signal(|| 0i64);
    let mut form = use_signal(Map::<String, Value>::new);
    let mut editing_sections = use_signal(HashMap::<String, bool>::new);
    let clinical_full_mode = use_signal(|| false);

    let mut catalogs_cache = use_signal(|| load_catalogs_sync(&ctx.rt, &ctx.state));

    // Export CSV
    let csv_from_date = use_signal(|| today.clone());
    let csv_to_date = use_signal(|| today);
    let csv_status = use_signal(|| None::<i64>);
    let csv_q = use_signal(String::new);
    let csv_workplace = use_signal(String::new);
    let csv_classification = use_signal(String::new);
    let mut csv_rows = use_signal(Vec::<KskRecordRow>::new);
    let mut csv_selected_ids = use_signal(HashSet::<i64>::new);

    // Admin
    let mut db_info = use_signal(|| None::<DbInfo>);
    let publish_changelog = use_signal(String::new);
    let mut users_list = use_signal(Vec::<User>::new);
    let mut new_user_username = use_signal(String::new);
    let mut new_user_password = use_signal(String::new);
    let mut new_user_fullname = use_signal(String::new);
    let new_user_roles = use_signal(|| vec!["reception".to_string()]);
    let catalog_type_filter = use_signal(|| "DAN_TOC".to_string());
    let catalog_search = use_signal(String::new);
    let mut catalog_rows = use_signal(Vec::<CatalogItem>::new);

    // Toast: (message, is_error)
    let mut toast = use_signal(|| None::<(String, bool)>);

    let mut refresh_records = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move || {
            let f_from = filter_from_date.read().clone();
            let f_to = filter_to_date.read().clone();
            let f_st = *filter_status.read();
            let f_q = filter_q.read().clone();
            let rows = run_sync(&rt, async {
                let db = state.db.read().await;
                query_filtered_records(
                    &*db,
                    Some(f_from.as_str()),
                    Some(f_to.as_str()),
                    f_st,
                    Some(f_q.as_str()),
                    None,
                    None,
                    None,
                    Some(300),
                    false,
                )
                .await
            });
            let list = rows
                .into_iter()
                .map(|r| {
                    let map: Map<String, Value> =
                        serde_json::from_str(&r.payload_json).unwrap_or_default();
                    let pl = map_get(&map, "PHAN_LOAI_SK");
                    KskRecordSummary {
                        id: r.id,
                        ma_tiep_nhan: r.ma_tiep_nhan,
                        ho_ten: r.ho_ten,
                        ngay_vao: r.ngay_vao,
                        status: r.status,
                        phan_loai_sk: pl,
                    }
                })
                .collect();
            records.set(list);
        }
    };

    let mut load_record = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move |record_id: i64| {
            let row: Option<KskRecordRow> = run_sync(&rt, async {
                let db = state.db.read().await;
                sqlx::query_as("SELECT * FROM ksk_records WHERE id = ?")
                    .bind(record_id)
                    .fetch_optional(&*db)
                    .await
                    .unwrap_or(None)
            });
            if let Some(r) = row {
                selected_record_id.set(Some(r.id));
                selected_record_status.set(r.status);
                form.set(serde_json::from_str(&r.payload_json).unwrap_or_default());
                editing_sections.write().clear();
            }
        }
    };

    // Lắng nghe push real-time từ AHK v2 UIA Bridge / Mock HIS Push
    let his_tx_sub = ctx.his_tx.clone();
    let refresh_for_his = refresh_records.clone();
    let load_for_his = load_record.clone();
    use_effect(move || {
        let mut rx = his_tx_sub.subscribe();
        let mut r_his = refresh_for_his.clone();
        let mut l_his = load_for_his.clone();
        spawn(async move {
            while let Ok(ma_tn) = rx.recv().await {
                toast.set(Some((format!("📡 HIS Bridge vừa đẩy hồ sơ: {}", ma_tn), false)));
                if current_user.read().is_some() {
                    r_his();
                    let found_id = records
                        .read()
                        .iter()
                        .find(|r| r.ma_tiep_nhan == ma_tn)
                        .map(|r| r.id);
                    if let Some(id) = found_id {
                        l_his(id);
                        active_tab.set(AppTab::Reception);
                    }
                }
            }
        });
    });

    let mut refresh_db_info = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move || {
            let path = cfg.read().database_path.clone();
            let info = run_sync(&rt, async {
                let db = state.db.read().await;
                get_db_info(&*db, &path).await
            });
            db_info.set(Some(info));
        }
    };

    let mut refresh_users = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move || {
            let rows: Vec<User> = run_sync(&rt, async {
                let db = state.db.read().await;
                sqlx::query_as("SELECT * FROM users ORDER BY id ASC")
                    .fetch_all(&*db)
                    .await
                    .unwrap_or_default()
            });
            users_list.set(rows);
        }
    };

    let mut refresh_admin_catalogs = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move || {
            let ct = catalog_type_filter.read().clone();
            let q = format!("%{}%", catalog_search.read().trim());
            let rows: Vec<CatalogItem> = run_sync(&rt, async {
                let db = state.db.read().await;
                sqlx::query_as(
                    "SELECT id, cat_type, code, name, parent_code, is_active, is_default FROM catalogs WHERE cat_type = ? AND (code LIKE ? OR name LIKE ?) ORDER BY is_default DESC, name ASC LIMIT 200",
                )
                .bind(&ct)
                .bind(&q)
                .bind(&q)
                .fetch_all(&*db)
                .await
                .unwrap_or_default()
            });
            catalog_rows.set(rows);
        }
    };

    let mut switch_database = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        let mut r_rec = refresh_records.clone();
        let mut r_db = refresh_db_info.clone();
        move |new_path: String| {
            let p = new_path.trim().to_string();
            if p.is_empty() {
                toast.set(Some(("Đường dẫn DB không được để trống!".to_string(), true)));
                return;
            }
            let res = run_sync(&rt, async {
                let new_pool = init_db(&p).await.map_err(|e| e.to_string())?;
                let mut db_guard = state.db.write().await;
                *db_guard = new_pool;
                Ok::<(), String>(())
            });
            match res {
                Ok(_) => {
                    cfg.write().database_path = p;
                    let c_clone = cfg.read().clone();
                    let _ = c_clone.save();
                    run_sync(&rt, async {
                        *state.config.write().await = c_clone;
                    });
                    catalogs_cache.set(load_catalogs_sync(&rt, &state));
                    r_db();
                    if current_user.read().is_some() {
                        r_rec();
                    }
                    toast.set(Some(("Đã kết nối cơ sở dữ liệu mới thành công!".to_string(), false)));
                }
                Err(e) => {
                    toast.set(Some((format!("Không thể mở DB: {}", e), true)));
                }
            }
        }
    };

    let mut do_login = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        let mut r_rec = refresh_records.clone();
        let mut r_db = refresh_db_info.clone();
        let mut r_usr = refresh_users.clone();
        let mut r_cat = refresh_admin_catalogs.clone();
        move || {
            let u_trim = login_username.read().trim().to_string();
            let p_trim = login_password.read().trim().to_string();
            if u_trim.is_empty() || p_trim.is_empty() {
                login_error.set(Some("Vui lòng nhập tên đăng nhập và mật khẩu.".to_string()));
                return;
            }
            let p_hash = hash_password(&p_trim);
            let res: Result<Option<User>, String> = run_sync(&rt, async {
                let db = state.db.read().await;
                sqlx::query_as("SELECT * FROM users WHERE username = ? AND password_hash = ?")
                    .bind(&u_trim)
                    .bind(&p_hash)
                    .fetch_optional(&*db)
                    .await
                    .map_err(|e| e.to_string())
            });
            match res {
                Ok(Some(u)) => {
                    if u.is_locked == 1 {
                        login_error.set(Some("Tài khoản đã bị khóa bởi Quản trị viên.".to_string()));
                        return;
                    }
                    let is_admin = u.has_role("admin");
                    profile_full_name.set(u.full_name.clone());
                    current_user.set(Some(u));
                    login_error.set(None);
                    login_password.set(String::new());
                    r_rec();
                    r_db();
                    if is_admin {
                        r_usr();
                        r_cat();
                    }
                    toast.set(Some(("Đăng nhập thành công!".to_string(), false)));
                }
                Ok(None) => login_error.set(Some("Sai tên đăng nhập hoặc mật khẩu.".to_string())),
                Err(e) => login_error.set(Some(format!("Lỗi kết nối CSDL: {}", e))),
            }
        }
    };

    // Màn hình Đăng nhập
    if current_user.read().is_none() {
        let upd = update_available.read().clone();
        let cfg_snap = cfg.read().clone();
        let err_msg = login_error.read().clone();
        let mut do_login_btn = do_login.clone();
        let mut switch_db_pick = switch_database.clone();
        let mut switch_db_btn = switch_database.clone();
        return rsx! {
            style { "{STYLES_CSS}" }
            div { class: "login-wrapper",
                div { class: "login-card",
                    div { style: "text-align: center;",
                        div { style: "font-size: 20px; font-weight: 800; color: #0f766e;", "🏥 HỆ THỐNG KHÁM SỨC KHỎE (KSK)" }
                        div { style: "font-size: 12.5px; color: #64748b; margin-top: 4px;", "{cfg_snap.ten_cskcb} • Phiên bản v{CURRENT_VERSION} (Dioxus)" }
                    }

                    if let Some(m) = upd {
                        div { style: "background: #fef3c7; border: 1px solid #f59e0b; padding: 10px; border-radius: 7px; display: flex; justify-content: space-between; align-items: center;",
                            span { style: "font-weight: 700; color: #92400e; font-size: 12.5px;", "🆕 Có phiên bản mới v{m.version}" }
                            button {
                                class: "btn btn-amber",
                                onclick: move |_| {
                                    if let Err(e) = updater::apply_update_and_restart(&cfg.read().update_dir_path, &m) {
                                        login_error.set(Some(e));
                                    }
                                },
                                "Cập nhật ngay"
                            }
                        }
                    }

                    div { class: "field",
                        label { "Tên đăng nhập" }
                        input {
                            r#type: "text",
                            placeholder: "VD: admin / letan / bsnoi...",
                            value: "{login_username}",
                            oninput: move |e| login_username.set(e.value()),
                        }
                    }
                    div { class: "field",
                        label { "Mật khẩu" }
                        input {
                            r#type: "password",
                            placeholder: "Nhập mật khẩu",
                            value: "{login_password}",
                            oninput: move |e| login_password.set(e.value()),
                            onkeypress: move |e| {
                                if e.key() == Key::Enter {
                                    do_login();
                                }
                            }
                        }
                    }

                    if let Some(err) = err_msg {
                        div { style: "color: #dc2626; font-size: 12.5px; font-weight: 600;", "{err}" }
                    }

                    button {
                        class: "btn btn-primary",
                        style: "width: 100%; justify-content: center; padding: 9px;",
                        onclick: move |_| do_login_btn(),
                        "ĐĂNG NHẬP"
                    }

                    div { style: "border-top: 1px solid #e2e8f0; padding-top: 10px;",
                        div { style: "font-size: 11.5px; color: #64748b; margin-bottom: 4px;", "Đường dẫn SQLite Database (config.toml):" }
                        div { style: "display: flex; gap: 6px;",
                            input {
                                r#type: "text",
                                value: "{cfg_snap.database_path}",
                                oninput: move |e| cfg.write().database_path = e.value(),
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    if let Some(p) = rfd::FileDialog::new().add_filter("SQLite DB", &["db", "sqlite"]).pick_file() {
                                        switch_db_pick(p.display().to_string());
                                    }
                                },
                                "📂 Chọn"
                            }
                            button {
                                class: "btn btn-blue",
                                onclick: move |_| {
                                    let p = cfg.read().database_path.clone();
                                    switch_db_btn(p);
                                },
                                "Kết nối"
                            }
                        }
                    }
                }
            }
        };
    }

    let user_opt = current_user.read().clone();
    let is_admin = user_opt.as_ref().map(|u| u.has_role("admin")).unwrap_or(false);
    let can_create = user_opt
        .as_ref()
        .map(|u| u.has_role("admin") || u.has_role("reception"))
        .unwrap_or(false);

    let save_section = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        let mut l_rec = load_record.clone();
        let mut r_rec = refresh_records.clone();
        move |section: String| {
            let Some(u) = current_user.read().clone() else {
                return;
            };
            let (req_role, allowed_fields) = section_role_and_fields(&section);
            if !u.has_role("admin") && !u.has_role(req_role) {
                toast.set(Some((format!("Bạn không có quyền sửa mục '{}'", section), true)));
                return;
            }
            if *selected_record_status.read() == 1 {
                toast.set(Some(("Hồ sơ đã ký khóa, không thể sửa!".to_string(), true)));
                return;
            }
            let form_snap = form.read().clone();
            let ma_tn = map_get(&form_snap, "MA_TIEP_NHAN").trim().to_string();
            if ma_tn.is_empty() {
                toast.set(Some(("Vui lòng nhập Mã tiếp nhận!".to_string(), true)));
                return;
            }
            let rec_id = *selected_record_id.read();
            let cfg_snap = cfg.read().clone();

            let res: Result<i64, String> = run_sync(&rt, async {
                let db = state.db.read().await;
                if let Some(id) = rec_id {
                    let row: Option<KskRecordRow> =
                        sqlx::query_as("SELECT * FROM ksk_records WHERE id = ?")
                            .bind(id)
                            .fetch_optional(&*db)
                            .await
                            .map_err(|e| e.to_string())?;
                    let Some(existing) = row else {
                        return Err("Không tìm thấy hồ sơ trong CSDL".to_string());
                    };
                    if existing.status == 1 {
                        return Err("Hồ sơ đã bị khóa!".to_string());
                    }
                    let mut merged: Map<String, Value> =
                        serde_json::from_str(&existing.payload_json).unwrap_or_default();
                    for k in allowed_fields {
                        if let Some(v) = form_snap.get(*k) {
                            merged.insert((*k).to_string(), v.clone());
                        }
                    }
                    if section == "cls" && map_get(&merged, "BS_CLS").is_empty() {
                        merged.insert("BS_CLS".to_string(), json!(u.full_name));
                    }
                    if section == "ketluan" && map_get(&merged, "BS_KET_LUAN").is_empty() {
                        merged.insert("BS_KET_LUAN".to_string(), json!(u.full_name));
                    }
                    let ho_ten = map_get(&merged, "HO_TEN").to_uppercase();
                    let ma_y_te = map_get(&merged, "MA_Y_TE");
                    let gioi_tinh = map_get(&merged, "GIOI_TINH");
                    let ngay_sinh = map_get(&merged, "NGAY_SINH");
                    let so_cccd = map_get(&merged, "SO_CCCD");
                    let ngay_vao = map_get(&merged, "NGAY_VAO");
                    let payload_str = serde_json::to_string(&merged).map_err(|e| e.to_string())?;

                    sqlx::query(
                        r#"
                        UPDATE ksk_records SET
                            ma_tiep_nhan = ?, ma_y_te = ?, ho_ten = ?, gioi_tinh = ?,
                            ngay_sinh = ?, so_cccd = ?, ngay_vao = ?,
                            payload_json = ?, updated_by = ?, updated_at = datetime('now', 'localtime')
                        WHERE id = ?
                        "#,
                    )
                    .bind(&ma_tn)
                    .bind(&ma_y_te)
                    .bind(&ho_ten)
                    .bind(&gioi_tinh)
                    .bind(&ngay_sinh)
                    .bind(&so_cccd)
                    .bind(&ngay_vao)
                    .bind(&payload_str)
                    .bind(&u.username)
                    .bind(id)
                    .execute(&*db)
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok(id)
                } else {
                    if section != "reception" {
                        return Err("Vui lòng lưu Thông tin tiếp nhận trước!".to_string());
                    }
                    let mut merged = Map::new();
                    for k in allowed_fields {
                        if let Some(v) = form_snap.get(*k) {
                            merged.insert((*k).to_string(), v.clone());
                        }
                    }
                    merged.insert("MA_LK".to_string(), json!(ma_tn));
                    merged.insert("MA_CSKCB".to_string(), json!(cfg_snap.ma_cskcb));
                    merged.insert("MA_GTIN_CSKCB".to_string(), json!(cfg_snap.ma_gtin_cskcb));
                    let ho_ten = map_get(&merged, "HO_TEN").to_uppercase();
                    let ma_y_te = map_get(&merged, "MA_Y_TE");
                    let gioi_tinh = map_get(&merged, "GIOI_TINH");
                    let ngay_sinh = map_get(&merged, "NGAY_SINH");
                    let so_cccd = map_get(&merged, "SO_CCCD");
                    let ngay_vao = map_get(&merged, "NGAY_VAO");
                    let payload_str = serde_json::to_string(&merged).map_err(|e| e.to_string())?;

                    let res = sqlx::query(
                        r#"
                        INSERT INTO ksk_records (
                            ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao,
                            status, payload_json, updated_by
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
                        "#,
                    )
                    .bind(&ma_tn)
                    .bind(&ma_y_te)
                    .bind(&ho_ten)
                    .bind(&gioi_tinh)
                    .bind(&ngay_sinh)
                    .bind(&so_cccd)
                    .bind(&ngay_vao)
                    .bind(&payload_str)
                    .bind(&u.username)
                    .execute(&*db)
                    .await
                    .map_err(|e| e.to_string())?;
                    Ok(res.last_insert_rowid())
                }
            });

            match res {
                Ok(id) => {
                    editing_sections.write().insert(section, false);
                    l_rec(id);
                    r_rec();
                    toast.set(Some(("Đã lưu thành công!".to_string(), false)));
                }
                Err(e) => toast.set(Some((format!("Lỗi lưu dữ liệu: {}", e), true))),
            }
        }
    };

    let cancel_section = {
        let mut l_rec = load_record.clone();
        move |section: String| {
            if let Some(id) = *selected_record_id.read() {
                l_rec(id);
            }
            editing_sections.write().insert(section, false);
        }
    };

    let export_docx = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move |(id, is_rec): (i64, bool)| {
            let cfg_snap = cfg.read().clone();
            let res = run_sync(&rt, async {
                let db = state.db.read().await;
                generate_docx_bytes(&*db, &cfg_snap, id, is_rec).await
            });
            match res {
                Ok((filename, bytes)) => {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_file_name(&filename)
                        .add_filter("Word Document", &["docx"])
                        .save_file()
                    {
                        match fs::write(&path, &bytes) {
                            Ok(_) => {
                                toast.set(Some((format!("Đã xuất file: {}", path.display()), false)));
                                let _ = std::process::Command::new("cmd")
                                    .args(["/C", "start", "", &path.display().to_string()])
                                    .spawn();
                            }
                            Err(e) => toast.set(Some((format!("Lỗi ghi file: {}", e), true))),
                        }
                    }
                }
                Err(e) => toast.set(Some((e, true))),
            }
        }
    };

    let fetch_his = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move |_: ()| {
            let ma_tn = map_get(&form.read(), "MA_TIEP_NHAN").trim().to_string();
            if ma_tn.is_empty() {
                toast.set(Some(("Nhập Mã tiếp nhận trước khi lấy dữ liệu HIS!".to_string(), true)));
                return;
            }
            let cfg_snap = cfg.read().clone();
            let res: Result<Value, String> = run_sync(&rt, async {
                let payload_str = cfg_snap.his_payload_template.replace("{{ma_tn}}", &ma_tn);
                let payload_val: Value = serde_json::from_str(&payload_str)
                    .unwrap_or_else(|_| json!({"ma_tiep_nhan": ma_tn}));
                let resp = state
                    .http_client
                    .post(&cfg_snap.his_api_url)
                    .header(&cfg_snap.his_auth_header, &cfg_snap.his_api_key)
                    .json(&payload_val)
                    .send()
                    .await
                    .map_err(|e| format!("Lỗi gọi HIS API: {}", e))?;
                resp.json::<Value>().await.map_err(|e| e.to_string())
            });
            match res {
                Ok(val) => {
                    if let Some(data) = val.get("data").and_then(|d| d.as_object()) {
                        let mut f = form.write();
                        for (k, v) in data {
                            f.insert(k.to_uppercase(), v.clone());
                        }
                        toast.set(Some(("Đã nạp dữ liệu từ HIS!".to_string(), false)));
                    } else {
                        toast.set(Some(("HIS không trả về dữ liệu bệnh nhân".to_string(), true)));
                    }
                }
                Err(e) => toast.set(Some((e, true))),
            }
        }
    };

    let lock_toggle = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        let mut l_rec = load_record.clone();
        let mut r_rec = refresh_records.clone();
        move |(id, lock): (i64, bool)| {
            let has_perm = current_user
                .read()
                .as_ref()
                .map(|u| u.has_role("admin") || u.has_role("bs_ketluan"))
                .unwrap_or(false);
            if !has_perm {
                toast.set(Some(("Chỉ Bác sĩ Kết luận hoặc Admin mới có quyền ký khóa/mở khóa!".to_string(), true)));
                return;
            }
            let new_status = if lock { 1 } else { 0 };
            let res = run_sync(&rt, async {
                let db = state.db.read().await;
                sqlx::query("UPDATE ksk_records SET status = ?, updated_at = datetime('now', 'localtime') WHERE id = ?")
                    .bind(new_status)
                    .bind(id)
                    .execute(&*db)
                    .await
                    .map_err(|e| e.to_string())
            });
            match res {
                Ok(_) => {
                    l_rec(id);
                    r_rec();
                    toast.set(Some((
                        if lock { "Đã ký khóa hồ sơ!" } else { "Đã mở khóa hồ sơ!" }.to_string(),
                        false,
                    )));
                }
                Err(e) => toast.set(Some((e, true))),
            }
        }
    };

    let mut refresh_csv = {
        let rt = ctx.rt.clone();
        let state = ctx.state.clone();
        move |_: ()| {
            let f_from = csv_from_date.read().clone();
            let f_to = csv_to_date.read().clone();
            let f_st = *csv_status.read();
            let f_q = csv_q.read().clone();
            let f_wp = csv_workplace.read().clone();
            let f_cl = csv_classification.read().clone();
            let rows = run_sync(&rt, async {
                let db = state.db.read().await;
                query_filtered_records(
                    &*db,
                    Some(f_from.as_str()),
                    Some(f_to.as_str()),
                    f_st,
                    Some(f_q.as_str()),
                    Some(f_wp.as_str()),
                    Some(f_cl.as_str()),
                    None,
                    None,
                    true,
                )
                .await
            });
            csv_rows.set(rows);
            csv_selected_ids.write().clear();
        }
    };

    let export_csv = move |only_selected: bool| {
        let sel = csv_selected_ids.read().clone();
        let rows: Vec<KskRecordRow> = if only_selected {
            csv_rows
                .read()
                .iter()
                .filter(|r| sel.contains(&r.id))
                .cloned()
                .collect()
        } else {
            csv_rows.read().clone()
        };
        if rows.is_empty() {
            toast.set(Some(("Không có hồ sơ nào để xuất CSV!".to_string(), true)));
            return;
        }
        match generate_csv_bytes(&cfg.read(), rows) {
            Ok(bytes) => {
                let default_name = format!(
                    "KSK_QD3176_{}.csv",
                    chrono::Local::now().format("%Y%m%d_%H%M%S")
                );
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&default_name)
                    .add_filter("CSV (UTF-8 BOM)", &["csv"])
                    .save_file()
                {
                    match fs::write(&path, &bytes) {
                        Ok(_) => toast.set(Some((format!("Đã xuất CSV: {}", path.display()), false))),
                        Err(e) => toast.set(Some((format!("Lỗi lưu CSV: {}", e), true))),
                    }
                }
            }
            Err(e) => toast.set(Some((e, true))),
        }
    };

    let cur_tab = *active_tab.read();
    let cfg_snap = cfg.read().clone();
    let toast_snap = toast.read().clone();
    let upd_snap = update_available.read().clone();
    let user_full_name = user_opt
        .as_ref()
        .map(|u| u.full_name.clone())
        .unwrap_or_default();

    let mut refresh_csv_nav = refresh_csv.clone();
    let mut refresh_users_nav = refresh_users.clone();
    let mut refresh_users_btn = refresh_users.clone();
    let mut refresh_admin_cats_btn = refresh_admin_catalogs.clone();

    rsx! {
        style { "{STYLES_CSS}" }
        div { class: "app-shell",
            // Top Navigation
            div { class: "top-nav",
                div { style: "display: flex; align-items: center; gap: 12px;",
                    span { class: "brand", "🏥 {cfg_snap.ten_cskcb} (v{CURRENT_VERSION})" }
                    div { class: "nav-tabs",
                        for (t, lbl) in [
                            (AppTab::Reception, "1. Tiếp nhận"),
                            (AppTab::History, "2. Tiền sử"),
                            (AppTab::Physical, "3. Thể lực"),
                            (AppTab::Clinical, "4. Lâm sàng"),
                            (AppTab::Paraclinical, "5. Cận lâm sàng"),
                            (AppTab::Conclusion, "6. Kết luận & In"),
                            (AppTab::ExportCsv, "7. Xuất CSV"),
                        ] {
                            {
                                let mut r_csv = refresh_csv_nav.clone();
                                rsx! {
                                    button {
                                        class: if cur_tab == t { "nav-tab active" } else { "nav-tab" },
                                        onclick: move |_| {
                                            active_tab.set(t);
                                            if t == AppTab::ExportCsv && csv_rows.read().is_empty() {
                                                r_csv(());
                                            }
                                        },
                                        "{lbl}"
                                    }
                                }
                            }
                        }
                        if is_admin {
                            button {
                                class: if cur_tab == AppTab::Admin { "nav-tab active" } else { "nav-tab" },
                                onclick: move |_| {
                                    active_tab.set(AppTab::Admin);
                                    refresh_db_info();
                                    refresh_users_nav();
                                },
                                "⚙️ Quản trị"
                            }
                        }
                    }
                }

                div { class: "nav-right",
                    if let Some(m) = upd_snap {
                        button {
                            class: "btn btn-amber",
                            onclick: move |_| {
                                if let Err(e) = updater::apply_update_and_restart(&cfg.read().update_dir_path, &m) {
                                    toast.set(Some((e, true)));
                                }
                            },
                            "🆕 Cập nhật v{m.version}"
                        }
                    }
                    button {
                        class: "btn",
                        onclick: move |_| {
                            profile_full_name.set(user_full_name.clone());
                            show_profile_modal.set(true);
                        },
                        "👤 {user_full_name}"
                    }
                    button {
                        class: "btn",
                        onclick: move |_| {
                            current_user.set(None);
                            selected_record_id.set(None);
                            form.write().clear();
                        },
                        "Đăng xuất"
                    }
                }
            }

            // Main Body
            div { class: "main-body",
                if cur_tab != AppTab::ExportCsv && cur_tab != AppTab::Admin {
                    Sidebar {
                        open: sidebar_open,
                        can_create: can_create,
                        from_date: filter_from_date,
                        to_date: filter_to_date,
                        status_filter: filter_status,
                        search_q: filter_q,
                        records: records,
                        selected_id: selected_record_id,
                        on_refresh: move |_| refresh_records(),
                        on_new_record: move |_| {
                            selected_record_id.set(None);
                            selected_record_status.set(0);
                            editing_sections.write().clear();
                            editing_sections.write().insert("reception".to_string(), true);
                            let mut m = Map::new();
                            m.insert("GIOI_TINH".to_string(), json!("1"));
                            m.insert("MA_DAN_TOC".to_string(), json!("01"));
                            m.insert("MA_NGHE_NGHIEP".to_string(), json!("00"));
                            m.insert("DOI_TUONG".to_string(), json!("13"));
                            m.insert("NGUON_CHI_TRA".to_string(), json!("9"));
                            m.insert("MATINH_CU_TRU".to_string(), json!("11221"));
                            m.insert("NGAY_VAO".to_string(), json!(chrono::Local::now().format("%d/%m/%Y").to_string()));
                            form.set(m);
                            active_tab.set(AppTab::Reception);
                        },
                        on_select_record: move |id| load_record(id),
                    }
                }

                div { class: "content-area",
                    if let Some((msg, is_err)) = toast_snap {
                        div { class: if is_err { "toast toast-err" } else { "toast toast-ok" },
                            span { "{msg}" }
                            button {
                                style: "border: none; background: transparent; cursor: pointer; font-weight: 700;",
                                onclick: move |_| toast.set(None),
                                "✕"
                            }
                        }
                    }

                    match cur_tab {
                        AppTab::Reception => rsx! {
                            TabReception {
                                user: user_opt.clone(),
                                selected_record_id: selected_record_id,
                                selected_record_status: selected_record_status,
                                form: form,
                                editing_sections: editing_sections,
                                catalogs_cache: catalogs_cache,
                                on_save_section: save_section,
                                on_cancel_section: cancel_section,
                                on_fetch_his: fetch_his,
                                on_export_docx: export_docx,
                            }
                        },
                        AppTab::History => rsx! {
                            TabHistory {
                                user: user_opt.clone(),
                                selected_record_id: selected_record_id,
                                selected_record_status: selected_record_status,
                                form: form,
                                editing_sections: editing_sections,
                                catalogs_cache: catalogs_cache,
                                on_save_section: save_section,
                                on_cancel_section: cancel_section,
                            }
                        },
                        AppTab::Physical => rsx! {
                            TabPhysical {
                                user: user_opt.clone(),
                                selected_record_id: selected_record_id,
                                selected_record_status: selected_record_status,
                                form: form,
                                editing_sections: editing_sections,
                                on_save_section: save_section,
                                on_cancel_section: cancel_section,
                            }
                        },
                        AppTab::Clinical => rsx! {
                            TabClinical {
                                user: user_opt.clone(),
                                selected_record_id: selected_record_id,
                                selected_record_status: selected_record_status,
                                form: form,
                                editing_sections: editing_sections,
                                clinical_full_mode: clinical_full_mode,
                                on_save_section: save_section,
                                on_cancel_section: cancel_section,
                            }
                        },
                        AppTab::Paraclinical => rsx! {
                            TabParaclinical {
                                user: user_opt.clone(),
                                selected_record_id: selected_record_id,
                                selected_record_status: selected_record_status,
                                form: form,
                                editing_sections: editing_sections,
                                on_save_section: save_section,
                                on_cancel_section: cancel_section,
                            }
                        },
                        AppTab::Conclusion => rsx! {
                            TabConclusion {
                                user: user_opt.clone(),
                                selected_record_id: selected_record_id,
                                selected_record_status: selected_record_status,
                                form: form,
                                editing_sections: editing_sections,
                                catalogs_cache: catalogs_cache,
                                on_save_section: save_section,
                                on_cancel_section: cancel_section,
                                on_lock_toggle: lock_toggle,
                                on_export_docx: export_docx,
                            }
                        },
                        AppTab::ExportCsv => rsx! {
                            TabExportCsv {
                                csv_from_date: csv_from_date,
                                csv_to_date: csv_to_date,
                                csv_status: csv_status,
                                csv_q: csv_q,
                                csv_workplace: csv_workplace,
                                csv_classification: csv_classification,
                                csv_rows: csv_rows,
                                csv_selected_ids: csv_selected_ids,
                                on_filter: refresh_csv,
                                on_export: export_csv,
                            }
                        },
                        AppTab::Admin => rsx! {
                            TabAdmin {
                                user: user_opt.clone(),
                                cfg: cfg,
                                db_info: db_info,
                                publish_changelog: publish_changelog,
                                users_list: users_list,
                                new_user_username: new_user_username,
                                new_user_password: new_user_password,
                                new_user_fullname: new_user_fullname,
                                new_user_roles: new_user_roles,
                                catalog_type_filter: catalog_type_filter,
                                catalog_search: catalog_search,
                                catalog_rows: catalog_rows,
                                on_switch_db: move |p| switch_database(p),
                                on_save_cfg: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    move |_| {
                                        let c = cfg.read().clone();
                                        if let Err(e) = c.save() {
                                            toast.set(Some((format!("Lỗi lưu config.toml: {}", e), true)));
                                            return;
                                        }
                                        run_sync(&rt, async { *state.config.write().await = c; });
                                        toast.set(Some(("Đã lưu cấu hình vào config.toml!".to_string(), false)));
                                    }
                                },
                                on_publish_update: move |_| {
                                    let dir = cfg.read().update_dir_path.clone();
                                    let log = publish_changelog.read().clone();
                                    match updater::publish_update_to_share(&dir, &log) {
                                        Ok(m) => toast.set(Some((format!("Đã phát hành v{} lên {}", m.version, dir), false))),
                                        Err(e) => toast.set(Some((e, true))),
                                    }
                                },
                                on_create_user: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    let mut r_usr = refresh_users.clone();
                                    move |_| {
                                        let u = new_user_username.read().trim().to_string();
                                        let p = new_user_password.read().trim().to_string();
                                        let f = new_user_fullname.read().trim().to_string();
                                        let roles = new_user_roles.read().clone();
                                        if u.is_empty() || p.is_empty() || f.is_empty() || roles.is_empty() {
                                            toast.set(Some(("Vui lòng nhập đủ thông tin và chọn ít nhất 1 vai trò!".to_string(), true)));
                                            return;
                                        }
                                        let role_str = roles.join(",");
                                        let p_hash = hash_password(&p);
                                        let res = run_sync(&rt, async {
                                            let db = state.db.read().await;
                                            sqlx::query("INSERT INTO users (username, password_hash, full_name, role, is_locked) VALUES (?, ?, ?, ?, 0)")
                                                .bind(&u).bind(&p_hash).bind(&f).bind(&role_str)
                                                .execute(&*db).await.map_err(|e| e.to_string())
                                        });
                                        match res {
                                            Ok(_) => {
                                                new_user_username.set(String::new());
                                                new_user_password.set(String::new());
                                                new_user_fullname.set(String::new());
                                                r_usr();
                                                toast.set(Some(("Đã tạo tài khoản mới!".to_string(), false)));
                                            }
                                            Err(e) => toast.set(Some((format!("Lỗi tạo tài khoản: {}", e), true))),
                                        }
                                    }
                                },
                                on_toggle_lock_user: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    let mut r_usr = refresh_users.clone();
                                    move |uid| {
                                        let _ = run_sync(&rt, async {
                                            let db = state.db.read().await;
                                            sqlx::query("UPDATE users SET is_locked = CASE WHEN is_locked = 1 THEN 0 ELSE 1 END WHERE id = ?")
                                                .bind(uid).execute(&*db).await
                                        });
                                        r_usr();
                                    }
                                },
                                on_refresh_users: move |_| refresh_users_btn(),
                                on_refresh_catalogs: move |_| refresh_admin_cats_btn(),
                                on_set_default_cat: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    let mut r_cat = refresh_admin_catalogs.clone();
                                    move |cid| {
                                        let ct = catalog_type_filter.read().clone();
                                        let _ = run_sync(&rt, async {
                                            let db = state.db.read().await;
                                            let _ = sqlx::query("UPDATE catalogs SET is_default = 0 WHERE cat_type = ?").bind(&ct).execute(&*db).await;
                                            let _ = sqlx::query("UPDATE catalogs SET is_default = 1 WHERE id = ?").bind(cid).execute(&*db).await;
                                        });
                                        r_cat();
                                        catalogs_cache.set(load_catalogs_sync(&rt, &state));
                                    }
                                },
                                on_toggle_active_cat: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    let mut r_cat = refresh_admin_catalogs.clone();
                                    move |cid| {
                                        let _ = run_sync(&rt, async {
                                            let db = state.db.read().await;
                                            sqlx::query("UPDATE catalogs SET is_active = CASE WHEN is_active = 1 THEN 0 ELSE 1 END WHERE id = ?")
                                                .bind(cid).execute(&*db).await
                                        });
                                        r_cat();
                                        catalogs_cache.set(load_catalogs_sync(&rt, &state));
                                    }
                                },
                                on_reload_asset_cats: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    let mut r_cat = refresh_admin_catalogs.clone();
                                    move |_| {
                                        let res = run_sync(&rt, async {
                                            let db = state.db.read().await;
                                            seed_catalogs_from_asset(&*db).await.map_err(|e| e.to_string())
                                        });
                                        match res {
                                            Ok(n) => {
                                                r_cat();
                                                catalogs_cache.set(load_catalogs_sync(&rt, &state));
                                                toast.set(Some((format!("Đã nạp lại {} mục danh mục chuẩn!", n), false)));
                                            }
                                            Err(e) => toast.set(Some((e, true))),
                                        }
                                    }
                                },
                            }
                        },
                    }
                }
            }

            // Profile Modal
            if *show_profile_modal.read() {
                div { class: "modal-backdrop",
                    div { class: "modal-box",
                        div { style: "display: flex; justify-content: space-between; align-items: center;",
                            span { style: "font-weight: 700; font-size: 15px; color: #0f766e;", "👤 Thông tin tài khoản" }
                            button { class: "btn", onclick: move |_| show_profile_modal.set(false), "✕" }
                        }
                        div { class: "field",
                            label { "Họ tên / Chức danh hiển thị trên phiếu" }
                            if !is_admin {
                                span { style: "font-size: 11.5px; color: #b45309;", "🔒 Chỉ Quản trị viên được phép đổi Họ tên" }
                            }
                            input {
                                r#type: "text",
                                value: "{profile_full_name}",
                                disabled: !is_admin,
                                oninput: move |e| profile_full_name.set(e.value()),
                            }
                        }
                        if is_admin {
                            button {
                                class: "btn btn-primary",
                                onclick: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    move |_| {
                                        if let Some(ref mut u) = *current_user.write() {
                                            let uid = u.id;
                                            let new_name = profile_full_name.read().trim().to_string();
                                            let _ = run_sync(&rt, async {
                                                let db = state.db.read().await;
                                                sqlx::query("UPDATE users SET full_name = ? WHERE id = ?")
                                                    .bind(&new_name).bind(uid).execute(&*db).await
                                            });
                                            u.full_name = new_name;
                                            toast.set(Some(("Đã cập nhật họ tên!".to_string(), false)));
                                        }
                                    }
                                },
                                "Lưu họ tên"
                            }
                        }
                        div { style: "border-top: 1px solid #e2e8f0; padding-top: 10px; display: flex; flex-direction: column; gap: 8px;",
                            label { style: "font-weight: 700; font-size: 12.5px;", "Đổi mật khẩu" }
                            input { r#type: "password", placeholder: "Mật khẩu cũ", value: "{profile_old_pw}", oninput: move |e| profile_old_pw.set(e.value()) }
                            input { r#type: "password", placeholder: "Mật khẩu mới", value: "{profile_new_pw}", oninput: move |e| profile_new_pw.set(e.value()) }
                            button {
                                class: "btn btn-blue",
                                onclick: {
                                    let rt = ctx.rt.clone();
                                    let state = ctx.state.clone();
                                    move |_| {
                                        let Some(u) = current_user.read().clone() else { return; };
                                        let old_h = hash_password(profile_old_pw.read().trim());
                                        let new_h = hash_password(profile_new_pw.read().trim());
                                        let res = run_sync(&rt, async {
                                            let db = state.db.read().await;
                                            let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM users WHERE id = ?")
                                                .bind(u.id).fetch_optional(&*db).await.map_err(|e| e.to_string())?;
                                            if row.map(|r| r.0) != Some(old_h) {
                                                return Err("Mật khẩu cũ không đúng!".to_string());
                                            }
                                            sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
                                                .bind(&new_h).bind(u.id).execute(&*db).await.map_err(|e| e.to_string())?;
                                            Ok(())
                                        });
                                        match res {
                                            Ok(_) => {
                                                profile_old_pw.set(String::new());
                                                profile_new_pw.set(String::new());
                                                toast.set(Some(("Đã đổi mật khẩu thành công!".to_string(), false)));
                                            }
                                            Err(e) => toast.set(Some((e, true))),
                                        }
                                    }
                                },
                                "Đổi mật khẩu"
                            }
                        }
                    }
                }
            }
        }
    }
}
