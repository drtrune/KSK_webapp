pub mod sidebar;
pub mod tab_catalogs;
pub mod tab_export_admin;
pub mod tabs_exam;
pub mod widgets;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gpui::{
    AnyElement, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _, px, rgb,
};
use gpui_component::{
    Sizable,
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState, TextareaState},
};
use serde_json::{Map, Value};
use tokio::sync::broadcast;

use crate::{
    config::AppConfig,
    db,
    handlers::{
        AppState, generate_csv_bytes, generate_docx_bytes, query_filtered_records,
    },
    models::{CatalogItem, KskRecordRow, UpdateManifest, User, merge_payload_by_roles},
    updater,
};

use self::widgets::{
    bg_app, bg_card, bg_subtle, border_col, DANGER_BG, DANGER_COL, PRIMARY_COL,
    SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, card_box,
    field_label, role_label_vi,
};

pub fn payload_field(row: &KskRecordRow, key: &str) -> String {
    serde_json::from_str::<Map<String, Value>>(&row.payload_json)
        .ok()
        .and_then(|m| m.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()))
        .unwrap_or_default()
}

#[derive(Clone)]
pub struct AppLaunchCtx {
    pub rt: tokio::runtime::Handle,
    pub state: Arc<AppState>,
    pub his_tx: broadcast::Sender<String>,
    pub initial_cfg: AppConfig,
}

pub struct KskGpuiApp {
    pub rt: tokio::runtime::Handle,
    pub state: Arc<AppState>,
    pub cfg: AppConfig,
    pub app_logo: Arc<gpui::Image>,

    // Auth & Profile
    pub current_user: Option<User>,
    pub login_username: Entity<InputState>,
    pub login_password: Entity<InputState>,
    pub login_db_path: Entity<InputState>,
    pub login_error: Option<String>,
    pub show_profile_modal: bool,
    pub prof_full_name: Entity<InputState>,
    pub prof_old_pw: Entity<InputState>,
    pub prof_new_pw: Entity<InputState>,
    pub prof_confirm_pw: Entity<InputState>,
    pub dark_mode: bool,
    pub clinical_catalog_mode: bool,
    pub adm_tree_expanded: HashSet<String>,
    pub adm_tree_expand_all: bool,

    // Navigation & Modals
    pub active_nav: String,       // "exam" | "export" | "admin"
    pub active_exam_tab: String,  // "reception" | "history" | "physical" | "clinical" | "paraclinical" | "conclusion"
    pub open_dropdown: Option<String>,
    pub dropdown_search_input: Entity<InputState>,
    pub toast: Option<(String, bool)>,
    pub available_update: Option<UpdateManifest>,

    // Sidebar
    pub sidebar_open: bool,
    pub sidebar_search: Entity<InputState>,
    pub sidebar_from_date: Entity<InputState>,
    pub sidebar_to_date: Entity<InputState>,
    pub sidebar_status_filter: Option<i64>,
    pub records: Vec<KskRecordRow>,

    // Current Exam Record
    pub selected_record_id: Option<i64>,
    pub is_creating_new: bool,
    pub current_record_status: i64,
    pub editing_section: Option<String>,
    pub form_values: HashMap<String, String>,
    pub form_inputs: HashMap<String, Entity<InputState>>,
    pub form_textareas: HashMap<String, Entity<TextareaState>>,
    pub form_subs: HashMap<String, Subscription>,

    // Clinical Disease Tree & Subtab
    pub active_clinical_subtab: String,
    pub active_noi_subtab: String,
    pub clinical_tree_mode: bool,
    pub expanded_diseases: HashSet<String>,
    pub expand_all_diseases: bool,
    pub selected_classifications: HashMap<String, String>,
    pub disease_search_input: Entity<InputState>,
    pub selected_specialty_filter: String,
    pub date_picker_view_ym: (i32, u32),

    // Export CSV Tab
    pub exp_from_date: Entity<InputState>,
    pub exp_to_date: Entity<InputState>,
    pub exp_q: Entity<InputState>,
    pub exp_workplace: Entity<InputState>,
    pub exp_status_filter: Option<i64>,
    pub export_rows: Vec<KskRecordRow>,
    pub export_selected_ids: HashSet<i64>,

    // Admin Tab
    pub admin_sub_tab: String,
    pub adm_cskcb: Entity<InputState>,
    pub adm_ma_cskcb: Entity<InputState>,
    pub adm_db_path: Entity<InputState>,
    pub adm_update_dir: Entity<InputState>,
    pub adm_auto_check_update: bool,
    pub adm_changelog: Entity<InputState>,
    pub users_list: Vec<User>,
    pub adm_new_user: Entity<InputState>,
    pub adm_new_pw: Entity<InputState>,
    pub adm_new_fullname: Entity<InputState>,
    pub adm_new_roles: Vec<String>,
    pub catalogs: Vec<CatalogItem>,
    pub adm_cat_type: String,
    pub adm_cat_code: Entity<InputState>,
    pub adm_cat_name: Entity<InputState>,
    pub adm_cat_parent: Entity<InputState>,
    pub adm_cat_parent_filter: Entity<InputState>,
    pub adm_cat_search: Entity<InputState>,
    pub adm_cat_edit_id: Option<i64>,

    pub _ui_subs: Vec<Subscription>,
}

impl KskGpuiApp {
    pub fn new(launch_ctx: AppLaunchCtx, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let cfg = launch_ctx.initial_cfg.clone();

        let login_username = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Tên đăng nhập (vd: admin)")
                .default_value("admin")
        });
        let login_password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu")
                .masked(true)
        });
        let login_db_path = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Đường dẫn file SQLite (.db)")
                .default_value(cfg.database_path.clone())
        });
        let sub_login_user = cx.subscribe_in(&login_username, window, |this, _, ev, window, cx| {
            if matches!(ev, InputEvent::PressEnter { .. }) {
                this.do_login(window, cx);
            }
        });
        let sub_login_pass = cx.subscribe_in(&login_password, window, |this, _, ev, window, cx| {
            if matches!(ev, InputEvent::PressEnter { .. }) {
                this.do_login(window, cx);
            }
        });

        let prof_full_name = cx.new(|cx| InputState::new(window, cx).placeholder("Họ tên"));
        let prof_old_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu cũ")
                .masked(true)
        });
        let prof_new_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu mới")
                .masked(true)
        });
        let prof_confirm_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Nhập lại mật khẩu mới")
                .masked(true)
        });

        let sidebar_search = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Quét mã vạch / Tên / CCCD...")
        });
        let sidebar_from_date = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("YYYY-MM-DD")
                .default_value(today.clone())
        });
        let sidebar_to_date = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("YYYY-MM-DD")
                .default_value(today.clone())
        });

        let disease_search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Tìm tên bệnh (vd: tăng huyết áp, cận thị...)")
        });
        let dropdown_search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Tìm theo mã hoặc tên...")
        });
        let sub_dd_search = cx.subscribe_in(&dropdown_search_input, window, |_, _, ev, _window, cx| {
            if matches!(ev, InputEvent::Change) {
                cx.notify();
            }
        });

        let exp_from_date = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("YYYY-MM-DD")
                .default_value(today.clone())
        });
        let exp_to_date = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("YYYY-MM-DD")
                .default_value(today)
        });
        let exp_q = cx.new(|cx| InputState::new(window, cx).placeholder("Tên / Mã BN / CCCD"));
        let exp_workplace = cx.new(|cx| InputState::new(window, cx).placeholder("Đơn vị / Công ty"));

        let adm_cskcb = cx.new(|cx| {
            InputState::new(window, cx).default_value(cfg.ten_cskcb.clone())
        });
        let adm_ma_cskcb = cx.new(|cx| {
            InputState::new(window, cx).default_value(cfg.ma_cskcb.clone())
        });
        let adm_db_path = cx.new(|cx| {
            InputState::new(window, cx).default_value(cfg.database_path.clone())
        });
        let adm_update_dir = cx.new(|cx| {
            InputState::new(window, cx).default_value(cfg.update_dir_path.clone())
        });
        let adm_changelog = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Nội dung cập nhật phiên bản mới...")
        });

        let adm_new_user = cx.new(|cx| InputState::new(window, cx).placeholder("username"));
        let adm_new_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("mật khẩu")
                .masked(true)
        });
        let adm_new_fullname = cx.new(|cx| InputState::new(window, cx).placeholder("BS. Nguyễn Văn A"));

        let adm_cat_code = cx.new(|cx| InputState::new(window, cx).placeholder("Mã (vd: 11221, I10)"));
        let adm_cat_name = cx.new(|cx| InputState::new(window, cx).placeholder("Tên hiển thị"));
        let adm_cat_parent = cx.new(|cx| InputState::new(window, cx).placeholder("Mã cha (vd: mã tỉnh, mã CK)"));
        let adm_cat_parent_filter = cx.new(|cx| InputState::new(window, cx).placeholder("Lọc mã cha..."));
        let adm_cat_search = cx.new(|cx| InputState::new(window, cx).placeholder("Tìm mã hoặc tên..."));

        let sub_cat_search = cx.subscribe_in(&adm_cat_search, window, |this, _, ev, _window, cx| {
            if matches!(ev, InputEvent::Change) {
                this.refresh_catalogs(cx);
            }
        });
        let sub_cat_parent_filter = cx.subscribe_in(&adm_cat_parent_filter, window, |this, _, ev, _window, cx| {
            if matches!(ev, InputEvent::Change) {
                this.refresh_catalogs(cx);
            }
        });

        let sub_search = cx.subscribe_in(&sidebar_search, window, |this, _, ev, window, cx| {
            if matches!(ev, InputEvent::Change) {
                this.refresh_records(window, cx);
                // Auto-match exact barcode
                let q = this.sidebar_search.read(cx).value().trim().to_string();
                if !q.is_empty() {
                    if let Some(m) = this.records.iter().find(|r| {
                        r.ma_y_te == q || r.so_cccd == q || r.ma_tiep_nhan == q
                    }) {
                        let id = m.id;
                        if this.selected_record_id != Some(id) {
                            this.load_record_into_form(id, window, cx);
                        }
                    }
                }
            }
        });
        let sub_from = cx.subscribe_in(&sidebar_from_date, window, |this, _, ev, window, cx| {
            if matches!(ev, InputEvent::Change) {
                this.refresh_records(window, cx);
            }
        });
        let sub_to = cx.subscribe_in(&sidebar_to_date, window, |this, _, ev, window, cx| {
            if matches!(ev, InputEvent::Change) {
                this.refresh_records(window, cx);
            }
        });
        let sub_disease = cx.subscribe_in(&disease_search_input, window, |_, _, ev, _, cx| {
            if matches!(ev, InputEvent::Change) {
                cx.notify();
            }
        });

        let available_update = if cfg.auto_check_update && !cfg.update_dir_path.trim().is_empty() {
            updater::check_for_update(&cfg.update_dir_path).ok().flatten()
        } else {
            None
        };

        // Background listener for HIS IPC Push (127.0.0.1:18080 -> his_tx)
        let mut his_rx = launch_ctx.his_tx.subscribe();
        cx.spawn(async move |this, cx| {
            loop {
                let (res, rx_back) = cx
                    .background_executor()
                    .spawn(async move {
                        let r = his_rx.blocking_recv();
                        (r, his_rx)
                    })
                    .await;
                his_rx = rx_back;
                match res {
                    Ok(ma_bn) => {
                        let _ = this.update(cx, |app, cx| {
                            app.handle_his_push_event(&ma_bn, cx);
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
        .detach();

        Self {
            rt: launch_ctx.rt,
            state: launch_ctx.state,
            adm_auto_check_update: cfg.auto_check_update,
            cfg,
            app_logo: Arc::new(gpui::Image::from_bytes(
                gpui::ImageFormat::Png,
                include_bytes!("../../asset/logo.png").to_vec(),
            )),
            current_user: None,
            login_username,
            login_password,
            login_db_path,
            login_error: None,
            show_profile_modal: false,
            prof_full_name,
            prof_old_pw,
            prof_new_pw,
            prof_confirm_pw,
            dark_mode: false,
            clinical_catalog_mode: true,
            adm_tree_expanded: HashSet::new(),
            adm_tree_expand_all: false,
            active_nav: "exam".into(),
            active_exam_tab: "reception".into(),
            open_dropdown: None,
            dropdown_search_input,
            toast: None,
            available_update,
            sidebar_open: true,
            sidebar_search,
            sidebar_from_date,
            sidebar_to_date,
            sidebar_status_filter: None,
            records: Vec::new(),
            selected_record_id: None,
            is_creating_new: false,
            current_record_status: 0,
            editing_section: None,
            form_values: HashMap::new(),
            form_inputs: HashMap::new(),
            form_textareas: HashMap::new(),
            form_subs: HashMap::new(),
            active_clinical_subtab: "noi".into(),
            active_noi_subtab: "CK_NOI_TIM_MACH".into(),
            clinical_tree_mode: false,
            expanded_diseases: HashSet::new(),
            expand_all_diseases: false,
            selected_classifications: HashMap::new(),
            disease_search_input,
            selected_specialty_filter: String::new(),
            date_picker_view_ym: {
                use chrono::Datelike;
                let now = chrono::Local::now().date_naive();
                (now.year(), now.month())
            },
            exp_from_date,
            exp_to_date,
            exp_q,
            exp_workplace,
            exp_status_filter: None,
            export_rows: Vec::new(),
            export_selected_ids: HashSet::new(),
            admin_sub_tab: "config".into(),
            adm_cskcb,
            adm_ma_cskcb,
            adm_db_path,
            adm_update_dir,
            adm_changelog,
            users_list: Vec::new(),
            adm_new_user,
            adm_new_pw,
            adm_new_fullname,
            adm_new_roles: vec!["reception".into()],
            catalogs: Vec::new(),
            adm_cat_type: "TINH_TP".into(),
            adm_cat_code,
            adm_cat_name,
            adm_cat_parent,
            adm_cat_parent_filter,
            adm_cat_search,
            adm_cat_edit_id: None,
            _ui_subs: vec![
                sub_search,
                sub_from,
                sub_to,
                sub_disease,
                sub_cat_search,
                sub_cat_parent_filter,
                sub_dd_search,
                sub_login_user,
                sub_login_pass,
            ],
        }
    }

    pub fn run_sync<F: std::future::Future + Send>(&self, fut: F) -> F::Output
    where
        F::Output: Send,
    {
        if tokio::runtime::Handle::try_current().is_ok() {
            let rt = self.rt.clone();
            std::thread::scope(|s| s.spawn(|| rt.block_on(fut)).join().unwrap())
        } else {
            self.rt.block_on(fut)
        }
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        let Some(u) = &self.current_user else {
            return false;
        };
        roles.iter().any(|r| u.has_role(r))
    }

    pub fn get_form_val(&self, key: &str) -> String {
        self.form_values.get(key).cloned().unwrap_or_default()
    }

    pub fn get_or_create_form_input(
        &mut self,
        key: &str,
        placeholder: &str,
        masked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        if let Some(existing) = self.form_inputs.get(key) {
            return existing.clone();
        }
        let raw_val = self.form_values.get(key).cloned().unwrap_or_default();
        let initial = if raw_val.is_empty() && placeholder.starts_with("Bình thường") {
            "Bình thường".to_string()
        } else {
            raw_val
        };
        let ph = placeholder.to_string();
        let entity = cx.new(|cx| {
            let s = InputState::new(window, cx)
                .placeholder(ph)
                .default_value(initial);
            if masked { s.masked(true) } else { s }
        });

        let key_owned = key.to_string();
        let sub = cx.subscribe_in(&entity, window, move |this, ent, ev, window, cx| {
            if matches!(ev, InputEvent::Change) {
                let val = ent.read(cx).value().to_string();
                this.form_values.insert(key_owned.clone(), val);
                if key_owned == "CHIEU_CAO" || key_owned == "CAN_NANG" {
                    this.recompute_bmi(window, cx);
                }
            }
        });
        self.form_subs.insert(key.to_string(), sub);
        self.form_inputs.insert(key.to_string(), entity.clone());
        entity
    }

    pub fn get_or_create_form_textarea(
        &mut self,
        key: &str,
        placeholder: &str,
        rows: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<TextareaState> {
        if let Some(existing) = self.form_textareas.get(key) {
            return existing.clone();
        }
        let raw_val = self.form_values.get(key).cloned().unwrap_or_default();
        let initial = if raw_val.is_empty() && placeholder.starts_with("Bình thường") {
            "Bình thường".to_string()
        } else {
            raw_val
        };
        let ph = placeholder.to_string();
        let entity = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder(ph)
                .default_value(initial)
                .rows(rows)
                .auto_grow(rows, 8)
        });

        let key_owned = key.to_string();
        let sub = cx.subscribe_in(&entity, window, move |this, ent, ev, _window, cx| {
            if matches!(ev, InputEvent::Change) {
                let val = ent.read(cx).value().to_string();
                this.form_values.insert(key_owned.clone(), val);
            }
        });
        self.form_subs
            .insert(format!("ta-{}", key), sub);
        self.form_textareas.insert(key.to_string(), entity.clone());
        entity
    }

    pub fn section_doctor_field(section: &str) -> Option<&'static str> {
        match section {
            "physical" => Some("BS_THE_LUC"),
            "clin_noi" => Some("BS_NOI"),
            "clin_ngoai" => Some("BS_NGOAI"),
            "clin_dalieu" => Some("BS_DA_LIEU"),
            "clin_san" => Some("BS_SAN"),
            "clin_mat" => Some("BS_MAT"),
            "clin_tmh" => Some("BS_TMH"),
            "clin_rhm" => Some("BS_RHM"),
            "paraclinical" => Some("BS_CLS"),
            "conclusion" => Some("BS_KET_LUAN"),
            _ => None,
        }
    }

    pub fn enter_edit_section(
        &mut self,
        section: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing_section = Some(section.to_string());
        if let (Some(doc_field), Some(user)) =
            (Self::section_doctor_field(section), self.current_user.as_ref())
        {
            let cur = self.get_form_val(doc_field);
            if cur.trim().is_empty() && !user.full_name.trim().is_empty() {
                let name = user.full_name.clone();
                self.form_values
                    .insert(doc_field.to_string(), name.clone());
                if let Some(inp) = self.form_inputs.get(doc_field) {
                    inp.update(cx, |s, cx| s.set_value(name, window, cx));
                }
            }
        }
        cx.notify();
    }

    fn recompute_bmi(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let h_cm = self
            .get_form_val("CHIEU_CAO")
            .replace(',', ".")
            .parse::<f64>()
            .unwrap_or(0.0);
        let w_kg = self
            .get_form_val("CAN_NANG")
            .replace(',', ".")
            .parse::<f64>()
            .unwrap_or(0.0);
        if h_cm > 30.0 && w_kg > 5.0 {
            let h_m = h_cm / 100.0;
            let bmi = w_kg / (h_m * h_m);
            let bmi_str = format!("{:.1}", bmi);
            self.form_values
                .insert("CHI_SO_BMI".into(), bmi_str.clone());
            if let Some(ent) = self.form_inputs.get("CHI_SO_BMI") {
                ent.update(cx, |s, cx| s.set_value(bmi_str, window, cx));
            }
        }
    }

    pub fn show_toast(&mut self, msg: &str, is_err: bool, cx: &mut Context<Self>) {
        self.toast = Some((msg.to_string(), is_err));
        cx.notify();
    }

    pub fn handle_his_push_event(&mut self, ma_bn: &str, cx: &mut Context<Self>) {
        tracing::info!("GUI nhan su kien HIS Push cho ma_bn={}", ma_bn);
        let state = self.state.clone();
        let rows = self.run_sync(async move {
            let pool = state.db.read().await;
            query_filtered_records(&pool, None, None, None, None, None, None, None, Some(200), true)
                .await
        });
        self.records = rows;
        if let Some(rec) = self
            .records
            .iter()
            .find(|r| r.ma_y_te == ma_bn || r.ma_tiep_nhan == ma_bn)
            .cloned()
        {
            self.load_record_map(&rec);
            self.form_inputs.clear();
            self.form_textareas.clear();
            self.form_subs.clear();
            let disp = if rec.ho_ten.is_empty() {
                ma_bn.to_string()
            } else {
                rec.ho_ten.clone()
            };
            self.show_toast(&format!("Đã nhận bệnh nhân từ HIS: {}", disp), false, cx);
        } else {
            self.show_toast(&format!("HIS vừa đẩy hồ sơ {}", ma_bn), false, cx);
        }
        cx.notify();
    }

    fn load_record_map(&mut self, rec: &KskRecordRow) {
        self.selected_record_id = Some(rec.id);
        self.is_creating_new = false;
        self.current_record_status = rec.status;
        self.editing_section = None;
        self.form_values.clear();
        self.selected_classifications.clear();
        self.expanded_diseases.clear();
        if let Ok(map) = serde_json::from_str::<Map<String, Value>>(&rec.payload_json) {
            for (k, v) in map {
                let s = match v {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => {
                        if b { "1".into() } else { "0".into() }
                    }
                    _ => String::new(),
                };
                self.form_values.insert(k, s);
            }
        }
        if let Some(raw_cls) = self.form_values.get("SELECTED_CLASSIFICATIONS_JSON") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(raw_cls) {
                self.selected_classifications = map;
            }
        }
    }

    pub fn refresh_records(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let q = self.sidebar_search.read(cx).value().trim().to_string();
        let from_d = self.sidebar_from_date.read(cx).value().trim().to_string();
        let to_d = self.sidebar_to_date.read(cx).value().trim().to_string();
        let st = self.sidebar_status_filter;
        let state = self.state.clone();

        let rows = self.run_sync(async move {
            let pool = state.db.read().await;
            query_filtered_records(
                &pool,
                if from_d.is_empty() { None } else { Some(from_d.as_str()) },
                if to_d.is_empty() { None } else { Some(to_d.as_str()) },
                st,
                if q.is_empty() { None } else { Some(q.as_str()) },
                None,
                None,
                None,
                Some(200),
                false,
            )
            .await
        });
        self.records = rows;
        cx.notify();
    }

    pub fn load_record_into_form(
        &mut self,
        record_id: i64,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = self.state.clone();
        let rec_opt = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query_as::<_, KskRecordRow>(
                "SELECT id, ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, payload_json, updated_by, created_at, updated_at FROM ksk_records WHERE id = ?",
            )
            .bind(record_id)
            .fetch_optional(&*pool)
            .await
            .ok()
            .flatten()
        });

        if let Some(rec) = rec_opt {
            self.load_record_map(&rec);
            self.form_inputs.clear();
            self.form_textareas.clear();
            self.form_subs.clear();
            cx.notify();
        }
    }

    pub fn start_new_patient(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_record_id = None;
        self.is_creating_new = true;
        self.current_record_status = 0;
        self.editing_section = Some("reception".into());
        self.active_exam_tab = "reception".into();
        self.form_values.clear();
        self.selected_classifications.clear();
        self.expanded_diseases.clear();
        self.form_inputs.clear();
        self.form_textareas.clear();
        self.form_subs.clear();
        let now_str = chrono::Local::now().format("%Y%m%d%H%M").to_string();
        self.form_values.insert("NGAY_VAO".into(), now_str);
        self.form_values
            .insert("MA_CSKCB".into(), self.cfg.ma_cskcb.clone());
        self.form_values.insert("GIOI_TINH".into(), "1".into());

        let def_tinh = self.get_default_catalog_code("TINH_TP", None);
        if !def_tinh.is_empty() {
            self.form_values.insert("MATINH_CU_TRU".into(), def_tinh.clone());
            let def_xa = self.get_default_catalog_code("XA_PHUONG", Some(&def_tinh));
            if !def_xa.is_empty() {
                self.form_values.insert("MAXA_CU_TRU".into(), def_xa);
            }
        }
        for (field, cat_type, fallback) in [
            ("MA_DAN_TOC", "DAN_TOC", "01"),
            ("MA_NGHE_NGHIEP", "NGHE_NGHIEP", "00"),
            ("DOI_TUONG", "DOI_TUONG", "13"),
            ("NGUON_CHI_TRA", "NGUON_CHI_TRA", "9"),
            ("MA_LOAI_KCB", "LOAI_KCB", "1"),
            ("NHOM_MAU", "NHOM_MAU", ""),
        ] {
            let def = self.get_default_catalog_code(cat_type, None);
            let val = if !def.is_empty() { def } else { fallback.to_string() };
            if !val.is_empty() {
                self.form_values.insert(field.into(), val);
            }
        }
        cx.notify();
    }

    pub fn save_section(
        &mut self,
        section: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(user) = self.current_user.clone() else {
            return;
        };
        if let Some(doc_field) = Self::section_doctor_field(section) {
            if self.get_form_val(doc_field).trim().is_empty() && !user.full_name.trim().is_empty()
            {
                self.form_values
                    .insert(doc_field.to_string(), user.full_name.clone());
            }
        }
        let roles_str = user.role.clone();
        let mut incoming = Map::new();
        for (k, v) in &self.form_values {
            incoming.insert(k.clone(), Value::String(v.clone()));
        }
        // Chỉ khi lưu đúng chuyên khoa này mới lấy giá trị mặc định trên UI (Bình thường / Loại 1) nếu chưa sửa
        for &f in crate::models::get_section_fields(section) {
            if !incoming.contains_key(f) {
                if let Some(ta) = self.form_textareas.get(f) {
                    let v = ta.read(cx).value().trim().to_string();
                    if !v.is_empty() {
                        incoming.insert(f.to_string(), Value::String(v));
                    }
                } else if f.ends_with("_PL") {
                    incoming.insert(f.to_string(), Value::String("1".into()));
                }
            }
        }

        let state = self.state.clone();
        let cfg_ma = self.cfg.ma_cskcb.clone();
        let cfg_gtin = self.cfg.ma_gtin_cskcb.clone();
        let rec_id_opt = self.selected_record_id;
        let sec_owned = section.to_string();

        let updated_by = user.username.clone();
        let res: Result<i64, String> = self.run_sync(async move {
            let pool = state.db.read().await;
            if let Some(rid) = rec_id_opt {
                let existing: Option<(String, i64)> = sqlx::query_as(
                    "SELECT payload_json, status FROM ksk_records WHERE id = ?",
                )
                .bind(rid)
                .fetch_optional(&*pool)
                .await
                .map_err(|e| e.to_string())?;
                let (ext_json, st) = existing.ok_or("Không tìm thấy hồ sơ")?;
                if st == 2 {
                    return Err("Hồ sơ đã ký khóa, không thể chỉnh sửa".into());
                }
                let merged = merge_payload_by_roles(
                    Some(&ext_json),
                    &incoming,
                    &roles_str,
                    &cfg_ma,
                    &cfg_gtin,
                    Some(&sec_owned),
                );
                let ho_ten = merged.get("HO_TEN").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ma_y_te = merged.get("MA_Y_TE").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ma_tn = merged.get("MA_TIEP_NHAN").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let gioi_tinh = merged.get("GIOI_TINH").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ngay_sinh = merged.get("NGAY_SINH").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let so_cccd = merged.get("SO_CCCD").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ngay_vao = merged.get("NGAY_VAO").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let json_str = serde_json::to_string(&merged).map_err(|e| e.to_string())?;
                sqlx::query(
                    "UPDATE ksk_records SET ma_tiep_nhan = ?, ma_y_te = ?, ho_ten = ?, gioi_tinh = ?, ngay_sinh = ?, so_cccd = ?, ngay_vao = ?, payload_json = ?, updated_by = ?, updated_at = datetime('now','localtime') WHERE id = ?",
                )
                .bind(&ma_tn)
                .bind(&ma_y_te)
                .bind(&ho_ten)
                .bind(&gioi_tinh)
                .bind(&ngay_sinh)
                .bind(&so_cccd)
                .bind(&ngay_vao)
                .bind(&json_str)
                .bind(&updated_by)
                .bind(rid)
                .execute(&*pool)
                .await
                .map_err(|e| e.to_string())?;
                Ok(rid)
            } else {
                let merged = merge_payload_by_roles(
                    None,
                    &incoming,
                    &roles_str,
                    &cfg_ma,
                    &cfg_gtin,
                    Some(&sec_owned),
                );
                let ho_ten = merged.get("HO_TEN").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ma_y_te = merged.get("MA_Y_TE").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ma_tn = merged.get("MA_TIEP_NHAN").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let gioi_tinh = merged.get("GIOI_TINH").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ngay_sinh = merged.get("NGAY_SINH").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let so_cccd = merged.get("SO_CCCD").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ngay_vao = merged.get("NGAY_VAO").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let json_str = serde_json::to_string(&merged).map_err(|e| e.to_string())?;
                let r = sqlx::query(
                    "INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, payload_json, updated_by) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)",
                )
                .bind(&ma_tn)
                .bind(&ma_y_te)
                .bind(&ho_ten)
                .bind(&gioi_tinh)
                .bind(&ngay_sinh)
                .bind(&so_cccd)
                .bind(&ngay_vao)
                .bind(&json_str)
                .bind(&updated_by)
                .execute(&*pool)
                .await
                .map_err(|e| e.to_string())?;
                Ok(r.last_insert_rowid())
            }
        });

        match res {
            Ok(rid) => {
                self.editing_section = None;
                self.is_creating_new = false;
                self.refresh_records(window, cx);
                self.load_record_into_form(rid, window, cx);
                self.show_toast("Đã lưu thông tin thành công!", false, cx);
            }
            Err(e) => {
                self.show_toast(&format!("Lỗi lưu dữ liệu: {}", e), true, cx);
            }
        }
    }

    pub fn set_record_status(
        &mut self,
        new_status: i64,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(rid) = self.selected_record_id else {
            return;
        };
        let state = self.state.clone();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query("UPDATE ksk_records SET status = ?, updated_at = datetime('now','localtime') WHERE id = ?")
                .bind(new_status)
                .bind(rid)
                .execute(&*pool)
                .await
                .map_err(|e| e.to_string())
        });
        match res {
            Ok(_) => {
                self.current_record_status = new_status;
                self.editing_section = None;
                self.refresh_records(window, cx);
                self.show_toast(
                    if new_status == 2 {
                        "Đã ký khóa hồ sơ!"
                    } else {
                        "Đã mở khóa hồ sơ!"
                    },
                    false,
                    cx,
                );
            }
            Err(e) => self.show_toast(&format!("Lỗi cập nhật trạng thái: {}", e), true, cx),
        }
    }

    pub fn export_docx(&mut self, is_reception: bool, cx: &mut Context<Self>) {
        let Some(rid) = self.selected_record_id else {
            return;
        };
        let state = self.state.clone();
        let cfg = self.cfg.clone();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            generate_docx_bytes(&pool, &cfg, rid, is_reception).await
        });
        match res {
            Ok((filename, bytes)) => {
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&filename)
                    .add_filter("Word Document", &["docx"])
                    .save_file()
                {
                    if let Err(e) = std::fs::write(&path, bytes) {
                        self.show_toast(&format!("Lỗi ghi file: {}", e), true, cx);
                    } else {
                        self.show_toast(
                            &format!("Đã xuất file: {}", path.display()),
                            false,
                            cx,
                        );
                    }
                }
            }
            Err(e) => self.show_toast(&format!("Lỗi xuất DOCX: {}", e), true, cx),
        }
    }

    pub fn refresh_export_rows(&mut self, cx: &mut Context<Self>) {
        let from_d = self.exp_from_date.read(cx).value().trim().to_string();
        let to_d = self.exp_to_date.read(cx).value().trim().to_string();
        let q = self.exp_q.read(cx).value().trim().to_string();
        let wp = self.exp_workplace.read(cx).value().trim().to_string();
        let st = self.exp_status_filter;
        let state = self.state.clone();

        let rows = self.run_sync(async move {
            let pool = state.db.read().await;
            query_filtered_records(
                &pool,
                if from_d.is_empty() { None } else { Some(from_d.as_str()) },
                if to_d.is_empty() { None } else { Some(to_d.as_str()) },
                st,
                if q.is_empty() { None } else { Some(q.as_str()) },
                if wp.is_empty() { None } else { Some(wp.as_str()) },
                None,
                None,
                Some(2000),
                false,
            )
            .await
        });
        self.export_rows = rows;
        self.export_selected_ids.clear();
        cx.notify();
    }

    pub fn execute_csv_export(&mut self, cx: &mut Context<Self>) {
        let rows: Vec<KskRecordRow> = if self.export_selected_ids.is_empty() {
            self.export_rows.clone()
        } else {
            self.export_rows
                .iter()
                .filter(|r| self.export_selected_ids.contains(&r.id))
                .cloned()
                .collect()
        };
        if rows.is_empty() {
            self.show_toast("Không có hồ sơ nào để xuất CSV!", true, cx);
            return;
        }
        match generate_csv_bytes(&self.cfg, rows) {
            Ok(bytes) => {
                let default_name = format!(
                    "KSK_Export_3176_{}.csv",
                    chrono::Local::now().format("%Y%m%d_%H%M")
                );
                if let Some(path) = rfd::FileDialog::new()
                    .set_file_name(&default_name)
                    .add_filter("CSV (UTF-8 BOM)", &["csv"])
                    .save_file()
                {
                    if let Err(e) = std::fs::write(&path, bytes) {
                        self.show_toast(&format!("Lỗi ghi file CSV: {}", e), true, cx);
                    } else {
                        self.show_toast(
                            &format!("Đã xuất CSV thành công: {}", path.display()),
                            false,
                            cx,
                        );
                    }
                }
            }
            Err(e) => self.show_toast(&format!("Lỗi tạo CSV: {}", e), true, cx),
        }
    }

    pub fn save_admin_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut new_cfg = self.cfg.clone();
        new_cfg.ten_cskcb = self.adm_cskcb.read(cx).value().trim().to_string();
        new_cfg.ma_cskcb = self.adm_ma_cskcb.read(cx).value().trim().to_string();
        new_cfg.database_path = self.adm_db_path.read(cx).value().trim().to_string();
        new_cfg.update_dir_path = self.adm_update_dir.read(cx).value().trim().to_string();
        new_cfg.auto_check_update = self.adm_auto_check_update;

        if let Err(e) = new_cfg.save() {
            self.show_toast(&format!("Lỗi lưu config.toml: {}", e), true, cx);
            return;
        }
        let db_path = new_cfg.database_path.clone();
        let state = self.state.clone();
        let cfg_clone = new_cfg.clone();
        let res = self.run_sync(async move {
            let pool = db::init_db(&db_path).await.map_err(|e| e.to_string())?;
            *state.db.write().await = pool;
            *state.config.write().await = cfg_clone;
            Ok::<(), String>(())
        });
        match res {
            Ok(_) => {
                self.cfg = new_cfg.clone();
                self.login_db_path
                    .update(cx, |s, cx| s.set_value(new_cfg.database_path, window, cx));
                self.refresh_records(window, cx);
                self.show_toast("Đã lưu cấu hình và kết nối lại CSDL!", false, cx);
            }
            Err(e) => self.show_toast(&format!("Lỗi kết nối CSDL mới: {}", e), true, cx),
        }
    }

    pub fn publish_update(&mut self, cx: &mut Context<Self>) {
        let dir = self.adm_update_dir.read(cx).value().trim().to_string();
        let log = self.adm_changelog.read(cx).value().trim().to_string();
        if dir.is_empty() {
            self.show_toast("Vui lòng nhập thư mục cập nhật trên ổ chung!", true, cx);
            return;
        }
        match updater::publish_update_to_share(&dir, &log) {
            Ok(m) => self.show_toast(
                &format!("Đã phát hành v{} lên {}", m.version, dir),
                false,
                cx,
            ),
            Err(e) => self.show_toast(&format!("Lỗi phát hành: {}", e), true, cx),
        }
    }

    pub fn refresh_users(&mut self, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let list = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, full_name, role, is_locked, created_at FROM users ORDER BY id ASC",
            )
            .fetch_all(&*pool)
            .await
            .unwrap_or_default()
        });
        self.users_list = list;
        cx.notify();
    }

    pub fn create_admin_user(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let username = self.adm_new_user.read(cx).value().trim().to_string();
        let pw = self.adm_new_pw.read(cx).value().to_string();
        let full_name = self.adm_new_fullname.read(cx).value().trim().to_string();
        if username.is_empty() || pw.is_empty() || full_name.is_empty() {
            self.show_toast("Vui lòng nhập đủ tên đăng nhập, mật khẩu, họ tên!", true, cx);
            return;
        }
        let roles_csv = if self.adm_new_roles.is_empty() {
            "reception".to_string()
        } else {
            self.adm_new_roles.join(",")
        };
        let hash = db::hash_password(&pw);
        let state = self.state.clone();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query(
                "INSERT INTO users (username, password_hash, full_name, role, is_locked) VALUES (?, ?, ?, ?, 0)",
            )
            .bind(&username)
            .bind(&hash)
            .bind(&full_name)
            .bind(&roles_csv)
            .execute(&*pool)
            .await
            .map_err(|e| e.to_string())
        });
        match res {
            Ok(_) => {
                self.adm_new_user.update(cx, |s, cx| s.set_value("", window, cx));
                self.adm_new_pw.update(cx, |s, cx| s.set_value("", window, cx));
                self.adm_new_fullname.update(cx, |s, cx| s.set_value("", window, cx));
                self.refresh_users(cx);
                self.show_toast("Đã tạo tài khoản mới!", false, cx);
            }
            Err(e) => self.show_toast(&format!("Lỗi tạo tài khoản: {}", e), true, cx),
        }
    }

    pub fn toggle_user_active(&mut self, user_id: i64, lock: bool, cx: &mut Context<Self>) {
        let val = if lock { 1i64 } else { 0i64 };
        let state = self.state.clone();
        let _ = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query("UPDATE users SET is_locked = ? WHERE id = ?")
                .bind(val)
                .bind(user_id)
                .execute(&*pool)
                .await
        });
        self.refresh_users(cx);
    }

    pub fn query_catalog_items(
        &self,
        cat_type: &str,
        parent_code: Option<&str>,
    ) -> Vec<CatalogItem> {
        let state = self.state.clone();
        let ct = cat_type.to_string();
        let pc = parent_code.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        self.run_sync(async move {
            let pool = state.db.read().await;
            if let Some(parent) = pc {
                sqlx::query_as::<_, CatalogItem>(
                    "SELECT id, cat_type, code, name, parent_code, is_active, is_default \
                     FROM catalogs \
                     WHERE cat_type = ? AND is_active = 1 AND (parent_code = ? OR parent_code = '') \
                     ORDER BY is_default DESC, name ASC, id ASC",
                )
                .bind(&ct)
                .bind(&parent)
                .fetch_all(&*pool)
                .await
                .unwrap_or_default()
            } else {
                sqlx::query_as::<_, CatalogItem>(
                    "SELECT id, cat_type, code, name, parent_code, is_active, is_default \
                     FROM catalogs \
                     WHERE cat_type = ? AND is_active = 1 \
                     ORDER BY is_default DESC, name ASC, id ASC",
                )
                .bind(&ct)
                .fetch_all(&*pool)
                .await
                .unwrap_or_default()
            }
        })
    }

    pub fn query_catalog_options(
        &self,
        cat_type: &str,
        parent_code: Option<&str>,
    ) -> Vec<(String, String)> {
        let show_code = cat_type == "ICD10";
        self.query_catalog_items(cat_type, parent_code)
            .into_iter()
            .map(|item| {
                let label = if item.name.trim().is_empty() {
                    item.code.clone()
                } else if show_code {
                    format!("{} - {}", item.code, item.name)
                } else {
                    item.name.clone()
                };
                (item.code, label)
            })
            .collect()
    }

    pub fn get_default_catalog_code(&self, cat_type: &str, parent_code: Option<&str>) -> String {
        let state = self.state.clone();
        let ct = cat_type.to_string();
        let pc = parent_code.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        self.run_sync(async move {
            let pool = state.db.read().await;
            let row: Option<(String,)> = if let Some(parent) = pc {
                sqlx::query_as(
                    "SELECT code FROM catalogs \
                     WHERE cat_type = ? AND is_active = 1 AND is_default = 1 AND (parent_code = ? OR parent_code = '') \
                     ORDER BY id ASC LIMIT 1",
                )
                .bind(&ct)
                .bind(&parent)
                .fetch_optional(&*pool)
                .await
                .unwrap_or(None)
            } else {
                sqlx::query_as(
                    "SELECT code FROM catalogs \
                     WHERE cat_type = ? AND is_active = 1 AND is_default = 1 \
                     ORDER BY id ASC LIMIT 1",
                )
                .bind(&ct)
                .fetch_optional(&*pool)
                .await
                .unwrap_or(None)
            };
            row.map(|r| r.0).unwrap_or_default()
        })
    }

    pub fn refresh_catalogs(&mut self, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let ct = self.adm_cat_type.clone();
        let search = self.adm_cat_search.read(cx).value().trim().to_lowercase();
        let parent_filter = self.adm_cat_parent_filter.read(cx).value().trim().to_string();
        let list = self.run_sync(async move {
            let pool = state.db.read().await;
            let mut sql = String::from(
                "SELECT id, cat_type, code, name, parent_code, is_active, is_default \
                 FROM catalogs WHERE cat_type = ?",
            );
            if !parent_filter.is_empty() {
                sql.push_str(" AND parent_code = ?");
            }
            if !search.is_empty() {
                sql.push_str(" AND (LOWER(code) LIKE ? OR LOWER(name) LIKE ?)");
            }
            sql.push_str(" ORDER BY is_default DESC, is_active DESC, name ASC, id ASC LIMIT 200");

            let mut q = sqlx::query_as::<_, CatalogItem>(&sql).bind(&ct);
            if !parent_filter.is_empty() {
                q = q.bind(&parent_filter);
            }
            if !search.is_empty() {
                let pat = format!("%{}%", search);
                q = q.bind(pat.clone()).bind(pat);
            }
            q.fetch_all(&*pool).await.unwrap_or_default()
        });
        self.catalogs = list;
        cx.notify();
    }

    pub fn save_catalog_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let code = self.adm_cat_code.read(cx).value().trim().to_string();
        let name = self.adm_cat_name.read(cx).value().trim().to_string();
        let parent_code = self.adm_cat_parent.read(cx).value().trim().to_string();
        let ct = self.adm_cat_type.clone();
        if code.is_empty() || name.is_empty() {
            self.show_toast("Vui lòng nhập Mã và Tên danh mục!", true, cx);
            return;
        }
        let state = self.state.clone();
        let edit_id = self.adm_cat_edit_id;
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            if let Some(id) = edit_id {
                sqlx::query(
                    "UPDATE catalogs SET code = ?, name = ?, parent_code = ? WHERE id = ?",
                )
                .bind(&code)
                .bind(&name)
                .bind(&parent_code)
                .bind(id)
                .execute(&*pool)
                .await
            } else {
                sqlx::query(
                    "INSERT INTO catalogs (cat_type, code, name, parent_code, is_active, is_default) \
                     VALUES (?, ?, ?, ?, 1, 0) \
                     ON CONFLICT(cat_type, code, parent_code) DO UPDATE SET name = excluded.name",
                )
                .bind(&ct)
                .bind(&code)
                .bind(&name)
                .bind(&parent_code)
                .execute(&*pool)
                .await
            }
        });
        if let Err(e) = res {
            self.show_toast(&format!("Lỗi lưu danh mục: {}", e), true, cx);
            return;
        }
        self.adm_cat_edit_id = None;
        self.adm_cat_code.update(cx, |s, cx| s.set_value("", window, cx));
        self.adm_cat_name.update(cx, |s, cx| s.set_value("", window, cx));
        self.adm_cat_parent.update(cx, |s, cx| s.set_value("", window, cx));
        self.refresh_catalogs(cx);
        self.show_toast(
            if edit_id.is_some() {
                "Đã cập nhật danh mục!"
            } else {
                "Đã thêm danh mục!"
            },
            false,
            cx,
        );
    }

    pub fn start_edit_catalog_item(
        &mut self,
        item: &CatalogItem,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.adm_cat_edit_id = Some(item.id);
        let code = item.code.clone();
        let name = item.name.clone();
        let parent = item.parent_code.clone();
        self.adm_cat_code.update(cx, |s, cx| s.set_value(code, window, cx));
        self.adm_cat_name.update(cx, |s, cx| s.set_value(name, window, cx));
        self.adm_cat_parent.update(cx, |s, cx| s.set_value(parent, window, cx));
        cx.notify();
    }

    pub fn cancel_edit_catalog_item(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.adm_cat_edit_id = None;
        self.adm_cat_code.update(cx, |s, cx| s.set_value("", window, cx));
        self.adm_cat_name.update(cx, |s, cx| s.set_value("", window, cx));
        self.adm_cat_parent.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }

    pub fn toggle_catalog_active(&mut self, id: i64, active: bool, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let _ = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query("UPDATE catalogs SET is_active = ? WHERE id = ?")
                .bind(if active { 1 } else { 0 })
                .bind(id)
                .execute(&*pool)
                .await
        });
        self.refresh_catalogs(cx);
    }

    pub fn set_catalog_default(&mut self, id: i64, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let ct = self.adm_cat_type.clone();
        let _ = self.run_sync(async move {
            let pool = state.db.read().await;
            let _ = sqlx::query("UPDATE catalogs SET is_default = 0 WHERE cat_type = ?")
                .bind(&ct)
                .execute(&*pool)
                .await;
            sqlx::query("UPDATE catalogs SET is_default = 1, is_active = 1 WHERE id = ?")
                .bind(id)
                .execute(&*pool)
                .await
        });
        self.refresh_catalogs(cx);
        self.show_toast("Đã đặt mặc định!", false, cx);
    }

    pub fn delete_catalog_item(&mut self, id: i64, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let _ = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query("DELETE FROM catalogs WHERE id = ?")
                .bind(id)
                .execute(&*pool)
                .await
        });
        if self.adm_cat_edit_id == Some(id) {
            self.adm_cat_edit_id = None;
        }
        self.refresh_catalogs(cx);
    }

    pub fn import_catalogs_from_file(&mut self, cx: &mut Context<Self>) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("TSV / CSV", &["tsv", "csv", "txt"])
            .pick_file()
        else {
            return;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            self.show_toast("Không đọc được file danh mục!", true, cx);
            return;
        };
        let default_ct = self.adm_cat_type.clone();
        let state = self.state.clone();
        let count = self.run_sync(async move {
            let pool = state.db.read().await;
            let mut tx = match pool.begin().await {
                Ok(t) => t,
                Err(_) => return 0usize,
            };
            let mut imported = 0usize;
            for (idx, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                let sep = if trimmed.contains('\t') { '\t' } else { ',' };
                let cols: Vec<&str> = trimmed.split(sep).map(|s| s.trim()).collect();
                if idx == 0
                    && cols.first().map(|s| s.eq_ignore_ascii_case("cat_type") || s.eq_ignore_ascii_case("code")).unwrap_or(false)
                {
                    continue;
                }
                let (ct, code, name, parent) = if cols.len() >= 3 {
                    let c0 = cols[0];
                    if matches!(
                        c0,
                        "TINH_TP" | "XA_PHUONG" | "DAN_TOC" | "NGHE_NGHIEP" | "NHOM_MAU"
                            | "DOI_TUONG" | "NGUON_CHI_TRA" | "LOAI_KCB" | "NOI_CAP"
                            | "ICD10" | "CHUYEN_KHOA" | "BENH_TAT" | "TIEU_CHUAN_PL"
                    ) {
                        (
                            c0.to_string(),
                            cols.get(1).copied().unwrap_or("").to_string(),
                            cols.get(2).copied().unwrap_or("").to_string(),
                            cols.get(3).copied().unwrap_or("").to_string(),
                        )
                    } else {
                        (
                            default_ct.clone(),
                            cols[0].to_string(),
                            cols[1].to_string(),
                            cols.get(2).copied().unwrap_or("").to_string(),
                        )
                    }
                } else if cols.len() == 2 {
                    (default_ct.clone(), cols[0].to_string(), cols[1].to_string(), String::new())
                } else {
                    continue;
                };
                if code.is_empty() || name.is_empty() {
                    continue;
                }
                if sqlx::query(
                    "INSERT INTO catalogs (cat_type, code, name, parent_code, is_active, is_default) \
                     VALUES (?, ?, ?, ?, 1, 0) \
                     ON CONFLICT(cat_type, code, parent_code) DO UPDATE SET name = excluded.name",
                )
                .bind(&ct)
                .bind(&code)
                .bind(&name)
                .bind(&parent)
                .execute(&mut *tx)
                .await
                .is_ok()
                {
                    imported += 1;
                }
            }
            let _ = tx.commit().await;
            imported
        });
        self.refresh_catalogs(cx);
        self.show_toast(&format!("Đã import {} dòng danh mục!", count), false, cx);
    }

    pub fn reload_seed_catalogs(&mut self, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            db::seed_catalogs_from_asset(&pool).await
        });
        match res {
            Ok(n) => {
                self.refresh_catalogs(cx);
                self.show_toast(&format!("Đã nạp lại {} mục từ seed_catalogs.tsv!", n), false, cx);
            }
            Err(e) => {
                self.show_toast(&format!("Lỗi nạp seed_catalogs.tsv: {}", e), true, cx);
            }
        }
    }

    pub fn set_default_tabs_for_user(&mut self, u: &User) {
        let roles = u.roles();
        let first_specific = roles
            .iter()
            .find(|r| r.as_str() != "admin")
            .or_else(|| roles.first());
        let Some(role) = first_specific else {
            return;
        };
        match role.as_str() {
            "reception" | "admin" => {
                self.active_exam_tab = "reception".into();
            }
            "tiensu" => {
                self.active_exam_tab = "history".into();
            }
            "bs_theluc" => {
                self.active_exam_tab = "physical".into();
            }
            "bs_noi" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "noi".into();
            }
            "bs_ngoai" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "ngoai".into();
            }
            "bs_dalieu" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "dalieu".into();
            }
            "bs_san" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "san".into();
            }
            "bs_mat" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "mat".into();
            }
            "bs_tmh" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "tmh".into();
            }
            "bs_rhm" => {
                self.active_exam_tab = "clinical".into();
                self.active_clinical_subtab = "rhm".into();
            }
            "bs_cls" | "cls" | "xetnghiem" | "cdha" => {
                self.active_exam_tab = "paraclinical".into();
            }
            "bs_ketluan" => {
                self.active_exam_tab = "conclusion".into();
            }
            _ => {}
        }
    }

    pub fn do_login(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let username = self.login_username.read(cx).value().trim().to_string();
        let password = self.login_password.read(cx).value().to_string();
        let db_path = self.login_db_path.read(cx).value().trim().to_string();

        if db_path != self.cfg.database_path && !db_path.is_empty() {
            let mut new_cfg = self.cfg.clone();
            new_cfg.database_path = db_path.clone();
            let _ = new_cfg.save();
            let state = self.state.clone();
            let cfg_c = new_cfg.clone();
            if let Err(e) = self.run_sync(async move {
                let pool = db::init_db(&db_path).await.map_err(|e| e.to_string())?;
                *state.db.write().await = pool;
                *state.config.write().await = cfg_c;
                Ok::<(), String>(())
            }) {
                self.login_error = Some(format!("Lỗi kết nối DB: {}", e));
                cx.notify();
                return;
            }
            self.cfg = new_cfg;
        }

        let hash = db::hash_password(&password);
        let state = self.state.clone();
        let user_opt = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, full_name, role, is_locked, created_at FROM users WHERE username = ?",
            )
            .bind(&username)
            .fetch_optional(&*pool)
            .await
            .ok()
            .flatten()
        });

        match user_opt {
            Some(u) if u.password_hash == hash => {
                if u.is_locked != 0 {
                    self.login_error = Some("Tài khoản đã bị khóa!".into());
                    cx.notify();
                    return;
                }
                self.login_error = None;
                self.prof_full_name
                    .update(cx, |s, cx| s.set_value(u.full_name.clone(), window, cx));
                self.set_default_tabs_for_user(&u);
                self.current_user = Some(u);
                self.refresh_catalogs(cx);
                self.refresh_records(window, cx);
                cx.notify();
            }
            _ => {
                self.login_error = Some("Sai tên đăng nhập hoặc mật khẩu!".into());
                cx.notify();
            }
        }
    }

    pub fn save_profile(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(u) = self.current_user.clone() else {
            return;
        };
        let is_admin = u.has_role("admin");
        let new_name = self.prof_full_name.read(cx).value().trim().to_string();
        let old_pw = self.prof_old_pw.read(cx).value().to_string();
        let new_pw = self.prof_new_pw.read(cx).value().to_string();
        let confirm_pw = self.prof_confirm_pw.read(cx).value().to_string();

        if !new_pw.is_empty() {
            if new_pw != confirm_pw {
                self.show_toast("Mật khẩu mới và xác nhận mật khẩu không khớp!", true, cx);
                return;
            }
            if db::hash_password(&old_pw) != u.password_hash {
                self.show_toast("Mật khẩu cũ không chính xác!", true, cx);
                return;
            }
        }

        let state = self.state.clone();
        let uid = u.id;
        let final_name = if is_admin && !new_name.is_empty() {
            new_name
        } else {
            u.full_name.clone()
        };
        let new_hash = if !new_pw.is_empty() {
            Some(db::hash_password(&new_pw))
        } else {
            None
        };

        let name_c = final_name.clone();
        let hash_c = new_hash.clone();
        let _ = self.run_sync(async move {
            let pool = state.db.read().await;
            let _ = sqlx::query("UPDATE users SET full_name = ? WHERE id = ?")
                .bind(&name_c)
                .bind(uid)
                .execute(&*pool)
                .await;
            if let Some(h) = hash_c {
                let _ = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
                    .bind(&h)
                    .bind(uid)
                    .execute(&*pool)
                    .await;
            }
        });

        if let Some(ref mut cur) = self.current_user {
            cur.full_name = final_name;
            if let Some(h) = new_hash {
                cur.password_hash = h;
            }
        }
        self.prof_old_pw.update(cx, |s, cx| s.set_value("", window, cx));
        self.prof_new_pw.update(cx, |s, cx| s.set_value("", window, cx));
        self.prof_confirm_pw.update(cx, |s, cx| s.set_value("", window, cx));
        self.show_profile_modal = false;
        self.show_toast("Đã cập nhật thông tin tài khoản!", false, cx);
    }

    pub fn apply_update_now(&mut self, cx: &mut Context<Self>) {
        if let Some(ref m) = self.available_update {
            if let Err(e) = updater::apply_update_and_restart(&self.cfg.update_dir_path, m) {
                self.show_toast(&format!("Lỗi cập nhật: {}", e), true, cx);
            }
        }
    }

    fn render_login(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        div()
            .size_full()
            .bg(rgb(bg_app()))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(px(24.0))
            .child(
                card_box()
                    .w(px(440.0))
                    .p(px(24.0))
                    .gap(px(14.0))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                gpui::img(self.app_logo.clone())
                                    .h(px(64.0))
                                    .w(px(138.0)),
                            )
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(PRIMARY_COL))
                                    .child("Hệ thống Khám Sức Khỏe (KSK)"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(text_muted()))
                                    .child(format!(
                                        "{} — Phiên bản v{}",
                                        self.cfg.ten_cskcb,
                                        updater::CURRENT_VERSION
                                    )),
                            ),
                    )
                    .when_some(self.available_update.clone(), |el, m| {
                        el.child(
                            div()
                                .p(px(10.0))
                                .rounded(px(6.0))
                                .bg(rgb(WARN_BG))
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(WARN_COL))
                                        .child(format!("Có bản cập nhật mới v{}!", m.version)),
                                )
                                .child(
                                    Button::new("btn-upd-login")
                                        .label("Cập nhật ngay")
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.apply_update_now(cx);
                                        })),
                                ),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Tên đăng nhập", true))
                            .child(Input::new(&self.login_username).small()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Mật khẩu", true))
                            .child(Input::new(&self.login_password).small().mask_toggle()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("File CSDL SQLite (LAN / Nội bộ)", false))
                                    .child(Input::new(&self.login_db_path).small()),
                            )
                            .child(
                                Button::new("btn-login-browse-db")
                                    .label("Chọn...")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(p) = rfd::FileDialog::new()
                                            .add_filter("SQLite DB", &["db", "sqlite"])
                                            .pick_file()
                                        {
                                            let s = p.to_string_lossy().to_string();
                                            this.login_db_path.update(cx, |inp, cx| {
                                                inp.set_value(s, window, cx);
                                            });
                                        }
                                    })),
                            ),
                    )
                    .when_some(self.login_error.clone(), |el, err| {
                        el.child(
                            div()
                                .p(px(8.0))
                                .rounded(px(6.0))
                                .bg(rgb(DANGER_BG))
                                .text_xs()
                                .text_color(rgb(DANGER_COL))
                                .child(err),
                        )
                    })
                    .child(
                        Button::new("btn-login-submit")
                            .label("Đăng nhập")
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.do_login(window, cx);
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl Render for KskGpuiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.current_user.is_none() {
            return self.render_login(window, cx);
        }

        let user = self.current_user.clone().unwrap();
        let is_admin = user.has_role("admin");
        let can_export = user.has_role("admin")
            || user.has_role("reception")
            || user.has_role("bs_ketluan");

        let active_nav = self.active_nav.clone();
        let main_body = match active_nav.as_str() {
            "export" => self.render_export_tab(window, cx),
            "admin" => self.render_admin_tab(window, cx),
            _ => div()
                .flex_1()
                .h_full()
                .flex()
                .flex_row()
                .overflow_hidden()
                .child(self.render_sidebar(window, cx))
                .child(self.render_exam_workspace(window, cx))
                .into_any_element(),
        };

        div()
            .relative()
            .size_full()
            .bg(rgb(bg_app()))
            .text_color(rgb(text_main()))
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                // Top Nav Bar
                div()
                    .h(px(52.0))
                    .px(px(16.0))
                    .bg(rgb(bg_card()))
                    .border_b_1()
                    .border_color(rgb(border_col()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                gpui::img(self.app_logo.clone())
                                    .h(px(30.0))
                                    .w(px(65.0)),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(PRIMARY_COL))
                                    .child("KSK Native"),
                            )
                            .child(
                                Button::new("nav-exam")
                                    .label("Khám bệnh")
                                    .small()
                                    .when(active_nav == "exam", |b| b.primary())
                                    .when(active_nav != "exam", |b| b.secondary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.active_nav = "exam".into();
                                        cx.notify();
                                    })),
                            )
                            .when(can_export, |el| {
                                el.child(
                                    Button::new("nav-export")
                                        .label("Xuất CSV 123 cột")
                                        .small()
                                        .when(active_nav == "export", |b| b.primary())
                                        .when(active_nav != "export", |b| b.secondary())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.active_nav = "export".into();
                                            this.refresh_export_rows(cx);
                                        })),
                                )
                            })
                            .when(is_admin, |el| {
                                el.child(
                                    Button::new("nav-admin")
                                        .label("Quản trị")
                                        .small()
                                        .when(active_nav == "admin", |b| b.primary())
                                        .when(active_nav != "admin", |b| b.secondary())
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.active_nav = "admin".into();
                                            cx.notify();
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .when_some(self.available_update.clone(), |el, m| {
                                el.child(
                                    Button::new("btn-upd-top")
                                        .label(format!("Cập nhật v{}", m.version))
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.apply_update_now(cx);
                                        })),
                                )
                            })
                            .child(
                                Button::new("btn-account-settings")
                                    .label(format!(
                                        "⚙ {} ({})",
                                        user.full_name,
                                        user.roles()
                                            .into_iter()
                                            .map(|r| role_label_vi(&r))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    ))
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_profile_modal = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-logout")
                                    .label("Đăng xuất")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.current_user = None;
                                        this.selected_record_id = None;
                                        this.records.clear();
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .when_some(self.toast.clone(), |el, (msg, is_err)| {
                el.child(
                    div()
                        .px(px(16.0))
                        .py(px(8.0))
                        .bg(rgb(if is_err { DANGER_BG } else { SUCCESS_BG }))
                        .border_b_1()
                        .border_color(rgb(border_col()))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(if is_err { DANGER_COL } else { SUCCESS_COL }))
                                .child(msg),
                        )
                        .child(
                            Button::new("btn-close-toast")
                                .label("✕")
                                .small()
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toast = None;
                                    cx.notify();
                                })),
                        ),
                )
            })
            .child(main_body)
            .when(self.show_profile_modal, |el| {
                el.child(
                    div()
                        .id("profile-modal-backdrop")
                        .absolute()
                        .inset_0()
                        .bg(gpui::hsla(0., 0., 0., 0.45))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            card_box()
                                .w(px(460.0))
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .child("Cài đặt & Tài khoản"),
                                )
                                .child(
                                    div()
                                        .p(px(10.0))
                                        .rounded(px(6.0))
                                        .bg(rgb(bg_subtle()))
                                        .border_1()
                                        .border_color(rgb(border_col()))
                                        .flex()
                                        .flex_col()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(field_label("Chế độ giao diện", false))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .gap(px(6.0))
                                                        .child(
                                                            Button::new("set-theme-light")
                                                                .label("☀ Sáng")
                                                                .small()
                                                                .when(!self.dark_mode, |b| b.primary())
                                                                .when(self.dark_mode, |b| b.secondary())
                                                                .on_click(cx.listener(|this, _, window, cx| {
                                                                    this.dark_mode = false;
                                                                    widgets::set_dark_mode(false);
                                                                    gpui_component::Theme::change(
                                                                        gpui_component::ThemeMode::Light,
                                                                        Some(window),
                                                                        cx,
                                                                    );
                                                                    cx.notify();
                                                                })),
                                                        )
                                                        .child(
                                                            Button::new("set-theme-dark")
                                                                .label("🌙 Tối")
                                                                .small()
                                                                .when(self.dark_mode, |b| b.primary())
                                                                .when(!self.dark_mode, |b| b.secondary())
                                                                .on_click(cx.listener(|this, _, window, cx| {
                                                                    this.dark_mode = true;
                                                                    widgets::set_dark_mode(true);
                                                                    gpui_component::Theme::change(
                                                                        gpui_component::ThemeMode::Dark,
                                                                        Some(window),
                                                                        cx,
                                                                    );
                                                                    cx.notify();
                                                                })),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .justify_between()
                                                .child(field_label("Kiểu khám lâm sàng", false))
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_row()
                                                        .gap(px(6.0))
                                                        .child(
                                                            Button::new("set-exam-catalog")
                                                                .label("Theo danh mục")
                                                                .small()
                                                                .when(self.clinical_catalog_mode, |b| b.primary())
                                                                .when(!self.clinical_catalog_mode, |b| b.secondary())
                                                                .on_click(cx.listener(|this, _, _, cx| {
                                                                    this.clinical_catalog_mode = true;
                                                                    cx.notify();
                                                                })),
                                                        )
                                                        .child(
                                                            Button::new("set-exam-normal")
                                                                .label("Nhập thường")
                                                                .small()
                                                                .when(!self.clinical_catalog_mode, |b| b.primary())
                                                                .when(self.clinical_catalog_mode, |b| b.secondary())
                                                                .on_click(cx.listener(|this, _, _, cx| {
                                                                    this.clinical_catalog_mode = false;
                                                                    cx.notify();
                                                                })),
                                                        ),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(field_label(
                                            if is_admin {
                                                "Họ tên (Admin được sửa)"
                                            } else {
                                                "Họ tên (Chỉ Admin được sửa)"
                                            },
                                            false,
                                        ))
                                        .child(
                                            Input::new(&self.prof_full_name)
                                                .small()
                                                .disabled(!is_admin),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(field_label("Mật khẩu cũ (nếu đổi mật khẩu)", false))
                                        .child(Input::new(&self.prof_old_pw).small()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(field_label("Mật khẩu mới", false))
                                        .child(Input::new(&self.prof_new_pw).small()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(4.0))
                                        .child(field_label("Xác nhận mật khẩu mới", false))
                                        .child(Input::new(&self.prof_confirm_pw).small()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .justify_end()
                                        .gap(px(8.0))
                                        .child(
                                            Button::new("btn-prof-close")
                                                .label("Đóng")
                                                .small()
                                                .secondary()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.show_profile_modal = false;
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Button::new("btn-prof-save")
                                                .label("Lưu thay đổi")
                                                .small()
                                                .primary()
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.save_profile(window, cx);
                                                })),
                                        ),
                                ),
                        ),
                )
            })
            .into_any_element()
    }
}
