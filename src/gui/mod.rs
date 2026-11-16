pub mod sidebar;
pub mod tab_catalogs;
pub mod tab_export_admin;
pub mod tab_group_exam;
pub mod tabs_exam;
pub mod widgets;

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use gpui::{
    AnyElement, AppContext as _, Context, Entity, InteractiveElement, IntoElement, ParentElement,
    Render, StatefulInteractiveElement, Styled, Subscription, Window, div, prelude::FluentBuilder as _, px, rgb,
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
        AppState, generate_csv_bytes, generate_docx_bytes_for_template, generate_group_print_pdf,
        generate_record_pdf_for_templates, query_filtered_records,
    },
    models::{
        CatalogItem, Company, ExamContractSummary, GroupImportRow, KskRecordRow, UpdateManifest,
        User, merge_payload_by_roles,
    },
    updater,
};

use self::widgets::{
    bg_app, bg_card, bg_subtle, border_col, DANGER_BG, DANGER_COL, PRIMARY_COL, primary_light,
    SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, badge, card_box,
    field_label, role_label_vi,
};

pub fn payload_field(row: &KskRecordRow, key: &str) -> String {
    serde_json::from_str::<Map<String, Value>>(&row.payload_json)
        .ok()
        .and_then(|m| m.get(key).and_then(|v| v.as_str()).map(|s| s.to_string()))
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PendingNav {
    Record(i64),
    Tab(String),
    Template(String),
    ClinicalSubtab(String),
    NewPatient,
    Nav(String),
    Logout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionProgress {
    Empty,
    Partial,
    Done,
}

pub fn format_ngay_vao_display(raw: &str) -> String {
    let s = raw.trim();
    if s.len() == 12 && s.chars().all(|c| c.is_ascii_digit()) {
        format!("{}/{}/{} {}:{}", &s[6..8], &s[4..6], &s[0..4], &s[8..10], &s[10..12])
    } else if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
        format!("{}/{}/{}", &s[6..8], &s[4..6], &s[0..4])
    } else {
        s.to_string()
    }
}

fn map_has_val(map: &Map<String, Value>, key: &str) -> bool {
    map.get(key)
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
}

pub fn section_progress_from_map(map: &Map<String, Value>, section: &str) -> SectionProgress {
    match section {
        "reception" => {
            if map_has_val(map, "HO_TEN") {
                SectionProgress::Done
            } else {
                SectionProgress::Empty
            }
        }
        "history" => {
            if map_has_val(map, "TSGD_MAC_BENH")
                || map_has_val(map, "TSBT_MAC_BENH")
                || map_has_val(map, "TIEN_SU_BAN_THAN_KHAC")
            {
                SectionProgress::Done
            } else {
                SectionProgress::Empty
            }
        }
        "physical" => {
            if map_has_val(map, "BS_THE_LUC") {
                SectionProgress::Done
            } else if map_has_val(map, "CHIEU_CAO")
                || map_has_val(map, "CAN_NANG")
                || map_has_val(map, "HUYET_AP")
            {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_noi" => {
            if map_has_val(map, "BS_NOI") {
                SectionProgress::Done
            } else if crate::models::get_section_fields("clin_noi")
                .iter()
                .any(|f| map_has_val(map, f))
            {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_ngoai" => {
            if map_has_val(map, "BS_NGOAI") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_QUA_KHAM_NGOAI_KHOA") || map_has_val(map, "KHAM_NGOAI_KHOA_PL") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_dalieu" => {
            if map_has_val(map, "BS_DA_LIEU") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_QUA_KHAM_DA_LIEU") || map_has_val(map, "KHAM_DA_LIEU_PL") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_san" => {
            if map_has_val(map, "BS_SAN") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_QUA_KHAM_SAN_PHU_KHOA") || map_has_val(map, "KHAM_SAN_PHU_KHOA_PL") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_mat" => {
            if map_has_val(map, "BS_MAT") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_QUA_KHAM_MAT") || map_has_val(map, "KHAM_MAT_PL") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_tmh" => {
            if map_has_val(map, "BS_TMH") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_QUA_KHAM_TMH") || map_has_val(map, "KHAM_TMH_PL") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "clin_rhm" => {
            if map_has_val(map, "BS_RHM") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_QUA_KHAM_RHM") || map_has_val(map, "KHAM_RHM_PL") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "paraclinical" => {
            if map_has_val(map, "BS_CLS") {
                SectionProgress::Done
            } else if crate::models::get_section_fields("paraclinical")
                .iter()
                .any(|f| map_has_val(map, f))
            {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        "conclusion" => {
            if map_has_val(map, "BS_KET_LUAN") || map_has_val(map, "PHAN_LOAI_SK") {
                SectionProgress::Done
            } else if map_has_val(map, "KET_LUAN_BENH") {
                SectionProgress::Partial
            } else {
                SectionProgress::Empty
            }
        }
        _ => SectionProgress::Empty,
    }
}

pub fn clinical_tab_progress(map: &Map<String, Value>) -> SectionProgress {
    let subs = [
        "clin_noi",
        "clin_ngoai",
        "clin_dalieu",
        "clin_san",
        "clin_mat",
        "clin_tmh",
        "clin_rhm",
    ];
    let mut done_cnt = 0usize;
    let mut any_partial = false;
    for s in subs {
        match section_progress_from_map(map, s) {
            SectionProgress::Done => done_cnt += 1,
            SectionProgress::Partial => any_partial = true,
            SectionProgress::Empty => {}
        }
    }
    if done_cnt == subs.len() {
        SectionProgress::Done
    } else if done_cnt > 0 || any_partial {
        SectionProgress::Partial
    } else {
        SectionProgress::Empty
    }
}

pub fn record_progress_summary(rec: &KskRecordRow) -> (usize, usize, Vec<&'static str>) {
    let map: Map<String, Value> = serde_json::from_str(&rec.payload_json).unwrap_or_default();
    let tpls = crate::templates::parse_template_list(&rec.exam_templates);
    if tpls.iter().any(|t| t == "TREN_18") {
        let checks: &[(&str, &str)] = &[
            ("history", "Tiền sử"),
            ("physical", "Thể lực"),
            ("clin_noi", "Nội"),
            ("clin_ngoai", "Ngoại"),
            ("clin_dalieu", "Da liễu"),
            ("clin_san", "Sản"),
            ("clin_mat", "Mắt"),
            ("clin_tmh", "TMH"),
            ("clin_rhm", "RHM"),
            ("paraclinical", "CLS"),
            ("conclusion", "Kết luận"),
        ];
        let mut done = 0usize;
        let mut missing = Vec::new();
        for (sec, lbl) in checks {
            if section_progress_from_map(&map, sec) == SectionProgress::Done {
                done += 1;
            } else {
                missing.push(*lbl);
            }
        }
        (done, checks.len(), missing)
    } else {
        let primary = tpls.first().map(|s| s.as_str()).unwrap_or("MAU_GIAO");
        let tabs = [
            ("history", "Tiền sử"),
            ("physical", "Thể lực"),
            ("clinical", "Lâm sàng"),
            ("conclusion", "Kết luận"),
        ];
        let mut done = 0usize;
        let mut missing = Vec::new();
        for (tab, lbl) in tabs {
            let grps = crate::templates::get_template_groups(primary, tab);
            let any_filled = grps.iter().any(|g| {
                g.fields.iter().any(|f| {
                    let k = crate::templates::namespaced_key(primary, f.key);
                    map_has_val(&map, &k)
                })
            });
            if any_filled {
                done += 1;
            } else {
                missing.push(lbl);
            }
        }
        (done, tabs.len(), missing)
    }
}

pub fn next_exam_tab_in_order(active_tpl: &str, current_tab: &str, delta: isize) -> String {
    let order: &[&str] = if active_tpl == "TREN_18" {
        &["reception", "history", "physical", "clinical", "paraclinical", "conclusion"]
    } else {
        &["reception", "history", "physical", "clinical", "conclusion"]
    };
    let idx = order.iter().position(|t| *t == current_tab).unwrap_or(0);
    let len = order.len() as isize;
    let next = ((idx as isize + delta).rem_euclid(len)) as usize;
    order[next].to_string()
}

pub fn next_section_for_user(
    active_tpl: &str,
    current_tab: &str,
    current_subtab: &str,
    user: &User,
) -> Option<(String, Option<String>)> {
    if active_tpl != "TREN_18" {
        let order = ["reception", "history", "physical", "clinical", "conclusion"];
        let idx = order.iter().position(|t| *t == current_tab)?;
        return order.get(idx + 1).map(|t| ((*t).to_string(), None));
    }

    let flow: &[(&str, Option<&str>, &[&str])] = &[
        ("reception", None, &["admin", "reception"]),
        ("history", None, &["admin", "tiensu", "reception"]),
        ("physical", None, &["admin", "bs_theluc"]),
        ("clinical", Some("noi"), &["admin", "bs_noi"]),
        ("clinical", Some("ngoai"), &["admin", "bs_ngoai"]),
        ("clinical", Some("dalieu"), &["admin", "bs_dalieu"]),
        ("clinical", Some("san"), &["admin", "bs_san"]),
        ("clinical", Some("mat"), &["admin", "bs_mat"]),
        ("clinical", Some("tmh"), &["admin", "bs_tmh"]),
        ("clinical", Some("rhm"), &["admin", "bs_rhm"]),
        ("paraclinical", None, &["admin", "bs_cls", "cls", "xetnghiem", "cdha"]),
        ("conclusion", None, &["admin", "bs_ketluan"]),
    ];

    let cur_idx = flow.iter().position(|(t, sub, _)| {
        *t == current_tab && (*t != "clinical" || *sub == Some(current_subtab))
    })?;

    for (tab, sub, roles) in flow.iter().skip(cur_idx + 1) {
        if roles.iter().any(|r| user.has_role(r)) {
            return Some(((*tab).to_string(), sub.map(|s| s.to_string())));
        }
    }
    None
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
    pub master_key: Option<[u8; 32]>,

    // First-time setup & Recovery Key
    pub needs_first_setup: bool,
    pub setup_db_path: Entity<InputState>,
    pub setup_admin_pw: Entity<InputState>,
    pub setup_admin_pw_confirm: Entity<InputState>,
    pub setup_generated_recovery_key: Option<String>,
    pub setup_recovery_saved_ack: bool,
    pub setup_error: Option<String>,
    pub show_recovery_panel: bool,
    pub recov_key_input: Entity<InputState>,
    pub recov_new_pw: Entity<InputState>,
    pub recov_confirm_pw: Entity<InputState>,

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
    pub toast_seq: u64,
    pub available_update: Option<UpdateManifest>,
    pub focus_handle: gpui::FocusHandle,

    // UX Safety & Workflow Modals
    pub is_dirty: bool,
    pub pending_nav: Option<PendingNav>,
    pub confirm_lock_modal: bool,
    pub lock_missing_sections: Vec<String>,
    pub confirm_logout_modal: bool,
    pub print_modal_action: Option<(bool, bool)>,
    pub print_modal_templates: Vec<String>,

    // Sidebar
    pub sidebar_open: bool,
    pub sidebar_exam_mode: String, // "le" | "doan"
    pub sidebar_contract_id: Option<i64>,
    pub sidebar_search: Entity<InputState>,
    pub sidebar_from_date: Entity<InputState>,
    pub sidebar_to_date: Entity<InputState>,
    pub sidebar_status_filter: Option<i64>,
    pub sidebar_only_my_pending: bool,
    pub records: Vec<KskRecordRow>,

    // Current Exam Record
    pub selected_record_id: Option<i64>,
    pub is_creating_new: bool,
    pub current_record_status: i64,
    pub current_record_exam_mode: String,
    pub current_record_contract_id: Option<i64>,
    pub current_record_templates: String,
    pub active_record_template: String,
    pub editing_section: Option<String>,
    pub form_values: HashMap<String, String>,
    pub form_inputs: HashMap<String, Entity<InputState>>,
    pub form_textareas: HashMap<String, Entity<TextareaState>>,
    pub form_subs: HashMap<String, Subscription>,

    // Clinical Disease Tree & Subtab
    pub active_clinical_subtab: String,
    pub active_noi_subtab: String,
    pub active_custom_subtab: Option<usize>,
    pub active_reception_subtab: Option<usize>,
    pub registered_tab_inputs: Vec<String>,
    pub pending_focus_first_input: bool,
    #[allow(dead_code)]
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
    pub exp_exam_mode: String,
    pub exp_contract_id: Option<i64>,
    pub export_rows: Vec<KskRecordRow>,
    pub export_selected_ids: HashSet<i64>,

    // Group Exam (Công ty & Hợp đồng 2 cấp + Dialog + Chi tiết HĐ)
    pub grp_search: Entity<InputState>,
    pub grp_collapsed_companies: HashSet<i64>,
    pub grp_show_company_dialog: bool,
    pub grp_show_contract_dialog: bool,
    pub grp_detail_contract_id: Option<i64>,
    pub grp_detail_records: Vec<KskRecordRow>,
    pub grp_detail_status_filter: Option<i64>,
    pub grp_show_import_dialog: bool,
    pub grp_show_split_dialog: bool,
    pub grp_split_selected_ids: Vec<i64>,
    pub grp_merge_preview: Option<db::MergePreviewSummary>,
    pub grp_merge_overwrite_all: bool,
    pub companies_list: Vec<Company>,
    pub contracts_list: Vec<ExamContractSummary>,
    pub grp_comp_edit_id: Option<i64>,
    pub grp_comp_code: Entity<InputState>,
    pub grp_comp_name: Entity<InputState>,
    pub grp_comp_tax: Entity<InputState>,
    pub grp_comp_addr: Entity<InputState>,
    pub grp_comp_contact: Entity<InputState>,
    pub grp_comp_phone: Entity<InputState>,
    pub grp_comp_note: Entity<InputState>,
    pub grp_cont_edit_id: Option<i64>,
    pub grp_cont_company_id: Option<i64>,
    pub grp_cont_code: Entity<InputState>,
    pub grp_cont_title: Entity<InputState>,
    pub grp_cont_date: Entity<InputState>,
    pub grp_cont_expected: Entity<InputState>,
    pub grp_cont_templates: String,
    pub grp_cont_note: Entity<InputState>,
    pub grp_import_contract_id: Option<i64>,
    pub grp_import_file_path: String,
    pub grp_import_rows: Vec<GroupImportRow>,
    pub grp_show_member_dialog: bool,
    pub grp_member_contract_id: Option<i64>,
    pub grp_member_edit_id: Option<i64>,
    pub grp_mem_templates: String,
    pub grp_mem_ma_tn: Entity<InputState>,
    pub grp_mem_ma_yte: Entity<InputState>,
    pub grp_mem_ho_ten: Entity<InputState>,
    pub grp_mem_ngay_sinh: Entity<InputState>,
    pub grp_mem_gioi_tinh: String,
    pub grp_mem_so_cccd: Entity<InputState>,
    pub grp_mem_ngaycap_cccd: Entity<InputState>,
    pub grp_mem_noicap_cccd: Entity<InputState>,
    pub grp_mem_dien_thoai: Entity<InputState>,
    pub grp_mem_matinh: String,
    pub grp_mem_maxa: String,
    pub grp_mem_dia_chi: Entity<InputState>,
    pub grp_mem_ma_dantoc: String,
    pub grp_mem_ma_nghe: String,
    pub grp_mem_noi_lam_viec: Entity<InputState>,
    pub grp_mem_ma_loai_kcb: String,
    pub grp_mem_ngay_vao: Entity<InputState>,
    pub grp_mem_nhom_mau: String,
    pub grp_mem_ly_do: Entity<InputState>,
    pub grp_show_bulk_edit_dialog: bool,
    pub grp_bulk_contract_id: Option<i64>,
    pub grp_bulk_apply_workplace: bool,
    pub grp_bulk_workplace: Entity<InputState>,
    pub grp_bulk_apply_nguon: bool,
    pub grp_bulk_nguon: String,
    pub grp_bulk_apply_doituong: bool,
    pub grp_bulk_doituong: String,
    pub grp_bulk_apply_lydo: bool,
    pub grp_bulk_lydo: Entity<InputState>,
    pub grp_bulk_apply_templates: bool,
    pub grp_bulk_templates: String,

    // Admin Tab
    pub admin_sub_tab: String,
    pub adm_cskcb: Entity<InputState>,
    pub adm_ma_cskcb: Entity<InputState>,
    pub adm_db_path: Entity<InputState>,
    pub adm_update_dir: Entity<InputState>,
    pub adm_auto_check_update: bool,
    pub adm_changelog: Entity<InputState>,
    pub adm_auto_backup: bool,
    pub adm_backup_dir: Entity<InputState>,
    pub adm_backup_interval: Entity<InputState>,
    pub last_backup_status: Option<String>,
    pub adm_new_db_path: Entity<InputState>,
    pub adm_new_db_admin_pw: Entity<InputState>,
    pub adm_new_db_admin_pw_confirm: Entity<InputState>,
    pub adm_new_db_recovery_key: Option<String>,
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
        let today = chrono::Local::now().format("%d/%m/%Y").to_string();
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
                .placeholder("dd/mm/yyyy")
                .default_value(today.clone())
        });
        let sidebar_to_date = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("dd/mm/yyyy")
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
                .placeholder("dd/mm/yyyy")
                .default_value(today.clone())
        });
        let exp_to_date = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("dd/mm/yyyy")
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
        let adm_backup_dir = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Thư mục lưu bản sao lưu (.db)")
                .default_value(cfg.backup_dir_path.clone())
        });
        let adm_backup_interval = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Số phút")
                .default_value(cfg.backup_interval_minutes.to_string())
        });
        let adm_new_db_path = cx.new(|cx| {
            InputState::new(window, cx).placeholder("VD: D:\\KSK\\ksk_moi.db")
        });
        let adm_new_db_admin_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu Admin cho DB mới (>= 4 ký tự)")
                .masked(true)
        });
        let adm_new_db_admin_pw_confirm = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Nhập lại mật khẩu Admin")
                .masked(true)
        });

        let setup_db_path = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Đường dẫn file SQLite (.db)")
                .default_value(cfg.database_path.clone())
        });
        let setup_admin_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu Admin mới")
                .masked(true)
        });
        let setup_admin_pw_confirm = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Nhập lại mật khẩu Admin")
                .masked(true)
        });
        let recov_key_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("KSK-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX")
        });
        let recov_new_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Mật khẩu Admin mới")
                .masked(true)
        });
        let recov_confirm_pw = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Nhập lại mật khẩu mới")
                .masked(true)
        });

        let st_check = launch_ctx.state.clone();
        let needs_first_setup = launch_ctx.rt.block_on(async move {
            let pool = st_check.db.read().await;
            !db::has_master_key_setup(&pool).await
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
            master_key: None,
            needs_first_setup,
            setup_db_path,
            setup_admin_pw,
            setup_admin_pw_confirm,
            setup_generated_recovery_key: None,
            setup_recovery_saved_ack: false,
            setup_error: None,
            show_recovery_panel: false,
            recov_key_input,
            recov_new_pw,
            recov_confirm_pw,
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
            toast_seq: 0,
            available_update,
            focus_handle: cx.focus_handle(),
            is_dirty: false,
            pending_nav: None,
            confirm_lock_modal: false,
            lock_missing_sections: Vec::new(),
            confirm_logout_modal: false,
            print_modal_action: None,
            print_modal_templates: Vec::new(),
            sidebar_open: true,
            sidebar_exam_mode: "le".into(),
            sidebar_contract_id: None,
            sidebar_search,
            sidebar_from_date,
            sidebar_to_date,
            sidebar_status_filter: None,
            sidebar_only_my_pending: false,
            records: Vec::new(),
            selected_record_id: None,
            is_creating_new: false,
            current_record_status: 0,
            current_record_exam_mode: "le".into(),
            current_record_contract_id: None,
            current_record_templates: "TREN_18".into(),
            active_record_template: "TREN_18".into(),
            editing_section: None,
            form_values: HashMap::new(),
            form_inputs: HashMap::new(),
            form_textareas: HashMap::new(),
            form_subs: HashMap::new(),
            active_clinical_subtab: "noi".into(),
            active_noi_subtab: "CK_NOI_TIM_MACH".into(),
            active_custom_subtab: None,
            active_reception_subtab: None,
            registered_tab_inputs: Vec::new(),
            pending_focus_first_input: false,
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
            exp_exam_mode: String::new(),
            exp_contract_id: None,
            export_rows: Vec::new(),
            export_selected_ids: HashSet::new(),
            grp_search: cx.new(|cx| InputState::new(window, cx).placeholder("Tìm công ty, mã HĐ, tên đợt khám...")),
            grp_collapsed_companies: HashSet::new(),
            grp_show_company_dialog: false,
            grp_show_contract_dialog: false,
            grp_detail_contract_id: None,
            grp_detail_records: Vec::new(),
            grp_detail_status_filter: None,
            grp_show_import_dialog: false,
            grp_show_split_dialog: false,
            grp_split_selected_ids: Vec::new(),
            grp_merge_preview: None,
            grp_merge_overwrite_all: false,
            companies_list: Vec::new(),
            contracts_list: Vec::new(),
            grp_comp_edit_id: None,
            grp_comp_code: cx.new(|cx| InputState::new(window, cx).placeholder("VD: CTY01")),
            grp_comp_name: cx.new(|cx| InputState::new(window, cx).placeholder("Tên công ty / đơn vị")),
            grp_comp_tax: cx.new(|cx| InputState::new(window, cx).placeholder("Mã số thuế")),
            grp_comp_addr: cx.new(|cx| InputState::new(window, cx).placeholder("Địa chỉ công ty")),
            grp_comp_contact: cx.new(|cx| InputState::new(window, cx).placeholder("Người liên hệ")),
            grp_comp_phone: cx.new(|cx| InputState::new(window, cx).placeholder("Số điện thoại")),
            grp_comp_note: cx.new(|cx| InputState::new(window, cx).placeholder("Ghi chú")),
            grp_cont_edit_id: None,
            grp_cont_company_id: None,
            grp_cont_code: cx.new(|cx| InputState::new(window, cx).placeholder("VD: HD-2026-001")),
            grp_cont_title: cx.new(|cx| InputState::new(window, cx).placeholder("VD: Khám SK định kỳ Q4/2026")),
            grp_cont_date: cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder("dd/mm/yyyy")
                    .default_value(chrono::Local::now().format("%d/%m/%Y").to_string())
            }),
            grp_cont_expected: cx.new(|cx| InputState::new(window, cx).placeholder("Số lượng dự kiến")),
            grp_cont_templates: "TREN_18".into(),
            grp_cont_note: cx.new(|cx| InputState::new(window, cx).placeholder("Ghi chú hợp đồng")),
            grp_import_contract_id: None,
            grp_import_file_path: String::new(),
            grp_import_rows: Vec::new(),
            grp_show_member_dialog: false,
            grp_member_contract_id: None,
            grp_member_edit_id: None,
            grp_mem_templates: "TREN_18".into(),
            grp_mem_ma_tn: cx.new(|cx| InputState::new(window, cx).placeholder("VD: HD01-0001")),
            grp_mem_ma_yte: cx.new(|cx| InputState::new(window, cx).placeholder("Mã y tế")),
            grp_mem_ho_ten: cx.new(|cx| InputState::new(window, cx).placeholder("Họ và tên")),
            grp_mem_ngay_sinh: cx.new(|cx| InputState::new(window, cx).placeholder("dd/mm/yyyy")),
            grp_mem_gioi_tinh: "1".into(),
            grp_mem_so_cccd: cx.new(|cx| InputState::new(window, cx).placeholder("Số CCCD")),
            grp_mem_ngaycap_cccd: cx.new(|cx| InputState::new(window, cx).placeholder("dd/mm/yyyy")),
            grp_mem_noicap_cccd: cx.new(|cx| InputState::new(window, cx).placeholder("Nơi cấp CCCD")),
            grp_mem_dien_thoai: cx.new(|cx| InputState::new(window, cx).placeholder("Số điện thoại")),
            grp_mem_matinh: String::new(),
            grp_mem_maxa: String::new(),
            grp_mem_dia_chi: cx.new(|cx| InputState::new(window, cx).placeholder("Số nhà, đường...")),
            grp_mem_ma_dantoc: "01".into(),
            grp_mem_ma_nghe: "00".into(),
            grp_mem_noi_lam_viec: cx.new(|cx| InputState::new(window, cx).placeholder("Công ty / Bộ phận...")),
            grp_mem_ma_loai_kcb: "1".into(),
            grp_mem_ngay_vao: cx.new(|cx| InputState::new(window, cx).placeholder("dd/mm/yyyy")),
            grp_mem_nhom_mau: String::new(),
            grp_mem_ly_do: cx.new(|cx| InputState::new(window, cx).placeholder("Khám sức khỏe")),
            grp_show_bulk_edit_dialog: false,
            grp_bulk_contract_id: None,
            grp_bulk_apply_workplace: true,
            grp_bulk_workplace: cx.new(|cx| InputState::new(window, cx).placeholder("Nơi làm việc / Học tập")),
            grp_bulk_apply_nguon: true,
            grp_bulk_nguon: "9".into(),
            grp_bulk_apply_doituong: true,
            grp_bulk_doituong: "13".into(),
            grp_bulk_apply_lydo: true,
            grp_bulk_lydo: cx.new(|cx| InputState::new(window, cx).placeholder("Khám sức khỏe định kỳ")),
            grp_bulk_apply_templates: false,
            grp_bulk_templates: "TREN_18".into(),
            admin_sub_tab: "config".into(),
            adm_cskcb,
            adm_ma_cskcb,
            adm_db_path,
            adm_update_dir,
            adm_changelog,
            adm_auto_backup: launch_ctx.initial_cfg.auto_backup_enabled,
            adm_backup_dir,
            adm_backup_interval,
            last_backup_status: None,
            adm_new_db_path,
            adm_new_db_admin_pw,
            adm_new_db_admin_pw_confirm,
            adm_new_db_recovery_key: None,
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
                let prev = this.form_values.get(&key_owned).cloned().unwrap_or_default();
                if val != prev && this.editing_section.is_some() && !key_owned.ends_with("CHI_SO_BMI") && key_owned != "BMI" {
                    this.is_dirty = true;
                }
                this.form_values.insert(key_owned.clone(), val);
                if key_owned.ends_with("CHIEU_CAO") || key_owned.ends_with("CHIEU_DAI") || key_owned.ends_with("CAN_NANG") {
                    this.recompute_bmi(window, cx);
                }
            } else if matches!(ev, InputEvent::PressEnter { secondary: false, .. }) {
                this.focus_next_form_input(&key_owned, window, cx);
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
                let prev = this.form_values.get(&key_owned).cloned().unwrap_or_default();
                if val != prev && this.editing_section.is_some() {
                    this.is_dirty = true;
                }
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
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing_section = Some(section.to_string());
        cx.notify();
    }

    fn recompute_bmi(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for prefix in ["", "D6_", "U18_", "MG_", "HS_"] {
            let cc_key = format!("{}CHIEU_CAO", prefix);
            let cn_key = format!("{}CAN_NANG", prefix);
            let bmi_key = format!("{}CHI_SO_BMI", prefix);
            let h_cm = self
                .get_form_val(&cc_key)
                .replace(',', ".")
                .parse::<f64>()
                .unwrap_or(0.0);
            let w_kg = self
                .get_form_val(&cn_key)
                .replace(',', ".")
                .parse::<f64>()
                .unwrap_or(0.0);
            if h_cm > 30.0 && w_kg > 2.0 {
                let h_m = h_cm / 100.0;
                let bmi = w_kg / (h_m * h_m);
                let bmi_str = format!("{:.1}", bmi);
                self.form_values
                    .insert(bmi_key.clone(), bmi_str.clone());
                if let Some(ent) = self.form_inputs.get(&bmi_key) {
                    ent.update(cx, |s, cx| s.set_value(bmi_str, window, cx));
                }
            }
        }
    }

    pub fn show_toast(&mut self, msg: &str, is_err: bool, cx: &mut Context<Self>) {
        self.toast_seq = self.toast_seq.wrapping_add(1);
        let my_seq = self.toast_seq;
        self.toast = Some((msg.to_string(), is_err));
        let secs = if is_err { 8 } else { 4 };
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(secs))
                .await;
            let _ = this.update(cx, |app, cx| {
                if app.toast_seq == my_seq {
                    app.toast = None;
                    cx.notify();
                }
            });
        })
        .detach();
        cx.notify();
    }

    pub fn handle_his_push_event(&mut self, ma_bn: &str, cx: &mut Context<Self>) {
        tracing::info!("GUI nhan su kien HIS Push cho ma_bn={}", ma_bn);
        let state = self.state.clone();
        let mk = self.master_key;
        let rows = self.run_sync(async move {
            let pool = state.db.read().await;
            query_filtered_records(
                &pool,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some("le"),
                None,
                Some(200),
                true,
                mk.as_ref(),
            )
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
        self.current_record_exam_mode = if rec.exam_mode.is_empty() {
            "le".to_string()
        } else {
            rec.exam_mode.clone()
        };
        self.current_record_contract_id = rec.contract_id;
        let tpls = crate::templates::parse_template_list(&rec.exam_templates);
        self.current_record_templates = crate::templates::format_template_list(&tpls);
        if !tpls.contains(&self.active_record_template) {
            self.active_record_template = tpls
                .first()
                .cloned()
                .unwrap_or_else(|| "TREN_18".to_string());
        }
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
        if self.sidebar_exam_mode == "doan" && self.sidebar_contract_id.is_none() {
            self.records.clear();
            cx.notify();
            return;
        }
        let q = self.sidebar_search.read(cx).value().trim().to_string();
        let from_d = self.sidebar_from_date.read(cx).value().trim().to_string();
        let to_d = self.sidebar_to_date.read(cx).value().trim().to_string();
        let st = self.sidebar_status_filter;
        let mode = self.sidebar_exam_mode.clone();
        let contract_id = if mode == "doan" {
            self.sidebar_contract_id
        } else {
            None
        };
        let state = self.state.clone();

        let mk = self.master_key;
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
                Some(mode.as_str()),
                contract_id,
                Some(200),
                false,
                mk.as_ref(),
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
        let mk = self.master_key;
        let rec_opt = self.run_sync(async move {
            let pool = state.db.read().await;
            let mut r = sqlx::query_as::<_, KskRecordRow>(
                "SELECT id, ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json, updated_by, created_at, updated_at FROM ksk_records WHERE id = ?",
            )
            .bind(record_id)
            .fetch_optional(&*pool)
            .await
            .ok()
            .flatten();
            if let Some(ref mut row) = r {
                crate::crypto::decrypt_record_row(mk.as_ref(), row);
            }
            r
        });

        if let Some(rec) = rec_opt {
            self.load_record_map(&rec);
            self.is_dirty = false;
            self.form_inputs.clear();
            self.form_textareas.clear();
            self.form_subs.clear();
            self.apply_role_landing();
            self.auto_enter_edit_for_current_view(_window, cx);
            cx.notify();
        }
    }

    pub fn current_section_key_and_can_edit(&self) -> Option<(String, bool)> {
        let user = self.current_user.as_ref()?;
        if self.current_record_status == 2 {
            return None;
        }
        let selected_tpls = crate::templates::parse_template_list(&self.current_record_templates);
        let active_tpl = if selected_tpls.contains(&self.active_record_template) {
            self.active_record_template.as_str()
        } else {
            selected_tpls.first().map(|s| s.as_str()).unwrap_or("TREN_18")
        };

        if self.active_exam_tab == "reception" {
            return Some(("reception".to_string(), user.has_role("admin") || user.has_role("reception")));
        }
        if active_tpl != "TREN_18" {
            let tab_key = if self.active_exam_tab == "paraclinical" {
                "clinical"
            } else {
                self.active_exam_tab.as_str()
            };
            return Some((format!("tpl_{}_{}", active_tpl, tab_key), true));
        }

        match self.active_exam_tab.as_str() {
            "history" => Some((
                "history".to_string(),
                user.has_role("admin") || user.has_role("tiensu") || user.has_role("reception"),
            )),
            "physical" => Some((
                "physical".to_string(),
                user.has_role("admin") || user.has_role("bs_theluc"),
            )),
            "clinical" => match self.active_clinical_subtab.as_str() {
                "noi" => Some(("clin_noi".to_string(), user.has_role("admin") || user.has_role("bs_noi"))),
                "ngoai" => Some(("clin_ngoai".to_string(), user.has_role("admin") || user.has_role("bs_ngoai"))),
                "dalieu" => Some(("clin_dalieu".to_string(), user.has_role("admin") || user.has_role("bs_dalieu"))),
                "san" => Some(("clin_san".to_string(), user.has_role("admin") || user.has_role("bs_san"))),
                "mat" => Some(("clin_mat".to_string(), user.has_role("admin") || user.has_role("bs_mat"))),
                "tmh" => Some(("clin_tmh".to_string(), user.has_role("admin") || user.has_role("bs_tmh"))),
                "rhm" => Some(("clin_rhm".to_string(), user.has_role("admin") || user.has_role("bs_rhm"))),
                _ => None,
            },
            "paraclinical" => Some((
                "paraclinical".to_string(),
                user.has_role("admin")
                    || user.has_role("bs_cls")
                    || user.has_role("cls")
                    || user.has_role("xetnghiem")
                    || user.has_role("cdha"),
            )),
            "conclusion" => Some((
                "conclusion".to_string(),
                user.has_role("admin") || user.has_role("bs_ketluan"),
            )),
            _ => None,
        }
    }

    pub fn apply_role_landing(&mut self) {
        let Some(user) = self.current_user.as_ref() else {
            return;
        };
        if user.has_role("admin") {
            return;
        }
        let selected_tpls = crate::templates::parse_template_list(&self.current_record_templates);
        let active_tpl = if selected_tpls.contains(&self.active_record_template) {
            self.active_record_template.as_str()
        } else {
            selected_tpls.first().map(|s| s.as_str()).unwrap_or("TREN_18")
        };

        if active_tpl != "TREN_18" {
            if user.has_role("bs_ketluan") {
                self.active_exam_tab = "conclusion".to_string();
            } else if user.has_role("bs_noi")
                || user.has_role("bs_ngoai")
                || user.has_role("bs_dalieu")
                || user.has_role("bs_san")
                || user.has_role("bs_mat")
                || user.has_role("bs_tmh")
                || user.has_role("bs_rhm")
                || user.has_role("bs_cls")
                || user.has_role("cls")
                || user.has_role("xetnghiem")
                || user.has_role("cdha")
            {
                self.active_exam_tab = "clinical".to_string();
            } else if user.has_role("bs_theluc") {
                self.active_exam_tab = "physical".to_string();
            } else if user.has_role("tiensu") {
                self.active_exam_tab = "history".to_string();
            } else if user.has_role("reception") {
                self.active_exam_tab = "reception".to_string();
            }
            return;
        }

        if user.has_role("bs_ketluan") {
            self.active_exam_tab = "conclusion".to_string();
        } else if user.has_role("bs_cls")
            || user.has_role("cls")
            || user.has_role("xetnghiem")
            || user.has_role("cdha")
        {
            self.active_exam_tab = "paraclinical".to_string();
        } else if user.has_role("bs_noi") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "noi".to_string();
        } else if user.has_role("bs_ngoai") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "ngoai".to_string();
        } else if user.has_role("bs_dalieu") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "dalieu".to_string();
        } else if user.has_role("bs_san") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "san".to_string();
        } else if user.has_role("bs_mat") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "mat".to_string();
        } else if user.has_role("bs_tmh") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "tmh".to_string();
        } else if user.has_role("bs_rhm") {
            self.active_exam_tab = "clinical".to_string();
            self.active_clinical_subtab = "rhm".to_string();
        } else if user.has_role("bs_theluc") {
            self.active_exam_tab = "physical".to_string();
        } else if user.has_role("tiensu") {
            self.active_exam_tab = "history".to_string();
        } else if user.has_role("reception") {
            self.active_exam_tab = "reception".to_string();
        }
    }

    pub fn section_has_data(&self, section: &str) -> bool {
        let has_val = |k: &str| {
            self.form_values
                .get(k)
                .map(|v| !v.trim().is_empty())
                .unwrap_or(false)
        };
        if let Some(rest) = section.strip_prefix("tpl_") {
            if let Some((tpl_code, tab_key)) = rest.rsplit_once('_') {
                for g in crate::templates::get_template_groups(tpl_code, tab_key) {
                    for f in g.fields {
                        let nk = crate::templates::namespaced_key(tpl_code, f.key);
                        if has_val(&nk) {
                            return true;
                        }
                    }
                }
            }
            return false;
        }
        let model_sec = match section {
            "clin_noi" => "clinical_noi",
            "clin_ngoai" => "clinical_ngoai",
            "clin_dalieu" => "clinical_dalieu",
            "clin_san" => "clinical_san",
            "clin_mat" => "clinical_mat",
            "clin_tmh" => "clinical_tmh",
            "clin_rhm" => "clinical_rhm",
            other => other,
        };
        crate::models::get_section_fields(model_sec)
            .iter()
            .any(|f| has_val(f))
    }

    pub fn is_section_examined(&self, section: &str) -> bool {
        if self.is_creating_new {
            return false;
        }
        if let Some(raw) = self.form_values.get("_EXAMINED_SECTIONS") {
            return raw.split(',').any(|s| s.trim() == section);
        }
        if self.selected_record_id.is_some() {
            if section == "reception" {
                return !self.get_form_val("HO_TEN").trim().is_empty();
            }
            return self.section_has_data(section);
        }
        false
    }

    pub fn mark_section_examined(&mut self, section: &str) {
        let mut list: Vec<String> = self
            .form_values
            .get("_EXAMINED_SECTIONS")
            .map(|raw| {
                raw.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();
        if !list.iter().any(|s| s == section) {
            list.push(section.to_string());
        }
        self.form_values
            .insert("_EXAMINED_SECTIONS".to_string(), list.join(","));
    }

    pub fn can_print_full_ksk(&self) -> bool {
        if self.is_creating_new || self.selected_record_id.is_none() {
            return false;
        }
        if self.current_record_status >= 1 {
            return true;
        }
        let selected_tpls = crate::templates::parse_template_list(&self.current_record_templates);
        let active_tpl = if selected_tpls.contains(&self.active_record_template) {
            self.active_record_template.as_str()
        } else {
            selected_tpls.first().map(|s| s.as_str()).unwrap_or("TREN_18")
        };
        if active_tpl == "TREN_18" {
            let is_male = self.get_form_val("GIOI_TINH") == "1";
            self.is_section_examined("reception")
                && self.is_section_examined("history")
                && self.is_section_examined("physical")
                && self.is_section_examined("clin_noi")
                && self.is_section_examined("clin_ngoai")
                && self.is_section_examined("clin_dalieu")
                && (is_male || self.is_section_examined("clin_san"))
                && self.is_section_examined("clin_mat")
                && self.is_section_examined("clin_tmh")
                && self.is_section_examined("clin_rhm")
                && self.is_section_examined("paraclinical")
                && self.is_section_examined("conclusion")
        } else {
            self.is_section_examined("reception")
                && self.is_section_examined(&format!("tpl_{}_history", active_tpl))
                && self.is_section_examined(&format!("tpl_{}_physical", active_tpl))
                && self.is_section_examined(&format!("tpl_{}_clinical", active_tpl))
                && self.is_section_examined(&format!("tpl_{}_conclusion", active_tpl))
        }
    }

    pub fn auto_enter_edit_for_current_view(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_record_id.is_none() && !self.is_creating_new {
            return;
        }
        if let Some((sec, true)) = self.current_section_key_and_can_edit() {
            if !self.is_section_examined(&sec) {
                self.enter_edit_section(&sec, window, cx);
                self.is_dirty = false;
                self.pending_focus_first_input = true;
            } else {
                self.editing_section = None;
                self.pending_focus_first_input = false;
            }
        } else {
            self.editing_section = None;
            self.pending_focus_first_input = false;
        }
    }

    pub fn guard_unsaved(&mut self, nav: PendingNav, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_dirty && self.editing_section.is_some() {
            self.pending_nav = Some(nav);
            self.open_dropdown = None;
            cx.notify();
        } else {
            self.perform_nav(nav, window, cx);
        }
    }

    pub fn perform_nav(&mut self, nav: PendingNav, window: &mut Window, cx: &mut Context<Self>) {
        self.pending_nav = None;
        self.open_dropdown = None;
        match nav {
            PendingNav::Record(rid) => {
                self.active_custom_subtab = None;
                self.active_reception_subtab = None;
                self.load_record_into_form(rid, window, cx);
            }
            PendingNav::Tab(tab) => {
                self.active_exam_tab = tab;
                self.active_custom_subtab = None;
                self.active_reception_subtab = None;
                self.is_dirty = false;
                self.auto_enter_edit_for_current_view(window, cx);
                cx.notify();
            }
            PendingNav::Template(code) => {
                self.active_record_template = code;
                self.active_custom_subtab = None;
                if self.active_record_template != "TREN_18" && self.active_exam_tab == "paraclinical"
                {
                    self.active_exam_tab = "clinical".to_string();
                }
                self.is_dirty = false;
                self.auto_enter_edit_for_current_view(window, cx);
                cx.notify();
            }
            PendingNav::ClinicalSubtab(sub) => {
                self.active_clinical_subtab = sub;
                self.is_dirty = false;
                self.auto_enter_edit_for_current_view(window, cx);
                cx.notify();
            }
            PendingNav::NewPatient => {
                self.active_custom_subtab = None;
                self.active_reception_subtab = None;
                self.start_new_patient(window, cx);
                self.pending_focus_first_input = true;
            }
            PendingNav::Nav(nav_key) => {
                self.active_nav = nav_key.clone();
                if nav_key == "exam" {
                    self.refresh_group_data(cx);
                    self.refresh_records(window, cx);
                } else if nav_key == "group" {
                    self.refresh_group_data(cx);
                } else if nav_key == "export" {
                    self.refresh_export_rows(cx);
                } else {
                    cx.notify();
                }
            }
            PendingNav::Logout => {
                self.current_user = None;
                self.selected_record_id = None;
                self.is_creating_new = false;
                self.is_dirty = false;
                self.editing_section = None;
                self.records.clear();
                cx.notify();
            }
        }
    }

    pub fn focus_first_form_input(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(first_key) = self.registered_tab_inputs.first() {
            if let Some(inp) = self.form_inputs.get(first_key) {
                inp.update(cx, |s, cx| s.focus(window, cx));
            }
        }
    }

    pub fn focus_next_form_input(
        &self,
        current_key: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(idx) = self
            .registered_tab_inputs
            .iter()
            .position(|k| k == current_key)
        {
            if let Some(next_key) = self.registered_tab_inputs.get(idx + 1) {
                if let Some(inp) = self.form_inputs.get(next_key) {
                    inp.update(cx, |s, cx| s.focus(window, cx));
                }
            }
        }
    }

    pub fn cycle_exam_tab(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let selected_tpls = crate::templates::parse_template_list(&self.current_record_templates);
        let active_tpl = if selected_tpls.contains(&self.active_record_template) {
            self.active_record_template.as_str()
        } else {
            selected_tpls.first().map(|s| s.as_str()).unwrap_or("TREN_18")
        };
        let next = next_exam_tab_in_order(active_tpl, &self.active_exam_tab, delta);
        self.guard_unsaved(PendingNav::Tab(next), window, cx);
    }

    pub fn cycle_subtab(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        let selected_tpls = crate::templates::parse_template_list(&self.current_record_templates);
        let active_tpl = if selected_tpls.contains(&self.active_record_template) {
            self.active_record_template.as_str()
        } else {
            selected_tpls.first().map(|s| s.as_str()).unwrap_or("TREN_18")
        };

        if self.active_exam_tab == "reception" {
            let total = 4isize; // None (All), 0, 1, 2
            let cur: isize = match self.active_reception_subtab {
                None => 0,
                Some(i) => (i as isize + 1).clamp(1, 3),
            };
            let nxt = (cur + delta).rem_euclid(total);
            self.active_reception_subtab = if nxt == 0 { None } else { Some((nxt - 1) as usize) };
            self.pending_focus_first_input = true;
            cx.notify();
            return;
        }

        if active_tpl == "TREN_18" && self.active_exam_tab == "clinical" {
            let subs = ["noi", "ngoai", "dalieu", "san", "mat", "tmh", "rhm", "tree"];
            let cur = subs
                .iter()
                .position(|s| *s == self.active_clinical_subtab)
                .unwrap_or(0) as isize;
            let nxt = (cur + delta).rem_euclid(subs.len() as isize) as usize;
            self.guard_unsaved(PendingNav::ClinicalSubtab(subs[nxt].to_string()), window, cx);
            return;
        }

        if active_tpl != "TREN_18" {
            let tab_key = if self.active_exam_tab == "paraclinical" {
                "clinical"
            } else {
                self.active_exam_tab.as_str()
            };
            let groups = crate::templates::get_template_groups(active_tpl, tab_key);
            if groups.len() >= 2 {
                let total = (groups.len() + 1) as isize;
                let cur: isize = match self.active_custom_subtab {
                    None => 0,
                    Some(i) => (i as isize + 1).min(total - 1),
                };
                let nxt = (cur + delta).rem_euclid(total);
                self.active_custom_subtab = if nxt == 0 { None } else { Some((nxt - 1) as usize) };
                self.pending_focus_first_input = true;
                cx.notify();
            }
        }
    }

    pub fn navigate_adjacent_record(
        &mut self,
        delta: isize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.records.is_empty() {
            return;
        }
        let target_id = if let Some(cur) = self.selected_record_id {
            if let Some(pos) = self.records.iter().position(|r| r.id == cur) {
                let next_idx = (pos as isize + delta).clamp(0, (self.records.len() - 1) as isize) as usize;
                if next_idx == pos {
                    return;
                }
                self.records[next_idx].id
            } else {
                self.records[0].id
            }
        } else {
            self.records[0].id
        };
        self.guard_unsaved(PendingNav::Record(target_id), window, cx);
    }

    pub fn request_lock_record(&mut self, cx: &mut Context<Self>) {
        if self.selected_record_id.is_none() || self.current_record_status == 2 {
            return;
        }
        let mut map = Map::new();
        for (k, v) in &self.form_values {
            map.insert(k.clone(), Value::String(v.clone()));
        }
        let payload_json = serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string());
        let dummy = KskRecordRow {
            id: self.selected_record_id.unwrap_or(0),
            ma_tiep_nhan: self.get_form_val("MA_TIEP_NHAN"),
            ma_y_te: self.get_form_val("MA_Y_TE"),
            ho_ten: self.get_form_val("HO_TEN"),
            gioi_tinh: self.get_form_val("GIOI_TINH"),
            ngay_sinh: self.get_form_val("NGAY_SINH"),
            so_cccd: self.get_form_val("SO_CCCD"),
            ngay_vao: self.get_form_val("NGAY_VAO"),
            status: self.current_record_status,
            exam_mode: self.current_record_exam_mode.clone(),
            contract_id: self.current_record_contract_id,
            exam_templates: self.current_record_templates.clone(),
            payload_json,
            updated_by: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        };
        let (_, _, missing) = record_progress_summary(&dummy);
        self.lock_missing_sections = missing.into_iter().map(|s| s.to_string()).collect();
        self.confirm_lock_modal = true;
        cx.notify();
    }

    pub fn start_new_patient(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.selected_record_id = None;
        self.is_creating_new = true;
        self.is_dirty = false;
        self.current_record_status = 0;
        self.current_record_exam_mode = "le".into();
        self.current_record_contract_id = None;
        self.current_record_templates = "TREN_18".into();
        self.active_record_template = "TREN_18".into();
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
    ) -> bool {
        let Some(user) = self.current_user.clone() else {
            return false;
        };
        if let Some(doc_field) = Self::section_doctor_field(section) {
            if self.get_form_val(doc_field).trim().is_empty() && !user.full_name.trim().is_empty()
            {
                self.form_values
                    .insert(doc_field.to_string(), user.full_name.clone());
            }
        }
        self.mark_section_examined(section);
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
        let exam_templates_val = {
            let list = crate::templates::parse_template_list(&self.current_record_templates);
            crate::templates::format_template_list(&list)
        };
        let mk = self.master_key;
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
                let (ext_json_raw, st) = existing.ok_or("Không tìm thấy hồ sơ")?;
                if st == 2 {
                    return Err("Hồ sơ đã ký khóa, không thể chỉnh sửa".into());
                }
                let ext_json = crate::crypto::decrypt_field(mk.as_ref(), &ext_json_raw);
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
                let (enc_ho_ten, enc_ngay_sinh, enc_so_cccd, enc_json_str) =
                    crate::crypto::encrypt_record_fields(
                        mk.as_ref(),
                        &ho_ten,
                        &ngay_sinh,
                        &so_cccd,
                        &json_str,
                    );
                sqlx::query(
                    "UPDATE ksk_records SET ma_tiep_nhan = ?, ma_y_te = ?, ho_ten = ?, gioi_tinh = ?, ngay_sinh = ?, so_cccd = ?, ngay_vao = ?, exam_templates = ?, payload_json = ?, updated_by = ?, updated_at = datetime('now','localtime') WHERE id = ?",
                )
                .bind(&ma_tn)
                .bind(&ma_y_te)
                .bind(&enc_ho_ten)
                .bind(&gioi_tinh)
                .bind(&enc_ngay_sinh)
                .bind(&enc_so_cccd)
                .bind(&ngay_vao)
                .bind(&exam_templates_val)
                .bind(&enc_json_str)
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
                let (enc_ho_ten, enc_ngay_sinh, enc_so_cccd, enc_json_str) =
                    crate::crypto::encrypt_record_fields(
                        mk.as_ref(),
                        &ho_ten,
                        &ngay_sinh,
                        &so_cccd,
                        &json_str,
                    );
                let r = sqlx::query(
                    "INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json, updated_by) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 'le', NULL, ?, ?, ?)",
                )
                .bind(&ma_tn)
                .bind(&ma_y_te)
                .bind(&enc_ho_ten)
                .bind(&gioi_tinh)
                .bind(&enc_ngay_sinh)
                .bind(&enc_so_cccd)
                .bind(&ngay_vao)
                .bind(&exam_templates_val)
                .bind(&enc_json_str)
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
                self.is_dirty = false;
                self.is_creating_new = false;
                self.refresh_records(window, cx);
                self.load_record_into_form(rid, window, cx);
                self.show_toast("Đã lưu thông tin thành công!", false, cx);
                true
            }
            Err(e) => {
                self.show_toast(&format!("Lỗi lưu dữ liệu: {}", e), true, cx);
                false
            }
        }
    }

    pub fn save_and_next_section(
        &mut self,
        section: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let cur_tab = self.active_exam_tab.clone();
        let cur_sub = self.active_clinical_subtab.clone();
        let selected_tpls = crate::templates::parse_template_list(&self.current_record_templates);
        let active_tpl = if selected_tpls.contains(&self.active_record_template) {
            self.active_record_template.clone()
        } else {
            selected_tpls
                .first()
                .cloned()
                .unwrap_or_else(|| "TREN_18".to_string())
        };
        if !self.save_section(section, window, cx) {
            return;
        }
        if let Some(user) = self.current_user.clone() {
            if let Some((next_tab, next_sub)) =
                next_section_for_user(&active_tpl, &cur_tab, &cur_sub, &user)
            {
                self.active_exam_tab = next_tab;
                if let Some(sub) = next_sub {
                    self.active_clinical_subtab = sub;
                }
                self.auto_enter_edit_for_current_view(window, cx);
                cx.notify();
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
                self.is_dirty = false;
                self.refresh_records(window, cx);
                if new_status == 2 {
                    self.show_toast("Đã ký khóa hồ sơ!", false, cx);
                    if let Some(next_rec) = self
                        .records
                        .iter()
                        .find(|r| r.id != rid && r.status == 0)
                        .cloned()
                    {
                        self.load_record_into_form(next_rec.id, window, cx);
                    }
                } else {
                    self.show_toast("Đã mở khóa hồ sơ!", false, cx);
                    self.auto_enter_edit_for_current_view(window, cx);
                }
            }
            Err(e) => self.show_toast(&format!("Lỗi cập nhật trạng thái: {}", e), true, cx),
        }
    }

    pub fn open_print_template_modal(
        &mut self,
        is_docx: bool,
        is_reception: bool,
        cx: &mut Context<Self>,
    ) {
        if self.selected_record_id.is_none() {
            return;
        }
        let mut tpls = crate::templates::parse_template_list(&self.current_record_templates);
        if tpls.is_empty() {
            tpls.push(self.active_record_template.clone());
        }
        if tpls.len() == 1 {
            if is_docx {
                self.execute_export_docx(is_reception, tpls, cx);
            } else {
                self.execute_print_record_html(is_reception, tpls, cx);
            }
            return;
        }
        self.print_modal_templates = tpls;
        self.print_modal_action = Some((is_docx, is_reception));
        self.open_dropdown = None;
        cx.notify();
    }

    pub fn print_record_html(&mut self, is_reception: bool, cx: &mut Context<Self>) {
        self.open_print_template_modal(false, is_reception, cx);
    }

    pub fn execute_print_record_html(
        &mut self,
        is_reception: bool,
        templates: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(rid) = self.selected_record_id else {
            return;
        };
        if templates.is_empty() {
            self.show_toast("Vui lòng chọn ít nhất 1 mẫu để in!", true, cx);
            return;
        }
        let state = self.state.clone();
        let cfg = self.cfg.clone();
        let mk = self.master_key;
        let tpls_for_toast = templates.len();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            generate_record_pdf_for_templates(
                &pool,
                &cfg,
                rid,
                is_reception,
                &templates,
                mk.as_ref(),
            )
            .await
        });
        match res {
            Ok((filename, pdf_bytes)) => {
                let path = std::env::temp_dir().join(filename);
                if let Err(e) = std::fs::write(&path, &pdf_bytes) {
                    self.show_toast(&format!("Lỗi tạo file PDF: {}", e), true, cx);
                    return;
                }
                let path_str = path.to_string_lossy().to_string();
                match std::process::Command::new("cmd")
                    .args(["/C", "start", "", &path_str])
                    .spawn()
                {
                    Ok(_) => self.show_toast(
                        &format!("Đã xuất PDF A4 ({} mẫu)!", tpls_for_toast),
                        false,
                        cx,
                    ),
                    Err(e) => self.show_toast(&format!("Không mở được file PDF: {}", e), true, cx),
                }
            }
            Err(e) => self.show_toast(&format!("Lỗi tạo PDF: {}", e), true, cx),
        }
    }

    pub fn print_group_html(
        &mut self,
        contract_id: i64,
        is_reception: bool,
        cx: &mut Context<Self>,
    ) {
        let state = self.state.clone();
        let cfg = self.cfg.clone();
        let mk = self.master_key;
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            generate_group_print_pdf(&pool, &cfg, contract_id, is_reception, mk.as_ref()).await
        });
        match res {
            Ok((filename, pdf_bytes, count)) => {
                let path = std::env::temp_dir().join(filename);
                if let Err(e) = std::fs::write(&path, &pdf_bytes) {
                    self.show_toast(&format!("Lỗi tạo file PDF đoàn: {}", e), true, cx);
                    return;
                }
                let path_str = path.to_string_lossy().to_string();
                match std::process::Command::new("cmd")
                    .args(["/C", "start", "", &path_str])
                    .spawn()
                {
                    Ok(_) => self.show_toast(
                        &format!("Đã xuất PDF A4 cho {} hồ sơ trong đoàn!", count),
                        false,
                        cx,
                    ),
                    Err(e) => self.show_toast(&format!("Không mở được file PDF: {}", e), true, cx),
                }
            }
            Err(e) => self.show_toast(&e, true, cx),
        }
    }

    #[allow(dead_code)]
    pub fn export_docx(&mut self, is_reception: bool, cx: &mut Context<Self>) {
        self.open_print_template_modal(true, is_reception, cx);
    }

    pub fn execute_export_docx(
        &mut self,
        is_reception: bool,
        templates: Vec<String>,
        cx: &mut Context<Self>,
    ) {
        let Some(rid) = self.selected_record_id else {
            return;
        };
        if templates.is_empty() {
            self.show_toast("Vui lòng chọn ít nhất 1 mẫu để xuất DOCX!", true, cx);
            return;
        }
        let mut saved_count = 0usize;
        for tpl_code in templates {
            let state = self.state.clone();
            let cfg = self.cfg.clone();
            let mk = self.master_key;
            let tpl_c = tpl_code.clone();
            let res = self.run_sync(async move {
                let pool = state.db.read().await;
                generate_docx_bytes_for_template(&pool, &cfg, rid, is_reception, &tpl_c, mk.as_ref())
                    .await
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
                            return;
                        } else {
                            saved_count += 1;
                        }
                    }
                }
                Err(e) => {
                    self.show_toast(&format!("Lỗi xuất DOCX: {}", e), true, cx);
                    return;
                }
            }
        }
        if saved_count > 0 {
            self.show_toast(
                &format!("Đã xuất thành công {} file Word (.docx)!", saved_count),
                false,
                cx,
            );
        }
    }

    pub fn refresh_group_data(&mut self, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let (companies, contracts) = self.run_sync(async move {
            let pool = state.db.read().await;
            let comps = db::list_companies(&pool).await;
            let conts = db::list_exam_contracts(&pool, None).await;
            (comps, conts)
        });
        self.companies_list = companies;
        self.contracts_list = contracts;
        if self.grp_cont_company_id.is_none() {
            self.grp_cont_company_id = self.companies_list.first().map(|c| c.id);
        }
        if self.grp_import_contract_id.is_none() {
            self.grp_import_contract_id = self
                .contracts_list
                .iter()
                .find(|c| c.status == 0)
                .map(|c| c.id);
        }
        if self.sidebar_contract_id.is_none() {
            self.sidebar_contract_id = self
                .contracts_list
                .iter()
                .find(|c| c.status == 0)
                .map(|c| c.id);
        }
        cx.notify();
    }

    pub fn refresh_export_rows(&mut self, cx: &mut Context<Self>) {
        let from_d = self.exp_from_date.read(cx).value().trim().to_string();
        let to_d = self.exp_to_date.read(cx).value().trim().to_string();
        let q = self.exp_q.read(cx).value().trim().to_string();
        let wp = self.exp_workplace.read(cx).value().trim().to_string();
        let st = self.exp_status_filter;
        let mode = self.exp_exam_mode.clone();
        let contract_id = if mode == "doan" {
            self.exp_contract_id
        } else {
            None
        };
        let state = self.state.clone();

        let mk = self.master_key;
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
                if mode.is_empty() { None } else { Some(mode.as_str()) },
                contract_id,
                Some(2000),
                false,
                mk.as_ref(),
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
        new_cfg.auto_backup_enabled = self.adm_auto_backup;
        new_cfg.backup_dir_path = self.adm_backup_dir.read(cx).value().trim().to_string();
        new_cfg.backup_interval_minutes = self
            .adm_backup_interval
            .read(cx)
            .value()
            .trim()
            .parse::<u64>()
            .unwrap_or(60)
            .max(5);

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

    pub fn create_and_switch_new_db(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let db_path = self.adm_new_db_path.read(cx).value().trim().to_string();
        let pw = self.adm_new_db_admin_pw.read(cx).value().to_string();
        let pw2 = self.adm_new_db_admin_pw_confirm.read(cx).value().to_string();

        if db_path.is_empty() {
            self.show_toast("Vui lòng nhập hoặc chọn đường dẫn file DB mới!", true, cx);
            return;
        }
        let path_obj = std::path::Path::new(&db_path);
        if path_obj.exists()
            && std::fs::metadata(path_obj).map(|m| m.len() > 0).unwrap_or(true)
        {
            self.show_toast(
                "File DB đã tồn tại! Vui lòng đặt tên file mới hoặc dùng mục Kết nối lại DB ở trên.",
                true,
                cx,
            );
            return;
        }
        if pw.len() < 4 {
            self.show_toast("Mật khẩu Admin cho DB mới phải có ít nhất 4 ký tự!", true, cx);
            return;
        }
        if pw != pw2 {
            self.show_toast("Mật khẩu xác nhận không khớp!", true, cx);
            return;
        }

        let mut new_cfg = self.cfg.clone();
        new_cfg.database_path = db_path.clone();
        if let Err(e) = new_cfg.save() {
            self.show_toast(&format!("Lỗi lưu cấu hình DB: {}", e), true, cx);
            return;
        }

        let state = self.state.clone();
        let cfg_clone = new_cfg.clone();
        let res = self.run_sync(async move {
            let pool = db::init_db(&db_path).await.map_err(|e| e.to_string())?;
            let (mk, rk) = db::initialize_master_key_for_admin(&pool, &pw).await?;
            *state.db.write().await = pool;
            *state.config.write().await = cfg_clone;
            *state.master_key.write().await = Some(mk);
            Ok::<([u8; 32], String), String>((mk, rk))
        });

        match res {
            Ok((mk, rk)) => {
                self.cfg = new_cfg.clone();
                self.master_key = Some(mk);
                self.adm_new_db_recovery_key = Some(rk);
                self.adm_new_db_admin_pw
                    .update(cx, |s, cx| s.set_value("", window, cx));
                self.adm_new_db_admin_pw_confirm
                    .update(cx, |s, cx| s.set_value("", window, cx));
                self.adm_db_path
                    .update(cx, |s, cx| s.set_value(new_cfg.database_path.clone(), window, cx));
                self.login_db_path
                    .update(cx, |s, cx| s.set_value(new_cfg.database_path.clone(), window, cx));
                self.setup_db_path
                    .update(cx, |s, cx| s.set_value(new_cfg.database_path, window, cx));
                self.selected_record_id = None;
                self.current_record_status = 0;
                self.form_values.clear();
                self.export_rows.clear();
                self.export_selected_ids.clear();
                self.refresh_catalogs(cx);
                self.refresh_users(cx);
                self.refresh_group_data(cx);
                self.refresh_records(window, cx);
                self.show_toast(
                    "Đã tạo DB mới và chuyển sang CSDL mới! Hãy lưu lại Recovery Key.",
                    false,
                    cx,
                );
            }
            Err(e) => self.show_toast(&format!("Lỗi tạo DB mới: {}", e), true, cx),
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
        let wrapped_mk = match self.master_key {
            Some(ref mk) => match crate::crypto::wrap_master_key(mk, &pw) {
                Ok(w) => w,
                Err(e) => {
                    self.show_toast(&format!("Lỗi bọc Master Key: {}", e), true, cx);
                    return;
                }
            },
            None => String::new(),
        };
        let state = self.state.clone();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query(
                "INSERT INTO users (username, password_hash, full_name, role, is_locked, wrapped_mk) VALUES (?, ?, ?, ?, 0, ?)",
            )
            .bind(&username)
            .bind(&hash)
            .bind(&full_name)
            .bind(&roles_csv)
            .bind(&wrapped_mk)
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
            "business" => {
                self.active_nav = "group".into();
            }
            "reception" | "admin" => {
                self.active_nav = "exam".into();
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
        let (user_opt, wrapped_mk) = self.run_sync(async move {
            let pool = state.db.read().await;
            let u = sqlx::query_as::<_, User>(
                "SELECT id, username, password_hash, full_name, role, is_locked, created_at FROM users WHERE username = ?",
            )
            .bind(&username)
            .fetch_optional(&*pool)
            .await
            .ok()
            .flatten();
            let wmk = if let Some(ref usr) = u {
                sqlx::query_as::<_, (String,)>("SELECT wrapped_mk FROM users WHERE id = ?")
                    .bind(usr.id)
                    .fetch_optional(&*pool)
                    .await
                    .ok()
                    .flatten()
                    .map(|(s,)| s)
                    .unwrap_or_default()
            } else {
                String::new()
            };
            (u, wmk)
        });

        match user_opt {
            Some(u) if u.password_hash == hash => {
                if u.is_locked != 0 {
                    self.login_error = Some("Tài khoản đã bị khóa!".into());
                    cx.notify();
                    return;
                }
                if !wrapped_mk.trim().is_empty() {
                    match crate::crypto::unwrap_master_key(&wrapped_mk, &password) {
                        Ok(mk) => {
                            self.master_key = Some(mk);
                            let st_mk = self.state.clone();
                            self.run_sync(async move {
                                *st_mk.master_key.write().await = Some(mk);
                            });
                        }
                        Err(e) => {
                            self.login_error = Some(format!("Không thể giải mã Master Key: {}", e));
                            cx.notify();
                            return;
                        }
                    }
                } else {
                    self.login_error = Some(
                        "Tài khoản chưa được cấp Master Key. Vui lòng dùng Admin hoặc Recovery Key để khôi phục.".into(),
                    );
                    cx.notify();
                    return;
                }
                self.login_error = None;
                self.prof_full_name
                    .update(cx, |s, cx| s.set_value(u.full_name.clone(), window, cx));
                self.set_default_tabs_for_user(&u);
                self.current_user = Some(u);
                self.refresh_catalogs(cx);
                self.refresh_group_data(cx);
                self.refresh_records(window, cx);
                self.start_auto_backup_loop(cx);
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
        let new_wrapped_mk = if !new_pw.is_empty() {
            self.master_key
                .as_ref()
                .and_then(|mk| crate::crypto::wrap_master_key(mk, &new_pw).ok())
        } else {
            None
        };

        let name_c = final_name.clone();
        let hash_c = new_hash.clone();
        let wmk_c = new_wrapped_mk;
        let _ = self.run_sync(async move {
            let pool = state.db.read().await;
            let _ = sqlx::query("UPDATE users SET full_name = ? WHERE id = ?")
                .bind(&name_c)
                .bind(uid)
                .execute(&*pool)
                .await;
            if let Some(h) = hash_c {
                if let Some(wmk) = wmk_c {
                    let _ = sqlx::query("UPDATE users SET password_hash = ?, wrapped_mk = ? WHERE id = ?")
                        .bind(&h)
                        .bind(&wmk)
                        .bind(uid)
                        .execute(&*pool)
                        .await;
                } else {
                    let _ = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
                        .bind(&h)
                        .bind(uid)
                        .execute(&*pool)
                        .await;
                }
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
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_between()
                            .items_center()
                            .child(
                                Button::new("btn-toggle-recovery")
                                    .label(if self.show_recovery_panel {
                                        "Ẩn khôi phục mật khẩu"
                                    } else {
                                        "Quên mật khẩu Admin? Khôi phục bằng Recovery Key"
                                    })
                                    .xsmall()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_recovery_panel = !this.show_recovery_panel;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .when(self.show_recovery_panel, |el| {
                        el.child(
                            div()
                                .p(px(10.0))
                                .rounded(px(8.0))
                                .bg(rgb(bg_subtle()))
                                .border_1()
                                .border_color(rgb(border_col()))
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(text_main()))
                                        .child("Khôi phục tài khoản Admin bằng Recovery Key"),
                                )
                                .child(Input::new(&self.recov_key_input).small())
                                .child(Input::new(&self.recov_new_pw).small().mask_toggle())
                                .child(Input::new(&self.recov_confirm_pw).small().mask_toggle())
                                .child(
                                    Button::new("btn-exec-recovery")
                                        .label("Khôi phục & Đặt mật khẩu mới")
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.run_recovery_admin(window, cx);
                                        })),
                                ),
                        )
                    }),
            )
            .into_any_element()
    }

    pub fn run_first_time_setup(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let db_path = self.setup_db_path.read(cx).value().trim().to_string();
        let pw = self.setup_admin_pw.read(cx).value().to_string();
        let pw2 = self.setup_admin_pw_confirm.read(cx).value().to_string();
        if db_path.is_empty() {
            self.setup_error = Some("Vui lòng chọn đường dẫn cơ sở dữ liệu!".into());
            cx.notify();
            return;
        }
        if pw.len() < 4 {
            self.setup_error = Some("Mật khẩu Admin phải có ít nhất 4 ký tự!".into());
            cx.notify();
            return;
        }
        if pw != pw2 {
            self.setup_error = Some("Mật khẩu xác nhận không khớp!".into());
            cx.notify();
            return;
        }

        let mut new_cfg = self.cfg.clone();
        new_cfg.database_path = db_path.clone();
        if let Err(e) = new_cfg.save() {
            self.setup_error = Some(format!("Lỗi lưu cấu hình DB: {}", e));
            cx.notify();
            return;
        }

        let state = self.state.clone();
        let cfg_c = new_cfg.clone();
        let res = self.run_sync(async move {
            let pool = db::init_db(&db_path).await.map_err(|e| e.to_string())?;
            let (mk, rk) = db::initialize_master_key_for_admin(&pool, &pw).await?;
            *state.db.write().await = pool;
            *state.config.write().await = cfg_c;
            *state.master_key.write().await = Some(mk);
            Ok::<([u8; 32], String), String>((mk, rk))
        });

        match res {
            Ok((mk, rk)) => {
                self.cfg = new_cfg.clone();
                self.master_key = Some(mk);
                self.setup_error = None;
                self.setup_generated_recovery_key = Some(rk);
                self.setup_recovery_saved_ack = false;
                self.login_db_path
                    .update(cx, |s, cx| s.set_value(new_cfg.database_path.clone(), window, cx));
                self.adm_db_path
                    .update(cx, |s, cx| s.set_value(new_cfg.database_path, window, cx));
                cx.notify();
            }
            Err(e) => {
                self.setup_error = Some(e);
                cx.notify();
            }
        }
    }

    pub fn run_recovery_admin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let rk = self.recov_key_input.read(cx).value().trim().to_string();
        let pw = self.recov_new_pw.read(cx).value().to_string();
        let pw2 = self.recov_confirm_pw.read(cx).value().to_string();
        if rk.is_empty() || pw.is_empty() {
            self.login_error = Some("Vui lòng nhập Recovery Key và mật khẩu mới!".into());
            cx.notify();
            return;
        }
        if pw != pw2 {
            self.login_error = Some("Mật khẩu mới và xác nhận không khớp!".into());
            cx.notify();
            return;
        }

        let state = self.state.clone();
        let pw_for_rec = pw.clone();
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            let mk = db::recover_admin_with_key(&pool, &rk, &pw_for_rec).await?;
            *state.master_key.write().await = Some(mk);
            Ok::<[u8; 32], String>(mk)
        });

        match res {
            Ok(mk) => {
                self.master_key = Some(mk);
                self.login_error = None;
                self.show_recovery_panel = false;
                self.recov_key_input.update(cx, |s, cx| s.set_value("", window, cx));
                self.recov_new_pw.update(cx, |s, cx| s.set_value("", window, cx));
                self.recov_confirm_pw.update(cx, |s, cx| s.set_value("", window, cx));
                self.login_username.update(cx, |s, cx| s.set_value("admin", window, cx));
                self.login_password.update(cx, |s, cx| s.set_value(pw, window, cx));
                self.do_login(window, cx);
            }
            Err(e) => {
                self.login_error = Some(e);
                cx.notify();
            }
        }
    }

    pub fn trigger_manual_backup(&mut self, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let db_path = self.cfg.database_path.clone();
        let backup_dir = self.adm_backup_dir.read(cx).value().trim().to_string();
        let keep = self.cfg.backup_keep_count;
        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            db::backup_database(&pool, &db_path, &backup_dir, keep).await
        });
        match res {
            Ok(path) => {
                self.last_backup_status = Some(format!(
                    "Đã sao lưu lúc {}: {}",
                    chrono::Local::now().format("%H:%M:%S %d/%m/%Y"),
                    path
                ));
                self.show_toast(&format!("Đã backup database: {}", path), false, cx);
            }
            Err(e) => {
                self.show_toast(&format!("Lỗi backup DB: {}", e), true, cx);
            }
        }
    }

    pub fn start_auto_backup_loop(&mut self, cx: &mut Context<Self>) {
        if !self.cfg.auto_backup_enabled {
            return;
        }
        let state = self.state.clone();
        let db_path = self.cfg.database_path.clone();
        let backup_dir = self.cfg.backup_dir_path.clone();
        let keep = self.cfg.backup_keep_count;
        let interval_mins = self.cfg.backup_interval_minutes.max(5);

        cx.spawn(async move |this, cx| {
            loop {
                let st_c = state.clone();
                let dp_c = db_path.clone();
                let bd_c = backup_dir.clone();
                let res = cx
                    .background_executor()
                    .spawn(async move {
                        let rt = tokio::runtime::Handle::try_current();
                        if let Ok(handle) = rt {
                            handle.block_on(async move {
                                let pool = st_c.db.read().await;
                                db::backup_database(&pool, &dp_c, &bd_c, keep).await
                            })
                        } else {
                            let rt_new = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
                            rt_new.block_on(async move {
                                let pool = st_c.db.read().await;
                                db::backup_database(&pool, &dp_c, &bd_c, keep).await
                            })
                        }
                    })
                    .await;

                if let Ok(path) = res {
                    let msg = format!(
                        "Tự động sao lưu lúc {}: {}",
                        chrono::Local::now().format("%H:%M:%S %d/%m/%Y"),
                        path
                    );
                    let _ = this.update(cx, |app, cx| {
                        app.last_backup_status = Some(msg);
                        cx.notify();
                    });
                }

                cx.background_executor()
                    .timer(std::time::Duration::from_secs(interval_mins * 60))
                    .await;
            }
        })
        .detach();
    }

    fn render_first_time_setup(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
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
                    .w(px(500.0))
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
                                    .h(px(56.0))
                                    .w(px(121.0)),
                            )
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(PRIMARY_COL))
                                    .child("Cài đặt lần đầu — Khởi tạo Bảo mật & CSDL"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted()))
                                    .child("Tạo cơ sở dữ liệu, mật khẩu Admin và Recovery Key để mã hóa toàn bộ hồ sơ bệnh án (AES-256-GCM)."),
                            ),
                    )
                    .when(self.setup_generated_recovery_key.is_none(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(field_label("Đường dẫn file cơ sở dữ liệu (.db)", true))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap(px(6.0))
                                        .child(div().flex_1().child(Input::new(&self.setup_db_path).small()))
                                        .child(
                                            Button::new("btn-setup-browse-db")
                                                .label("Chọn nơi lưu...")
                                                .small()
                                                .secondary()
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    if let Some(p) = rfd::FileDialog::new()
                                                        .set_file_name("ksk_data.db")
                                                        .add_filter("SQLite DB", &["db", "sqlite"])
                                                        .save_file()
                                                    {
                                                        let s = p.to_string_lossy().to_string();
                                                        this.setup_db_path.update(cx, |inp, cx| {
                                                            inp.set_value(s, window, cx);
                                                        });
                                                    }
                                                })),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(field_label("Mật khẩu Admin", true))
                                .child(Input::new(&self.setup_admin_pw).small().mask_toggle()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(field_label("Xác nhận mật khẩu Admin", true))
                                .child(Input::new(&self.setup_admin_pw_confirm).small().mask_toggle()),
                        )
                        .when_some(self.setup_error.clone(), |err_el, err| {
                            err_el.child(
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
                            Button::new("btn-setup-init")
                                .label("Khởi tạo Database & Tạo Recovery Key")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.run_first_time_setup(window, cx);
                                })),
                        )
                    })
                    .when_some(self.setup_generated_recovery_key.clone(), |el, rk| {
                        let rk_copy = rk.clone();
                        el.child(
                            div()
                                .p(px(12.0))
                                .rounded(px(8.0))
                                .bg(rgb(WARN_BG))
                                .border_1()
                                .border_color(rgb(WARN_COL))
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(WARN_COL))
                                        .child("QUAN TRỌNG: Hãy lưu lại Recovery Key này ở nơi an toàn!"),
                                )
                                .child(
                                    div()
                                        .p(px(10.0))
                                        .rounded(px(6.0))
                                        .bg(rgb(bg_card()))
                                        .border_1()
                                        .border_color(rgb(border_col()))
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(PRIMARY_COL))
                                        .child(rk),
                                )
                                .child(
                                    Button::new("btn-copy-rk")
                                        .label("Sao chép Recovery Key")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                                rk_copy.clone(),
                                            ));
                                            this.show_toast(
                                                "Đã sao chép Recovery Key vào clipboard!",
                                                false,
                                                cx,
                                            );
                                        })),
                                ),
                        )
                        .child(
                            gpui_component::checkbox::Checkbox::new("chk-rk-ack")
                                .label("Tôi đã lưu giữ Recovery Key cẩn thận")
                                .checked(self.setup_recovery_saved_ack)
                                .on_change(cx.listener(|this, val, _, cx| {
                                    this.setup_recovery_saved_ack = *val;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-finish-setup")
                                .label("Hoàn tất & Vào Đăng nhập")
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    if !this.setup_recovery_saved_ack {
                                        this.show_toast(
                                            "Vui lòng xác nhận đã lưu Recovery Key trước khi tiếp tục!",
                                            true,
                                            cx,
                                        );
                                        return;
                                    }
                                    this.needs_first_setup = false;
                                    this.login_username
                                        .update(cx, |s, cx| s.set_value("admin", window, cx));
                                    cx.notify();
                                })),
                        )
                    }),
            )
            .into_any_element()
    }
}

impl Render for KskGpuiApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.needs_first_setup {
            return self.render_first_time_setup(window, cx);
        }
        if self.current_user.is_none() {
            return self.render_login(window, cx);
        }

        let user = self.current_user.clone().unwrap();
        let is_admin = user.has_role("admin");
        let can_group = user.has_role("admin") || user.has_role("business") || user.has_role("reception");
        let can_export = user.has_role("admin")
            || user.has_role("reception")
            || user.has_role("business")
            || user.has_role("bs_ketluan");

        let active_nav = self.active_nav.clone();
        let main_body = match active_nav.as_str() {
            "group" => self.render_group_exam_tab(window, cx),
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

        let brand_title = if self.cfg.ten_cskcb.trim().is_empty() {
            "Hệ thống KSK".to_string()
        } else {
            self.cfg.ten_cskcb.clone()
        };

        div()
            .id("ksk-app-root")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, ev: &gpui::KeyDownEvent, window, cx| {
                let ks = &ev.keystroke;
                let key = ks.key.as_str();
                if ks.modifiers.control && !ks.modifiers.alt && key == "tab" {
                    if this.active_nav == "exam" && (this.selected_record_id.is_some() || this.is_creating_new) {
                        this.cycle_exam_tab(if ks.modifiers.shift { -1 } else { 1 }, window, cx);
                    }
                } else if ks.modifiers.control && !ks.modifiers.alt && !ks.modifiers.shift {
                    match key {
                        "s" | "S" => {
                            if let Some((sec, true)) = this.current_section_key_and_can_edit() {
                                if this.editing_section.as_deref() == Some(sec.as_str()) {
                                    this.save_section(&sec, window, cx);
                                }
                            }
                        }
                        "enter" => {
                            if let Some((sec, true)) = this.current_section_key_and_can_edit() {
                                if this.editing_section.as_deref() == Some(sec.as_str()) {
                                    this.save_and_next_section(&sec, window, cx);
                                }
                            }
                        }
                        "n" | "N" => {
                            if this.has_any_role(&["admin", "reception"]) && this.active_nav == "exam" {
                                this.guard_unsaved(PendingNav::NewPatient, window, cx);
                            }
                        }
                        "f" | "F" => {
                            if this.active_nav == "exam" {
                                this.sidebar_open = true;
                                this.sidebar_search.update(cx, |s, cx| {
                                    s.focus(window, cx);
                                });
                            }
                        }
                        "p" | "P" => {
                            if this.active_nav == "exam" && this.selected_record_id.is_some() {
                                let is_rec = this.active_exam_tab == "reception";
                                this.print_record_html(is_rec, cx);
                            }
                        }
                        _ => {}
                    }
                } else if ks.modifiers.alt && !ks.modifiers.control {
                    if this.active_nav == "exam" && (this.selected_record_id.is_some() || this.is_creating_new) {
                        let tab_opt = match key {
                            "1" => Some("reception"),
                            "2" => Some("history"),
                            "3" => Some("physical"),
                            "4" => Some("clinical"),
                            "5" => Some(if this.active_record_template == "TREN_18" { "paraclinical" } else { "conclusion" }),
                            "6" if this.active_record_template == "TREN_18" => Some("conclusion"),
                            _ => None,
                        };
                        if let Some(t) = tab_opt {
                            this.guard_unsaved(PendingNav::Tab(t.to_string()), window, cx);
                        } else if key == "left" {
                            this.navigate_adjacent_record(-1, window, cx);
                        } else if key == "right" {
                            this.navigate_adjacent_record(1, window, cx);
                        } else if key == "[" {
                            this.cycle_subtab(-1, window, cx);
                        } else if key == "]" {
                            this.cycle_subtab(1, window, cx);
                        }
                    }
                } else if key == "escape" {
                    if this.pending_nav.is_some() {
                        this.pending_nav = None;
                        cx.notify();
                    } else if this.print_modal_action.is_some() {
                        this.print_modal_action = None;
                        cx.notify();
                    } else if this.confirm_lock_modal {
                        this.confirm_lock_modal = false;
                        cx.notify();
                    } else if this.confirm_logout_modal {
                        this.confirm_logout_modal = false;
                        cx.notify();
                    } else if this.open_dropdown.is_some() {
                        this.open_dropdown = None;
                        cx.notify();
                    } else if this.editing_section.is_some() && this.is_dirty {
                        if let Some(id) = this.selected_record_id {
                            this.load_record_into_form(id, window, cx);
                        }
                    }
                }
            }))
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
                                    .max_w(px(220.0))
                                    .truncate()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(PRIMARY_COL))
                                    .child(brand_title),
                            )
                            .child(
                                Button::new("nav-exam")
                                    .label("Khám bệnh")
                                    .small()
                                    .when(active_nav == "exam", |b| b.primary())
                                    .when(active_nav != "exam", |b| b.secondary())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.guard_unsaved(PendingNav::Nav("exam".into()), window, cx);
                                    })),
                            )
                            .when(can_group, |el| {
                                el.child(
                                    Button::new("nav-group")
                                        .label("Khám đoàn & Hợp đồng")
                                        .small()
                                        .when(active_nav == "group", |b| b.primary())
                                        .when(active_nav != "group", |b| b.secondary())
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.guard_unsaved(PendingNav::Nav("group".into()), window, cx);
                                        })),
                                )
                            })
                            .when(can_export, |el| {
                                el.child(
                                    Button::new("nav-export")
                                        .label("Báo cáo & Xuất dữ liệu")
                                        .small()
                                        .when(active_nav == "export", |b| b.primary())
                                        .when(active_nav != "export", |b| b.secondary())
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.guard_unsaved(PendingNav::Nav("export".into()), window, cx);
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
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.guard_unsaved(PendingNav::Nav("admin".into()), window, cx);
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
                                        this.confirm_logout_modal = true;
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
            .when(
                self.pending_nav.is_some()
                    || self.confirm_lock_modal
                    || self.confirm_logout_modal
                    || self.print_modal_action.is_some(),
                |el| {
                    el.child(self.render_ux_modals(window, cx))
                },
            )
            .into_any_element()
    }
}

impl KskGpuiApp {
    fn render_ux_modals(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if let Some((is_docx, is_reception)) = self.print_modal_action {
            let record_tpls = crate::templates::parse_template_list(&self.current_record_templates);
            let ticked = self.print_modal_templates.clone();
            let title_str = match (is_docx, is_reception) {
                (true, true) => "Chọn mẫu xuất Phôi Tiếp Nhận (.docx)",
                (true, false) => "Chọn mẫu xuất Phiếu KSK (.docx)",
                (false, true) => "Chọn mẫu in Phôi Tiếp Nhận (PDF A4)",
                (false, false) => "Chọn mẫu in Giấy KSK (PDF A4)",
            };
            let confirm_lbl = if is_docx {
                format!("Xuất DOCX ({} mẫu)", ticked.len())
            } else {
                format!("In PDF A4 ({} mẫu)", ticked.len())
            };
            return div()
                .id("ux-print-tpl-backdrop")
                .absolute()
                .inset_0()
                .bg(gpui::hsla(0., 0., 0., 0.45))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    card_box()
                        .w(px(500.0))
                        .child(
                            div()
                                .text_base()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(text_main()))
                                .child(title_str),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(text_muted()))
                                .child("Tích chọn một hoặc nhiều mẫu phiếu khám bên dưới để in / xuất file:"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .gap(px(8.0))
                                .child(
                                    Button::new("btn-print-tpl-rec-only")
                                        .label("Chỉ mẫu của hồ sơ")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.print_modal_templates =
                                                crate::templates::parse_template_list(
                                                    &this.current_record_templates,
                                                );
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("btn-print-tpl-select-all")
                                        .label("Chọn tất cả (6 mẫu)")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.print_modal_templates =
                                                crate::templates::ALL_TEMPLATES
                                                    .iter()
                                                    .map(|(c, _, _)| (*c).to_string())
                                                    .collect();
                                            cx.notify();
                                        })),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .children(crate::templates::ALL_TEMPLATES.iter().map(
                                    |(code, short_lbl, full_lbl)| {
                                        let code_s = (*code).to_string();
                                        let is_checked = ticked.contains(&code_s);
                                        let in_rec = record_tpls.contains(&code_s);
                                        div()
                                            .id(gpui::SharedString::from(format!(
                                                "chk-print-tpl-{}",
                                                code
                                            )))
                                            .px(px(12.0))
                                            .py(px(8.0))
                                            .rounded(px(6.0))
                                            .border_1()
                                            .border_color(rgb(if is_checked {
                                                PRIMARY_COL
                                            } else {
                                                border_col()
                                            }))
                                            .bg(rgb(if is_checked {
                                                primary_light()
                                            } else {
                                                bg_subtle()
                                            }))
                                            .cursor_pointer()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .justify_between()
                                            .gap(px(8.0))
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if let Some(pos) = this
                                                    .print_modal_templates
                                                    .iter()
                                                    .position(|x| x == &code_s)
                                                {
                                                    if this.print_modal_templates.len() > 1 {
                                                        this.print_modal_templates.remove(pos);
                                                    }
                                                } else {
                                                    this.print_modal_templates.push(code_s.clone());
                                                }
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.0))
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .text_color(rgb(if is_checked {
                                                                PRIMARY_COL
                                                            } else {
                                                                text_main()
                                                            }))
                                                            .child(format!(
                                                                "{} {}",
                                                                if is_checked { "☑" } else { "☐" },
                                                                short_lbl
                                                            )),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(rgb(text_muted()))
                                                            .child(*full_lbl),
                                                    ),
                                            )
                                            .when(in_rec, |el| {
                                                el.child(badge(
                                                    "Mẫu của hồ sơ",
                                                    SUCCESS_BG,
                                                    SUCCESS_COL,
                                                ))
                                            })
                                    },
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .gap(px(8.0))
                                .pt(px(4.0))
                                .child(
                                    Button::new("btn-print-tpl-cancel")
                                        .label("Hủy")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.print_modal_action = None;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("btn-print-tpl-confirm")
                                        .label(confirm_lbl)
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let chosen = this.print_modal_templates.clone();
                                            this.print_modal_action = None;
                                            if is_docx {
                                                this.execute_export_docx(is_reception, chosen, cx);
                                            } else {
                                                this.execute_print_record_html(
                                                    is_reception,
                                                    chosen,
                                                    cx,
                                                );
                                            }
                                            cx.notify();
                                        })),
                                ),
                        ),
                )
                .into_any_element();
        }

        if let Some(pending) = self.pending_nav.clone() {
            let p_save = pending.clone();
            let p_discard = pending;
            let sec_opt = self.editing_section.clone();
            return div()
                .id("ux-unsaved-backdrop")
                .absolute()
                .inset_0()
                .bg(gpui::hsla(0., 0., 0., 0.45))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    card_box()
                        .w(px(440.0))
                        .child(
                            div()
                                .text_base()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(WARN_COL))
                                .child("Có thay đổi chưa lưu"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_main()))
                                .child("Bạn đã chỉnh sửa dữ liệu ở mục hiện tại nhưng chưa bấm Lưu. Bạn muốn xử lý thế nào?"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .gap(px(8.0))
                                .child(
                                    Button::new("btn-unsaved-stay")
                                        .label("Ở lại")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.pending_nav = None;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("btn-unsaved-discard")
                                        .label("Bỏ thay đổi")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.is_dirty = false;
                                            this.perform_nav(p_discard.clone(), window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("btn-unsaved-save")
                                        .label("Lưu rồi chuyển")
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            if let Some(sec) = sec_opt.as_deref() {
                                                if this.save_section(sec, window, cx) {
                                                    this.perform_nav(p_save.clone(), window, cx);
                                                }
                                            } else {
                                                this.perform_nav(p_save.clone(), window, cx);
                                            }
                                        })),
                                ),
                        ),
                )
                .into_any_element();
        }

        if self.confirm_lock_modal {
            let missing = self.lock_missing_sections.clone();
            let has_missing = !missing.is_empty();
            return div()
                .id("ux-lock-backdrop")
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
                                .text_color(rgb(if has_missing { DANGER_COL } else { PRIMARY_COL }))
                                .child(if has_missing {
                                    "Cảnh báo: Hồ sơ còn mục chưa hoàn tất"
                                } else {
                                    "Xác nhận ký khóa hồ sơ"
                                }),
                        )
                        .when(has_missing, |el| {
                            el.child(
                                div()
                                    .p(px(10.0))
                                    .rounded(px(6.0))
                                    .bg(rgb(DANGER_BG))
                                    .text_xs()
                                    .text_color(rgb(DANGER_COL))
                                    .child(format!("Các mục còn trống / chưa có bác sĩ ký: {}", missing.join(", "))),
                            )
                        })
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_main()))
                                .child("Sau khi ký khóa, hồ sơ sẽ chuyển sang chế độ chỉ xem (chỉ Admin mới mở khóa được). Bạn chắc chắn muốn ký khóa?"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .justify_end()
                                .gap(px(8.0))
                                .child(
                                    Button::new("btn-lock-cancel")
                                        .label("Quay lại kiểm tra")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.confirm_lock_modal = false;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Button::new("btn-lock-confirm")
                                        .label("Xác nhận ký khóa")
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.confirm_lock_modal = false;
                                            this.set_record_status(2, window, cx);
                                        })),
                                ),
                        ),
                )
                .into_any_element();
        }

        div()
            .id("ux-logout-backdrop")
            .absolute()
            .inset_0()
            .bg(gpui::hsla(0., 0., 0., 0.45))
            .flex()
            .items_center()
            .justify_center()
            .child(
                card_box()
                    .w(px(380.0))
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("Xác nhận đăng xuất"),
                    )
                    .when(self.is_dirty, |el| {
                        el.child(
                            div()
                                .p(px(8.0))
                                .rounded(px(6.0))
                                .bg(rgb(WARN_BG))
                                .text_xs()
                                .text_color(rgb(WARN_COL))
                                .child("Lưu ý: Bạn đang có thay đổi chưa lưu!"),
                        )
                    })
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(text_main()))
                            .child("Bạn có chắc muốn đăng xuất khỏi tài khoản hiện tại?"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(
                                Button::new("btn-logout-cancel")
                                    .label("Ở lại")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.confirm_logout_modal = false;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-logout-ok")
                                    .label("Đăng xuất")
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.confirm_logout_modal = false;
                                        this.perform_nav(PendingNav::Logout, window, cx);
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_ngay_vao_and_progress_helpers() {
        assert_eq!(format_ngay_vao_display("202611151230"), "15/11/2026 12:30");
        assert_eq!(format_ngay_vao_display("20261115"), "15/11/2026");
        assert_eq!(format_ngay_vao_display("15/11/2026"), "15/11/2026");

        let mut map = Map::new();
        assert_eq!(section_progress_from_map(&map, "physical"), SectionProgress::Empty);
        map.insert("CHIEU_CAO".into(), Value::String("170".into()));
        assert_eq!(section_progress_from_map(&map, "physical"), SectionProgress::Partial);
        map.insert("BS_THE_LUC".into(), Value::String("BS. A".into()));
        assert_eq!(section_progress_from_map(&map, "physical"), SectionProgress::Done);

        let user_mat = User {
            id: 1,
            username: "mat".into(),
            password_hash: String::new(),
            full_name: "BS Mat".into(),
            role: "bs_mat".into(),
            is_locked: 0,
            created_at: String::new(),
        };
        assert_eq!(next_section_for_user("TREN_18", "clinical", "mat", &user_mat), None);

        let user_admin = User {
            id: 2,
            username: "admin".into(),
            password_hash: String::new(),
            full_name: "Admin".into(),
            role: "admin".into(),
            is_locked: 0,
            created_at: String::new(),
        };
        assert_eq!(
            next_section_for_user("TREN_18", "physical", "", &user_admin),
            Some(("clinical".to_string(), Some("noi".to_string())))
        );
        assert_eq!(
            next_section_for_user("MAU_GIAO", "history", "", &user_admin),
            Some(("physical".to_string(), None))
        );
        assert_eq!(next_exam_tab_in_order("TREN_18", "conclusion", 1), "reception");
        assert_eq!(next_exam_tab_in_order("TREN_18", "reception", -1), "conclusion");
        assert_eq!(next_exam_tab_in_order("MAU_GIAO", "clinical", 1), "conclusion");
    }
}
