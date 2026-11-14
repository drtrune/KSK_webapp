use super::components::map_get;
use crate::config::AppConfig;
use crate::models::{CatalogItem, DbInfo, KskRecordRow, User, VALID_ROLES};
use dioxus::prelude::*;
use serde_json::{Map, Value};
use std::collections::HashSet;

#[component]
pub fn TabExportCsv(
    csv_from_date: Signal<String>,
    csv_to_date: Signal<String>,
    csv_status: Signal<Option<i64>>,
    csv_q: Signal<String>,
    csv_workplace: Signal<String>,
    csv_classification: Signal<String>,
    csv_rows: Signal<Vec<KskRecordRow>>,
    csv_selected_ids: Signal<HashSet<i64>>,
    on_filter: EventHandler<()>,
    on_export: EventHandler<bool>,
) -> Element {
    let rows = csv_rows.read().clone();
    let sel_ids = csv_selected_ids.read().clone();
    let total = rows.len();
    let locked = rows.iter().filter(|r| r.status == 1).count();
    let open = total.saturating_sub(locked);
    let selected_cnt = sel_ids.len();

    let st_val = match *csv_status.read() {
        None => "".to_string(),
        Some(v) => v.to_string(),
    };

    rsx! {
        div { class: "card",
            div { class: "section-header",
                span { class: "section-title", "📤 Trích xuất CSV chuẩn QĐ 3176/QĐ-BYT & QĐ 1804/QĐ-BYT (123 chỉ tiêu)" }
            }

            div { class: "grid-3", style: "margin-bottom: 10px;",
                div { class: "field",
                    label { "Từ ngày" }
                    input { r#type: "text", value: "{csv_from_date}", oninput: move |e| csv_from_date.set(e.value()) }
                }
                div { class: "field",
                    label { "Đến ngày" }
                    input { r#type: "text", value: "{csv_to_date}", oninput: move |e| csv_to_date.set(e.value()) }
                }
                div { class: "field",
                    label { "Trạng thái" }
                    select {
                        value: "{st_val}",
                        onchange: move |e| csv_status.set(e.value().parse::<i64>().ok()),
                        option { value: "", selected: st_val.is_empty(), "Tất cả" }
                        option { value: "0", selected: st_val == "0", "Đang khám" }
                        option { value: "1", selected: st_val == "1", "Đã kết luận" }
                    }
                }
                div { class: "field",
                    label { "Tìm kiếm" }
                    input { r#type: "text", placeholder: "Mã TN / Họ tên / CCCD...", value: "{csv_q}", oninput: move |e| csv_q.set(e.value()) }
                }
                div { class: "field",
                    label { "Nơi làm việc / Đơn vị" }
                    input { r#type: "text", placeholder: "Lọc đơn vị...", value: "{csv_workplace}", oninput: move |e| csv_workplace.set(e.value()) }
                }
                div { class: "field",
                    label { "Phân loại sức khỏe" }
                    select {
                        value: "{csv_classification}",
                        onchange: move |e| csv_classification.set(e.value()),
                        option { value: "", "Tất cả" }
                        for p in ["1", "2", "3", "4", "5"] {
                            option { value: "{p}", selected: *csv_classification.read() == p, "Loại {p}" }
                        }
                    }
                }
            }

            div { style: "display: flex; gap: 8px;",
                button { class: "btn btn-blue", onclick: move |_| on_filter.call(()), "🔍 Lọc danh sách" }
                button { class: "btn btn-primary", onclick: move |_| on_export.call(false), "📥 Xuất CSV (Tất cả kết quả lọc)" }
                if selected_cnt > 0 {
                    button { class: "btn btn-success", onclick: move |_| on_export.call(true), "📥 Xuất CSV ({selected_cnt} dòng đã chọn)" }
                }
            }
        }

        div { class: "grid-4",
            div { class: "card",
                div { style: "font-size: 12px; color: #64748b;", "Tổng hồ sơ" }
                div { style: "font-size: 22px; font-weight: 700; color: #0284c7;", "{total}" }
            }
            div { class: "card",
                div { style: "font-size: 12px; color: #64748b;", "Đã kết luận" }
                div { style: "font-size: 22px; font-weight: 700; color: #16a34a;", "{locked}" }
            }
            div { class: "card",
                div { style: "font-size: 12px; color: #64748b;", "Đang khám" }
                div { style: "font-size: 22px; font-weight: 700; color: #d97706;", "{open}" }
            }
            div { class: "card",
                div { style: "font-size: 12px; color: #64748b;", "Đang chọn" }
                div { style: "font-size: 22px; font-weight: 700; color: #0d9488;", "{selected_cnt}" }
            }
        }

        div { class: "card",
            div { style: "display: flex; gap: 8px; margin-bottom: 10px;",
                button {
                    class: "btn",
                    onclick: move |_| {
                        let all: HashSet<i64> = csv_rows.read().iter().map(|r| r.id).collect();
                        csv_selected_ids.set(all);
                    },
                    "Chọn tất cả"
                }
                button {
                    class: "btn",
                    onclick: move |_| csv_selected_ids.write().clear(),
                    "Bỏ chọn"
                }
            }
            div { class: "table-wrap",
                table {
                    thead {
                        tr {
                            th { "Chọn" }
                            th { "Mã TN" }
                            th { "Họ và tên" }
                            th { "Ngày sinh" }
                            th { "Nơi làm việc" }
                            th { "Phân loại" }
                            th { "Trạng thái" }
                        }
                    }
                    tbody {
                        for r in rows {
                            {
                                let rid = r.id;
                                let checked = sel_ids.contains(&rid);
                                let map: Map<String, Value> = serde_json::from_str(&r.payload_json).unwrap_or_default();
                                let wp = map_get(&map, "NOI_LAM_VIEC_HOC_TAP");
                                let pl = map_get(&map, "PHAN_LOAI_SK");
                                let pl_txt = if pl.is_empty() { "-".to_string() } else { format!("Loại {}", pl) };
                                rsx! {
                                    tr { key: "{rid}",
                                        td {
                                            input {
                                                r#type: "checkbox",
                                                style: "width: auto;",
                                                checked: checked,
                                                onchange: move |e| {
                                                    if e.checked() {
                                                        csv_selected_ids.write().insert(rid);
                                                    } else {
                                                        csv_selected_ids.write().remove(&rid);
                                                    }
                                                }
                                            }
                                        }
                                        td { "{r.ma_tiep_nhan}" }
                                        td { style: "font-weight: 600;", "{r.ho_ten}" }
                                        td { "{r.ngay_sinh}" }
                                        td { "{wp}" }
                                        td { "{pl_txt}" }
                                        td {
                                            if r.status == 1 {
                                                span { class: "badge badge-green", "Đã kết luận" }
                                            } else {
                                                span { class: "badge badge-amber", "Đang khám" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdminSubTab {
    Config,
    Users,
    Catalogs,
}

#[component]
pub fn TabAdmin(
    user: Option<User>,
    cfg: Signal<AppConfig>,
    db_info: Signal<Option<DbInfo>>,
    publish_changelog: Signal<String>,
    users_list: Signal<Vec<User>>,
    new_user_username: Signal<String>,
    new_user_password: Signal<String>,
    new_user_fullname: Signal<String>,
    new_user_roles: Signal<Vec<String>>,
    catalog_type_filter: Signal<String>,
    catalog_search: Signal<String>,
    catalog_rows: Signal<Vec<CatalogItem>>,
    on_switch_db: EventHandler<String>,
    on_save_cfg: EventHandler<()>,
    on_publish_update: EventHandler<()>,
    on_create_user: EventHandler<()>,
    on_toggle_lock_user: EventHandler<i64>,
    on_refresh_users: EventHandler<()>,
    on_refresh_catalogs: EventHandler<()>,
    on_set_default_cat: EventHandler<i64>,
    on_toggle_active_cat: EventHandler<i64>,
    on_reload_asset_cats: EventHandler<()>,
) -> Element {
    let is_admin = user.as_ref().map(|u| u.has_role("admin")).unwrap_or(false);
    if !is_admin {
        return rsx! { div { class: "card", style: "color: #dc2626;", "⛔ Chỉ Quản trị viên (Admin) mới có quyền truy cập mục này." } };
    }

    let mut subtab = use_signal(|| AdminSubTab::Config);
    let cur_sub = *subtab.read();
    let cfg_val = cfg.read().clone();
    let db_inf = db_info.read().clone();

    rsx! {
        div { style: "display: flex; gap: 8px;",
            button {
                class: if cur_sub == AdminSubTab::Config { "btn btn-primary" } else { "btn" },
                onclick: move |_| subtab.set(AdminSubTab::Config),
                "⚙️ Cấu hình Config / DB / Update"
            }
            button {
                class: if cur_sub == AdminSubTab::Users { "btn btn-primary" } else { "btn" },
                onclick: move |_| subtab.set(AdminSubTab::Users),
                "👥 Quản lý Tài khoản"
            }
            button {
                class: if cur_sub == AdminSubTab::Catalogs { "btn btn-primary" } else { "btn" },
                onclick: move |_| {
                    subtab.set(AdminSubTab::Catalogs);
                    on_refresh_catalogs.call(());
                },
                "📚 Quản lý Danh mục"
            }
        }

        if cur_sub == AdminSubTab::Config {
            div { class: "card",
                div { class: "section-header",
                    span { class: "section-title", "🗄️ Cơ sở dữ liệu SQLite (Ổ đĩa chung LAN/SMB) & Tự động cập nhật" }
                }

                div { class: "field", style: "margin-bottom: 10px;",
                    label { "Đường dẫn Database (.db)" }
                    div { style: "display: flex; gap: 8px;",
                        input {
                            r#type: "text",
                            value: "{cfg_val.database_path}",
                            oninput: move |e| cfg.write().database_path = e.value(),
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                if let Some(p) = rfd::FileDialog::new().add_filter("SQLite DB", &["db", "sqlite"]).save_file() {
                                    cfg.write().database_path = p.display().to_string();
                                }
                            },
                            "📂 Chọn file DB..."
                        }
                        button {
                            class: "btn btn-blue",
                            onclick: move |_| {
                                let p = cfg.read().database_path.clone();
                                on_switch_db.call(p);
                            },
                            "🔄 Kết nối DB ngay"
                        }
                    }
                }

                if let Some(info) = db_inf {
                    div { style: "font-size: 12px; color: #475569; margin-bottom: 12px; background: #f8fafc; padding: 6px 10px; border-radius: 6px;",
                        "Đang kết nối: {info.current_db_path} | {info.records_count} hồ sơ | {info.users_count} tài khoản | {info.catalogs_count} danh mục | {info.file_size_bytes / 1024} KB"
                    }
                }

                div { class: "field", style: "margin-bottom: 10px;",
                    label { "Thư mục Update trên ổ đĩa chung (LAN Share)" }
                    div { style: "display: flex; gap: 8px; align-items: center;",
                        input {
                            r#type: "text",
                            value: "{cfg_val.update_dir_path}",
                            placeholder: "\\\\Server\\Share\\KSK\\Update",
                            oninput: move |e| cfg.write().update_dir_path = e.value(),
                        }
                        button {
                            class: "btn",
                            onclick: move |_| {
                                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                    cfg.write().update_dir_path = p.display().to_string();
                                }
                            },
                            "📁 Chọn thư mục..."
                        }
                        label { class: "checkbox-item", style: "white-space: nowrap;",
                            input {
                                r#type: "checkbox",
                                checked: cfg_val.auto_check_update,
                                onchange: move |e| cfg.write().auto_check_update = e.checked(),
                            }
                            span { "Tự kiểm tra khi mở app" }
                        }
                    }
                }

                div { style: "display: flex; gap: 8px; align-items: center;",
                    input {
                        r#type: "text",
                        placeholder: "Ghi chú phiên bản phát hành...",
                        value: "{publish_changelog}",
                        oninput: move |e| publish_changelog.set(e.value()),
                    }
                    button {
                        class: "btn btn-amber",
                        style: "white-space: nowrap;",
                        onclick: move |_| on_publish_update.call(()),
                        "🚀 Phát hành bản hiện tại lên Share"
                    }
                }
            }

            div { class: "card",
                div { class: "section-header",
                    span { class: "section-title", "🏥 Thông tin CSKCB & Tích hợp AHK v2 HIS Bridge" }
                }
                div { class: "grid-2", style: "margin-bottom: 10px;",
                    div { class: "field",
                        label { "Mã CSKCB" }
                        input { r#type: "text", value: "{cfg_val.ma_cskcb}", oninput: move |e| cfg.write().ma_cskcb = e.value() }
                    }
                    div { class: "field",
                        label { "Mã GTIN CSKCB" }
                        input { r#type: "text", value: "{cfg_val.ma_gtin_cskcb}", oninput: move |e| cfg.write().ma_gtin_cskcb = e.value() }
                    }
                    div { class: "field",
                        label { "Tên CSKCB" }
                        input { r#type: "text", value: "{cfg_val.ten_cskcb}", oninput: move |e| cfg.write().ten_cskcb = e.value() }
                    }
                    div { class: "field",
                        label { "Địa danh (in phiếu)" }
                        input { r#type: "text", value: "{cfg_val.dia_danh}", oninput: move |e| cfg.write().dia_danh = e.value() }
                    }
                    div { class: "field",
                        label { "API Key (AHK Bridge)" }
                        input { r#type: "text", value: "{cfg_val.his_api_key}", oninput: move |e| cfg.write().his_api_key = e.value() }
                    }
                    div { class: "field",
                        label { "Cổng Local IPC (127.0.0.1)" }
                        input {
                            r#type: "text",
                            value: "{cfg_val.local_ipc_port}",
                            oninput: move |e| {
                                if let Ok(p) = e.value().parse::<u16>() {
                                    cfg.write().local_ipc_port = p;
                                }
                            }
                        }
                    }
                }
                button { class: "btn btn-primary", onclick: move |_| on_save_cfg.call(()), "💾 Lưu cấu hình vào config.toml" }
            }
        } else if cur_sub == AdminSubTab::Users {
            div { class: "card",
                div { class: "section-header",
                    span { class: "section-title", "➕ Tạo tài khoản mới" }
                }
                div { class: "grid-3", style: "margin-bottom: 10px;",
                    div { class: "field",
                        label { "Tên đăng nhập" }
                        input { r#type: "text", value: "{new_user_username}", oninput: move |e| new_user_username.set(e.value()) }
                    }
                    div { class: "field",
                        label { "Mật khẩu" }
                        input { r#type: "password", value: "{new_user_password}", oninput: move |e| new_user_password.set(e.value()) }
                    }
                    div { class: "field",
                        label { "Họ tên chức danh" }
                        input { r#type: "text", value: "{new_user_fullname}", oninput: move |e| new_user_fullname.set(e.value()) }
                    }
                }
                div { style: "display: flex; flex-wrap: wrap; gap: 10px; margin-bottom: 10px;",
                    for r in VALID_ROLES {
                        {
                            let r_str = (*r).to_string();
                            let checked = new_user_roles.read().contains(&r_str);
                            rsx! {
                                label { class: "checkbox-item",
                                    input {
                                        r#type: "checkbox",
                                        checked: checked,
                                        onchange: move |e| {
                                            if e.checked() {
                                                if !new_user_roles.read().contains(&r_str) {
                                                    new_user_roles.write().push(r_str.clone());
                                                }
                                            } else {
                                                new_user_roles.write().retain(|x| x != &r_str);
                                            }
                                        }
                                    }
                                    span { "{r}" }
                                }
                            }
                        }
                    }
                }
                button { class: "btn btn-primary", onclick: move |_| on_create_user.call(()), "Tạo tài khoản" }
            }

            div { class: "card",
                div { class: "section-header",
                    span { class: "section-title", "Danh sách tài khoản" }
                    button { class: "btn", onclick: move |_| on_refresh_users.call(()), "🔄 Tải lại" }
                }
                div { class: "table-wrap",
                    table {
                        thead {
                            tr {
                                th { "Username" }
                                th { "Họ tên" }
                                th { "Vai trò" }
                                th { "Trạng thái" }
                                th { "Thao tác" }
                            }
                        }
                        tbody {
                            for u in users_list.read().clone() {
                                {
                                    let uid = u.id;
                                    let lock_lbl = if u.is_locked == 1 { "Mở khóa" } else { "Khóa" };
                                    rsx! {
                                        tr { key: "{uid}",
                                            td { "{u.username}" }
                                            td { style: "font-weight: 600;", "{u.full_name}" }
                                            td { "{u.role}" }
                                            td {
                                                if u.is_locked == 1 {
                                                    span { class: "badge badge-red", "🔒 Đã khóa" }
                                                } else {
                                                    span { class: "badge badge-green", "✅ Hoạt động" }
                                                }
                                            }
                                            td {
                                                button { class: "btn", onclick: move |_| on_toggle_lock_user.call(uid), "{lock_lbl}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            div { class: "card",
                div { class: "section-header",
                    span { class: "section-title", "📚 Quản lý Danh mục" }
                    button { class: "btn btn-amber", onclick: move |_| on_reload_asset_cats.call(()), "♻️ Nạp lại danh mục chuẩn" }
                }
                div { style: "display: flex; gap: 8px; margin-bottom: 10px;",
                    select {
                        style: "width: 220px;",
                        value: "{catalog_type_filter}",
                        onchange: move |e| {
                            catalog_type_filter.set(e.value());
                            on_refresh_catalogs.call(());
                        },
                        for t in ["DAN_TOC", "NGHE_NGHIEP", "TINH_TP", "XA_PHUONG", "DOI_TUONG", "NGUON_CHI_TRA", "NOI_CAP", "ICD10"] {
                            option { value: "{t}", selected: *catalog_type_filter.read() == t, "{t}" }
                        }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Tìm mã / tên danh mục...",
                        value: "{catalog_search}",
                        oninput: move |e| {
                            catalog_search.set(e.value());
                            on_refresh_catalogs.call(());
                        }
                    }
                }
                div { class: "table-wrap",
                    table {
                        thead {
                            tr {
                                th { "Mã" }
                                th { "Tên danh mục" }
                                th { "Mã cha" }
                                th { "Mặc định" }
                                th { "Trạng thái" }
                            }
                        }
                        tbody {
                            for c in catalog_rows.read().clone() {
                                {
                                    let cid = c.id;
                                    let def_lbl = if c.is_default == 1 { "⭐ Mặc định" } else { "Đặt mặc định" };
                                    let act_lbl = if c.is_active == 1 { "✅ Đang dùng" } else { "⛔ Ẩn" };
                                    rsx! {
                                        tr { key: "{cid}",
                                            td { "{c.code}" }
                                            td { "{c.name}" }
                                            td { "{c.parent_code}" }
                                            td { button { class: "btn", onclick: move |_| on_set_default_cat.call(cid), "{def_lbl}" } }
                                            td { button { class: "btn", onclick: move |_| on_toggle_active_cat.call(cid), "{act_lbl}" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
