use super::components::{
    form_set_key, map_get, CatalogSelect, IcdMultiField, SectionHeader, SelectField,
    TextAreaField, TextField, PL_OPTIONS, YES_NO_OPTIONS,
};
use crate::models::{phan_loai_config, CatalogItem, User};
use dioxus::prelude::*;
use serde_json::{Map, Value};
use std::collections::HashMap;

const TSBT_ITEMS: &[(&str, &str)] = &[
    ("TSBT_BENH_TRONG_5_NAM_QUA", "Bệnh trong 5 năm qua"),
    ("TSBT_BENH_THAN_KINH", "Bệnh thần kinh"),
    ("TSBT_BENH_MAT", "Bệnh mắt / giảm thị lực"),
    ("TSBT_BENH_TAI", "Bệnh tai / thính lực"),
    ("TSBT_BENH_TIM", "Bệnh tim mạch"),
    ("TSBT_PHAU_THUAT_TIM", "Phẫu thuật tim / can thiệp mạch"),
    ("TSBT_TANG_HUYET_AP", "Tăng huyết áp"),
    ("TSBT_KHO_THO", "Khó thở / hen phế quản"),
    ("TSBT_BENH_PHOI", "Bệnh phổi / lao phổi"),
    ("TSBT_BENH_THAN", "Bệnh thận / tiết niệu"),
    ("TSBT_NGHIEN_RUOU", "Nghiện rượu / bia"),
    ("TSBT_DAI_THAO_DUONG", "Đái tháo đường"),
    ("TSBT_BENH_TAM_THAN", "Bệnh tâm thần"),
    ("TSBT_MAT_Y_THUC", "Mất ý thức / động kinh"),
    ("TSBT_NGAT", "Ngất / xỉu"),
    ("TSBT_BENH_TIEU_HOA", "Bệnh tiêu hóa / gan mật"),
    ("TSBT_ROI_LOAN_GIAC_NGU", "Rối loạn giấc ngủ"),
    ("TSBT_TAI_BIEN", "Tai biến mạch máu não"),
    ("TSBT_BENH_COT_SONG", "Bệnh cột sống / xương khớp"),
    ("TSBT_RUOU_THUONG_XUYEN", "Uống rượu thường xuyên"),
    ("TSBT_MA_TUY", "Sử dụng chất gây nghiện / ma túy"),
];

fn pl_opts() -> Vec<(String, String)> {
    PL_OPTIONS
        .iter()
        .map(|(c, n)| ((*c).to_string(), (*n).to_string()))
        .collect()
}

fn yn_opts() -> Vec<(String, String)> {
    YES_NO_OPTIONS
        .iter()
        .map(|(c, n)| ((*c).to_string(), (*n).to_string()))
        .collect()
}

fn check_section_unlocked(
    user: &Option<User>,
    selected_record_id: Option<i64>,
    selected_record_status: i64,
    editing_sections: &HashMap<String, bool>,
    section: &str,
    role: &str,
) -> (bool, bool) {
    let can_role = user
        .as_ref()
        .map(|u| u.has_role("admin") || u.has_role(role))
        .unwrap_or(false);
    if selected_record_status == 1 || !can_role {
        return (can_role, false);
    }
    if selected_record_id.is_none() {
        return (can_role, section == "reception");
    }
    let is_ed = *editing_sections.get(section).unwrap_or(&false);
    (can_role, is_ed)
}

#[component]
pub fn TabReception(
    user: Option<User>,
    selected_record_id: Signal<Option<i64>>,
    selected_record_status: Signal<i64>,
    form: Signal<Map<String, Value>>,
    editing_sections: Signal<HashMap<String, bool>>,
    catalogs_cache: Signal<HashMap<String, Vec<CatalogItem>>>,
    on_save_section: EventHandler<String>,
    on_cancel_section: EventHandler<String>,
    on_fetch_his: EventHandler<()>,
    on_export_docx: EventHandler<(i64, bool)>,
) -> Element {
    let rec_id = *selected_record_id.read();
    let rec_status = *selected_record_status.read();
    let eds = editing_sections.read().clone();
    let (can_role, enabled) =
        check_section_unlocked(&user, rec_id, rec_status, &eds, "reception", "reception");
    let is_editing = *eds.get("reception").unwrap_or(&false);

    let cats = catalogs_cache.read();
    let dan_toc = cats.get("DAN_TOC").cloned().unwrap_or_default();
    let nghe_nghiep = cats.get("NGHE_NGHIEP").cloned().unwrap_or_default();
    let tinh_tp = cats.get("TINH_TP").cloned().unwrap_or_default();
    let xa_phuong = cats.get("XA_PHUONG").cloned().unwrap_or_default();
    let doi_tuong = cats.get("DOI_TUONG").cloned().unwrap_or_default();
    let nguon_chi_tra = cats.get("NGUON_CHI_TRA").cloned().unwrap_or_default();
    drop(cats);

    let selected_tinh = map_get(&form.read(), "MATINH_CU_TRU");

    rsx! {
        div { class: "card",
            SectionHeader {
                title: "1. Tiếp nhận & Thông tin hành chính".to_string(),
                can_edit_role: can_role,
                is_new_record: rec_id.is_none(),
                is_locked_record: rec_status == 1,
                is_editing: is_editing,
                on_edit: move |_| { editing_sections.write().insert("reception".to_string(), true); },
                on_save: move |_| on_save_section.call("reception".to_string()),
                on_cancel: move |_| on_cancel_section.call("reception".to_string()),
            }

            div { style: "display: flex; gap: 8px; margin-bottom: 12px;",
                button {
                    class: "btn btn-blue",
                    disabled: !enabled,
                    onclick: move |_| on_fetch_his.call(()),
                    "🌐 Lấy từ HIS (API)"
                }
                if let Some(id) = rec_id {
                    button {
                        class: "btn",
                        onclick: move |_| on_export_docx.call((id, true)),
                        "🖨️ In Phiếu Tiếp Nhận (DOCX)"
                    }
                }
            }

            div { class: "grid-3",
                TextField { label: "Mã tiếp nhận (*)".to_string(), field_key: "MA_TIEP_NHAN".to_string(), form: form, enabled: enabled, hint: "VD: TN.2611.0009394".to_string() }
                TextField { label: "Mã y tế (HIS)".to_string(), field_key: "MA_Y_TE".to_string(), form: form, enabled: enabled, hint: "VD: 18086929".to_string() }
                TextField { label: "Ngày vào (dd/MM/yyyy)".to_string(), field_key: "NGAY_VAO".to_string(), form: form, enabled: enabled, hint: "dd/MM/yyyy".to_string() }

                TextField { label: "Họ và tên (*)".to_string(), field_key: "HO_TEN".to_string(), form: form, enabled: enabled, hint: "NGUYỄN VĂN A".to_string() }
                SelectField {
                    label: "Giới tính".to_string(),
                    field_key: "GIOI_TINH".to_string(),
                    form: form,
                    options: vec![("1".to_string(), "Nam".to_string()), ("2".to_string(), "Nữ".to_string())],
                    enabled: enabled,
                }
                TextField { label: "Ngày sinh (dd/MM/yyyy)".to_string(), field_key: "NGAY_SINH".to_string(), form: form, enabled: enabled, hint: "12/03/1967".to_string() }

                TextField { label: "Số CCCD / CMND".to_string(), field_key: "SO_CCCD".to_string(), form: form, enabled: enabled, hint: "052067005614".to_string() }
                TextField { label: "Ngày cấp CCCD".to_string(), field_key: "NGAYCAP_CCCD".to_string(), form: form, enabled: enabled, hint: "dd/MM/yyyy".to_string() }
                TextField { label: "Nơi cấp CCCD".to_string(), field_key: "NOICAP_CCCD".to_string(), form: form, enabled: enabled, hint: "Cục Cảnh sát QLHCVTTXH".to_string() }

                CatalogSelect { label: "Dân tộc".to_string(), field_key: "MA_DAN_TOC".to_string(), form: form, items: dan_toc, enabled: enabled }
                CatalogSelect { label: "Nghề nghiệp".to_string(), field_key: "MA_NGHE_NGHIEP".to_string(), form: form, items: nghe_nghiep, enabled: enabled }
                TextField { label: "Số điện thoại".to_string(), field_key: "DIEN_THOAI".to_string(), form: form, enabled: enabled, hint: "0968459138".to_string() }

                CatalogSelect { label: "Tỉnh / Thành phố cư trú".to_string(), field_key: "MATINH_CU_TRU".to_string(), form: form, items: tinh_tp, enabled: enabled }
                CatalogSelect { label: "Xã / Phường cư trú".to_string(), field_key: "MAXA_CU_TRU".to_string(), form: form, items: xa_phuong, parent_filter: selected_tinh, enabled: enabled }
                TextField { label: "Địa chỉ chi tiết (Số nhà/Thôn)".to_string(), field_key: "DIA_CHI".to_string(), form: form, enabled: enabled, hint: "Khu Phố 3...".to_string() }

                TextField { label: "Nơi làm việc / Học tập".to_string(), field_key: "NOI_LAM_VIEC_HOC_TAP".to_string(), form: form, enabled: enabled, hint: "Công ty / Đơn vị...".to_string() }
                CatalogSelect { label: "Đối tượng KSK".to_string(), field_key: "DOI_TUONG".to_string(), form: form, items: doi_tuong, enabled: enabled }
                CatalogSelect { label: "Nguồn chi trả".to_string(), field_key: "NGUON_CHI_TRA".to_string(), form: form, items: nguon_chi_tra, enabled: enabled }
            }

            div { class: "grid-2", style: "margin-top: 10px;",
                TextField { label: "Nhóm máu (nếu biết)".to_string(), field_key: "NHOM_MAU".to_string(), form: form, enabled: enabled, hint: "O / A / B / AB".to_string() }
                TextField { label: "Lý do khám / vào viện".to_string(), field_key: "LY_DO_VV".to_string(), form: form, enabled: enabled, hint: "Khám sức khỏe định kỳ / Lái xe...".to_string() }
            }
        }
    }
}

#[component]
pub fn TabHistory(
    user: Option<User>,
    selected_record_id: Signal<Option<i64>>,
    selected_record_status: Signal<i64>,
    form: Signal<Map<String, Value>>,
    editing_sections: Signal<HashMap<String, bool>>,
    catalogs_cache: Signal<HashMap<String, Vec<CatalogItem>>>,
    on_save_section: EventHandler<String>,
    on_cancel_section: EventHandler<String>,
) -> Element {
    let rec_id = *selected_record_id.read();
    if rec_id.is_none() {
        return rsx! { div { class: "card", "Vui lòng chọn hoặc tiếp nhận một hồ sơ bên danh sách trái." } };
    }
    let rec_status = *selected_record_status.read();
    let eds = editing_sections.read().clone();
    let (can_role, enabled) =
        check_section_unlocked(&user, rec_id, rec_status, &eds, "history", "tiensu");
    let is_editing = *eds.get("history").unwrap_or(&false);
    let his_raw = map_get(&form.read(), "HIS_TIEN_SU_RAW");
    let icd_items = catalogs_cache.read().get("ICD10").cloned().unwrap_or_default();

    rsx! {
        div { class: "card",
            SectionHeader {
                title: "2. Tiền sử gia đình & Tiền sử bản thân (21 mục)".to_string(),
                can_edit_role: can_role,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: is_editing,
                on_edit: move |_| { editing_sections.write().insert("history".to_string(), true); },
                on_save: move |_| on_save_section.call("history".to_string()),
                on_cancel: move |_| on_cancel_section.call("history".to_string()),
            }

            if !his_raw.is_empty() {
                div { style: "background: #fef3c7; border: 1px solid #f59e0b; padding: 8px 12px; border-radius: 6px; margin-bottom: 10px; color: #92400e; font-size: 12.5px;",
                    "📌 Ghi chú tiền sử từ HIS: {his_raw}"
                }
            }

            div { style: "font-weight: 700; color: #0284c7; margin-bottom: 6px;", "A. Tiền sử gia đình" }
            div { class: "grid-2", style: "margin-bottom: 12px;",
                SelectField { label: "Gia đình có người mắc bệnh?".to_string(), field_key: "TSGD_MAC_BENH".to_string(), form: form, options: yn_opts(), enabled: enabled }
                IcdMultiField { label: "Mã bệnh gia đình (ICD-10)".to_string(), field_key: "TSGD_MA_BENH".to_string(), form: form, icd_items: icd_items.clone(), enabled: enabled }
            }

            div { style: "font-weight: 700; color: #0284c7; margin-bottom: 8px;", "B. Tiền sử bản thân (21 câu hỏi sàng lọc)" }
            div { class: "checkbox-grid", style: "margin-bottom: 12px;",
                for (key, label) in TSBT_ITEMS {
                    {
                        let k = (*key).to_string();
                        let checked = map_get(&form.read(), &k) == "1";
                        rsx! {
                            label { class: "checkbox-item",
                                input {
                                    r#type: "checkbox",
                                    checked: checked,
                                    disabled: !enabled,
                                    onchange: move |evt| {
                                        form_set_key(&mut form, &k, if evt.checked() { "1".into() } else { "0".into() });
                                    }
                                }
                                span { "{label}" }
                            }
                        }
                    }
                }
            }

            div { class: "grid-2", style: "margin-bottom: 8px;",
                SelectField { label: "Bệnh khác (TSBT)".to_string(), field_key: "TSBT_BENH_KHAC".to_string(), form: form, options: yn_opts(), enabled: enabled }
                IcdMultiField { label: "Mã bệnh khác (ICD-10)".to_string(), field_key: "TSBT_MA_BENH_KHAC".to_string(), form: form, icd_items: icd_items.clone(), enabled: enabled }
            }
            div { class: "grid-3", style: "margin-bottom: 8px;",
                SelectField { label: "Đang điều trị bệnh?".to_string(), field_key: "TSBT_DANG_DIEU_TRI_BENH".to_string(), form: form, options: yn_opts(), enabled: enabled }
                IcdMultiField { label: "Mã bệnh đang điều trị".to_string(), field_key: "TSBT_MA_BENH".to_string(), form: form, icd_items: icd_items.clone(), enabled: enabled }
                TextField { label: "Tên thuốc & liều lượng".to_string(), field_key: "TSBT_TEN_THUOC_LIEU_LUONG".to_string(), form: form, enabled: enabled }
            }
            div { class: "grid-3",
                SelectField { label: "Tiền sử thai sản (Nữ)".to_string(), field_key: "TSBT_THAI_SAN".to_string(), form: form, options: yn_opts(), enabled: enabled }
                IcdMultiField { label: "Mã bệnh thai sản".to_string(), field_key: "TSBT_MA_BENH_THAI_SAN".to_string(), form: form, icd_items: icd_items, enabled: enabled }
                TextField { label: "Thuốc thai sản".to_string(), field_key: "TSBT_TEN_THUOC_THAI_SAN".to_string(), form: form, enabled: enabled }
            }
        }
    }
}

#[component]
pub fn TabPhysical(
    user: Option<User>,
    selected_record_id: Signal<Option<i64>>,
    selected_record_status: Signal<i64>,
    form: Signal<Map<String, Value>>,
    editing_sections: Signal<HashMap<String, bool>>,
    on_save_section: EventHandler<String>,
    on_cancel_section: EventHandler<String>,
) -> Element {
    let rec_id = *selected_record_id.read();
    if rec_id.is_none() {
        return rsx! { div { class: "card", "Vui lòng chọn một hồ sơ bên danh sách trái." } };
    }
    let rec_status = *selected_record_status.read();
    let eds = editing_sections.read().clone();
    let (can_role, enabled) =
        check_section_unlocked(&user, rec_id, rec_status, &eds, "physical", "bs_theluc");
    let is_editing = *eds.get("physical").unwrap_or(&false);

    // Tự động tính BMI
    let h: f64 = map_get(&form.read(), "CHIEU_CAO").replace(',', ".").parse().unwrap_or(0.0);
    let w: f64 = map_get(&form.read(), "CAN_NANG").replace(',', ".").parse().unwrap_or(0.0);
    if h > 50.0 && w > 10.0 {
        let bmi = format!("{:.1}", w / ((h / 100.0) * (h / 100.0)));
        if map_get(&form.read(), "CHI_SO_BMI") != bmi {
            form_set_key(&mut form, "CHI_SO_BMI", bmi);
        }
    }

    rsx! {
        div { class: "card",
            SectionHeader {
                title: "3. Khám thể lực & Sinh hiệu".to_string(),
                can_edit_role: can_role,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: is_editing,
                on_edit: move |_| { editing_sections.write().insert("physical".to_string(), true); },
                on_save: move |_| on_save_section.call("physical".to_string()),
                on_cancel: move |_| on_cancel_section.call("physical".to_string()),
            }

            div { class: "grid-3",
                TextField { label: "Chiều cao (cm)".to_string(), field_key: "CHIEU_CAO".to_string(), form: form, enabled: enabled, hint: "VD: 168".to_string() }
                TextField { label: "Cân nặng (kg)".to_string(), field_key: "CAN_NANG".to_string(), form: form, enabled: enabled, hint: "VD: 62".to_string() }
                TextField { label: "Chỉ số BMI (Tự động)".to_string(), field_key: "CHI_SO_BMI".to_string(), form: form, enabled: enabled }
                TextField { label: "Mạch (lần/phút)".to_string(), field_key: "MACH".to_string(), form: form, enabled: enabled, hint: "VD: 76".to_string() }
                TextField { label: "Huyết áp (mmHg)".to_string(), field_key: "HUYET_AP".to_string(), form: form, enabled: enabled, hint: "VD: 120/80".to_string() }
                SelectField { label: "Phân loại thể lực".to_string(), field_key: "KHAM_THE_LUC_PL".to_string(), form: form, options: pl_opts(), enabled: enabled }
            }
        }
    }
}

#[component]
pub fn TabClinical(
    user: Option<User>,
    selected_record_id: Signal<Option<i64>>,
    selected_record_status: Signal<i64>,
    form: Signal<Map<String, Value>>,
    editing_sections: Signal<HashMap<String, bool>>,
    clinical_full_mode: Signal<bool>,
    on_save_section: EventHandler<String>,
    on_cancel_section: EventHandler<String>,
) -> Element {
    let rec_id = *selected_record_id.read();
    if rec_id.is_none() {
        return rsx! { div { class: "card", "Vui lòng chọn một hồ sơ bên danh sách trái." } };
    }
    let rec_status = *selected_record_status.read();
    let eds = editing_sections.read().clone();

    let noi_rows: &[(&str, &str, &str, &str)] = &[
        ("Tuần hoàn", "NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "CKDT_NOI_KHOA_TUAN_HOAN"),
        ("Hô hấp", "NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL", "CKDT_NOI_KHOA_HO_HAP"),
        ("Tiêu hóa", "NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "CKDT_NOI_KHOA_TIEU_HOA"),
        ("Thận - Tiết niệu - SD", "NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TN_SD_PL", "CKDT_NOI_KHOA_THAN_TN_SD"),
        ("Nội tiết", "NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "CKDT_NOI_KHOA_NOI_TIET"),
        ("Cơ - Xương - Khớp", "NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL", "CKDT_NOI_KHOA_CO_XUONG_KHOP"),
        ("Thần kinh", "NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "CKDT_NOI_KHOA_THAN_KINH"),
        ("Tâm thần", "NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL", "CKDT_NOI_KHOA_TAM_THAN"),
    ];

    let (can_noi, en_noi) = check_section_unlocked(&user, rec_id, rec_status, &eds, "noi", "bs_noi");
    let (can_ngoai, en_ngoai) = check_section_unlocked(&user, rec_id, rec_status, &eds, "ngoai", "bs_ngoai");
    let (can_dl, en_dl) = check_section_unlocked(&user, rec_id, rec_status, &eds, "dalieu", "bs_dalieu");
    let (can_san, en_san) = check_section_unlocked(&user, rec_id, rec_status, &eds, "san", "bs_san");
    let (can_mat, en_mat) = check_section_unlocked(&user, rec_id, rec_status, &eds, "mat", "bs_mat");
    let (can_tmh, en_tmh) = check_section_unlocked(&user, rec_id, rec_status, &eds, "tmh", "bs_tmh");
    let (can_rhm, en_rhm) = check_section_unlocked(&user, rec_id, rec_status, &eds, "rhm", "bs_rhm");

    let full_tree = *clinical_full_mode.read();
    let cfg = phan_loai_config();

    rsx! {
        div { class: "card", style: "padding: 10px 16px;",
            label { class: "checkbox-item",
                input {
                    r#type: "checkbox",
                    checked: full_tree,
                    onchange: move |e| clinical_full_mode.set(e.checked()),
                }
                span { style: "font-weight: 600;", "Hiển thị Cây phân loại bệnh QĐ 3176 (15 chuyên khoa / 110 bệnh)" }
            }
            if full_tree {
                div { style: "margin-top: 10px; max-height: 240px; overflow-y: auto; border: 1px solid #e2e8f0; border-radius: 6px; padding: 8px;",
                    for sp in &cfg.specialties {
                        {
                            let diseases: Vec<_> = cfg.diseases.iter().filter(|d| d.specialty_id == sp.id).collect();
                            rsx! {
                                if !diseases.is_empty() {
                                    details { style: "margin-bottom: 4px;",
                                        summary { style: "cursor: pointer; font-weight: 600; color: #0f766e;", "{sp.name} ({diseases.len()} bệnh)" }
                                        div { style: "padding-left: 14px; margin-top: 4px;",
                                            for dis in diseases {
                                                details {
                                                    summary { style: "cursor: pointer; font-size: 12.5px;", "{dis.name}" }
                                                    ul { style: "padding-left: 18px; font-size: 12px; color: #475569;",
                                                        for c in cfg.classifications.iter().filter(|c| c.disease_id == dis.id) {
                                                            {
                                                                let score_lbl = c.score.map(|s| format!("Loại {}", s)).unwrap_or_else(|| "Tham chiếu".to_string());
                                                                rsx! { li { "[{score_lbl}] {c.name}" } }
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
            }
        }

        // Nội khoa
        div { class: "card",
            SectionHeader {
                title: "A. Khám Nội khoa (8 phân khoa)".to_string(),
                can_edit_role: can_noi,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("noi").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("noi".to_string(), true); },
                on_save: move |_| on_save_section.call("noi".to_string()),
                on_cancel: move |_| on_cancel_section.call("noi".to_string()),
            }
            for (name, k_res, k_pl, k_doc) in noi_rows {
                div { class: "grid-3", style: "margin-bottom: 8px;",
                    TextField { label: format!("{name} - Kết quả"), field_key: (*k_res).to_string(), form: form, enabled: en_noi, hint: "Bình thường".to_string() }
                    SelectField { label: "Phân loại".to_string(), field_key: (*k_pl).to_string(), form: form, options: pl_opts(), enabled: en_noi }
                    TextField { label: "Bác sĩ khám".to_string(), field_key: (*k_doc).to_string(), form: form, enabled: en_noi }
                }
            }
        }

        // Ngoại khoa
        div { class: "card",
            SectionHeader {
                title: "B. Ngoại khoa".to_string(),
                can_edit_role: can_ngoai,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("ngoai").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("ngoai".to_string(), true); },
                on_save: move |_| on_save_section.call("ngoai".to_string()),
                on_cancel: move |_| on_cancel_section.call("ngoai".to_string()),
            }
            div { class: "grid-3",
                TextField { label: "Kết quả khám".to_string(), field_key: "KET_QUA_KHAM_NGOAI_KHOA".to_string(), form: form, enabled: en_ngoai, hint: "Bình thường".to_string() }
                SelectField { label: "Phân loại".to_string(), field_key: "KHAM_NGOAI_KHOA_PL".to_string(), form: form, options: pl_opts(), enabled: en_ngoai }
                TextField { label: "Bác sĩ khám".to_string(), field_key: "CKDT_KHAM_NGOAI_KHOA".to_string(), form: form, enabled: en_ngoai }
            }
        }

        // Da liễu
        div { class: "card",
            SectionHeader {
                title: "C. Da liễu".to_string(),
                can_edit_role: can_dl,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("dalieu").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("dalieu".to_string(), true); },
                on_save: move |_| on_save_section.call("dalieu".to_string()),
                on_cancel: move |_| on_cancel_section.call("dalieu".to_string()),
            }
            div { class: "grid-3",
                TextField { label: "Kết quả khám".to_string(), field_key: "KET_QUA_KHAM_DA_LIEU".to_string(), form: form, enabled: en_dl, hint: "Bình thường".to_string() }
                SelectField { label: "Phân loại".to_string(), field_key: "KHAM_DA_LIEU_PL".to_string(), form: form, options: pl_opts(), enabled: en_dl }
                TextField { label: "Bác sĩ khám".to_string(), field_key: "CKDT_KHAM_DA_LIEU".to_string(), form: form, enabled: en_dl }
            }
        }

        // Sản phụ khoa
        div { class: "card",
            SectionHeader {
                title: "D. Sản phụ khoa".to_string(),
                can_edit_role: can_san,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("san").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("san".to_string(), true); },
                on_save: move |_| on_save_section.call("san".to_string()),
                on_cancel: move |_| on_cancel_section.call("san".to_string()),
            }
            div { class: "grid-3",
                TextField { label: "Kết quả khám".to_string(), field_key: "KET_QUA_KHAM_SAN_PHU_KHOA".to_string(), form: form, enabled: en_san, hint: "Bình thường".to_string() }
                SelectField { label: "Phân loại".to_string(), field_key: "KHAM_SAN_PHU_KHOA_PL".to_string(), form: form, options: pl_opts(), enabled: en_san }
                TextField { label: "Bác sĩ khám".to_string(), field_key: "CKDT_KHAM_SAN_PHU_KHOA".to_string(), form: form, enabled: en_san }
            }
        }

        // Mắt
        div { class: "card",
            SectionHeader {
                title: "E. Chuyên khoa Mắt".to_string(),
                can_edit_role: can_mat,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("mat").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("mat".to_string(), true); },
                on_save: move |_| on_save_section.call("mat".to_string()),
                on_cancel: move |_| on_cancel_section.call("mat".to_string()),
            }
            div { class: "grid-4", style: "margin-bottom: 8px;",
                TextField { label: "Không kính MP".to_string(), field_key: "KHONG_KINH_MAT_PHAI".to_string(), form: form, enabled: en_mat, hint: "10/10".to_string() }
                TextField { label: "Không kính MT".to_string(), field_key: "KHONG_KINH_MAT_TRAI".to_string(), form: form, enabled: en_mat, hint: "10/10".to_string() }
                TextField { label: "Có kính MP".to_string(), field_key: "CO_KINH_MAT_PHAI".to_string(), form: form, enabled: en_mat }
                TextField { label: "Có kính MT".to_string(), field_key: "CO_KINH_MAT_TRAI".to_string(), form: form, enabled: en_mat }
            }
            div { class: "grid-3",
                TextField { label: "Bệnh khác về mắt".to_string(), field_key: "BENH_KHAC_MAT".to_string(), form: form, enabled: en_mat }
                SelectField { label: "Phân loại Mắt".to_string(), field_key: "KHAM_MAT_PL".to_string(), form: form, options: pl_opts(), enabled: en_mat }
                TextField { label: "Bác sĩ Mắt".to_string(), field_key: "CKDT_KHAM_MAT".to_string(), form: form, enabled: en_mat }
            }
        }

        // Tai mũi họng
        div { class: "card",
            SectionHeader {
                title: "F. Tai - Mũi - Họng".to_string(),
                can_edit_role: can_tmh,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("tmh").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("tmh".to_string(), true); },
                on_save: move |_| on_save_section.call("tmh".to_string()),
                on_cancel: move |_| on_cancel_section.call("tmh".to_string()),
            }
            div { class: "grid-4", style: "margin-bottom: 8px;",
                TextField { label: "Tai trái nói thường".to_string(), field_key: "TAI_TRAI_NOI_THUONG".to_string(), form: form, enabled: en_tmh, hint: "5m".to_string() }
                TextField { label: "Tai trái nói thầm".to_string(), field_key: "TAI_TRAI_NOI_THAM".to_string(), form: form, enabled: en_tmh, hint: "5m".to_string() }
                TextField { label: "Tai phải nói thường".to_string(), field_key: "TAI_PHAI_NOI_THUONG".to_string(), form: form, enabled: en_tmh, hint: "5m".to_string() }
                TextField { label: "Tai phải nói thầm".to_string(), field_key: "TAI_PHAI_NOI_THAM".to_string(), form: form, enabled: en_tmh, hint: "5m".to_string() }
            }
            div { class: "grid-3",
                TextField { label: "Bệnh khác TMH".to_string(), field_key: "BENH_KHAC_TAI_MUI_HONG".to_string(), form: form, enabled: en_tmh }
                SelectField { label: "Phân loại TMH".to_string(), field_key: "KHAM_TAI_MUI_HONG_PL".to_string(), form: form, options: pl_opts(), enabled: en_tmh }
                TextField { label: "Bác sĩ TMH".to_string(), field_key: "CKDT_KHAM_TAI_MUI_HONG".to_string(), form: form, enabled: en_tmh }
            }
        }

        // Răng hàm mặt
        div { class: "card",
            SectionHeader {
                title: "G. Răng - Hàm - Mặt".to_string(),
                can_edit_role: can_rhm,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: *eds.get("rhm").unwrap_or(&false),
                on_edit: move |_| { editing_sections.write().insert("rhm".to_string(), true); },
                on_save: move |_| on_save_section.call("rhm".to_string()),
                on_cancel: move |_| on_cancel_section.call("rhm".to_string()),
            }
            div { class: "grid-2", style: "margin-bottom: 8px;",
                TextField { label: "Hàm trên".to_string(), field_key: "HAM_TREN".to_string(), form: form, enabled: en_rhm, hint: "Bình thường".to_string() }
                TextField { label: "Hàm dưới".to_string(), field_key: "HAM_DUOI".to_string(), form: form, enabled: en_rhm, hint: "Bình thường".to_string() }
            }
            div { class: "grid-3",
                TextField { label: "Bệnh khác RHM".to_string(), field_key: "BENH_KHAC_RANG_HAM_MAT".to_string(), form: form, enabled: en_rhm }
                SelectField { label: "Phân loại RHM".to_string(), field_key: "KHAM_RANG_HAM_MAT_PL".to_string(), form: form, options: pl_opts(), enabled: en_rhm }
                TextField { label: "Bác sĩ RHM".to_string(), field_key: "CKDT_KHAM_RANG_HAM_MAT".to_string(), form: form, enabled: en_rhm }
            }
        }
    }
}

#[component]
pub fn TabParaclinical(
    user: Option<User>,
    selected_record_id: Signal<Option<i64>>,
    selected_record_status: Signal<i64>,
    form: Signal<Map<String, Value>>,
    editing_sections: Signal<HashMap<String, bool>>,
    on_save_section: EventHandler<String>,
    on_cancel_section: EventHandler<String>,
) -> Element {
    let rec_id = *selected_record_id.read();
    if rec_id.is_none() {
        return rsx! { div { class: "card", "Vui lòng chọn một hồ sơ bên danh sách trái." } };
    }
    let rec_status = *selected_record_status.read();
    let eds = editing_sections.read().clone();
    let (can_role, enabled) =
        check_section_unlocked(&user, rec_id, rec_status, &eds, "cls", "bs_cls");
    let is_editing = *eds.get("cls").unwrap_or(&false);

    rsx! {
        div { class: "card",
            SectionHeader {
                title: "5. Cận lâm sàng (Xét nghiệm & CĐHA)".to_string(),
                can_edit_role: can_role,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: is_editing,
                on_edit: move |_| { editing_sections.write().insert("cls".to_string(), true); },
                on_save: move |_| on_save_section.call("cls".to_string()),
                on_cancel: move |_| on_cancel_section.call("cls".to_string()),
            }

            div { class: "grid-2", style: "margin-bottom: 10px;",
                TextAreaField { label: "Xét nghiệm Máu".to_string(), field_key: "CLS_MAU".to_string(), form: form, enabled: enabled, rows: 3 }
                TextAreaField { label: "Xét nghiệm Nước tiểu".to_string(), field_key: "CLS_NUOC_TIEU".to_string(), form: form, enabled: enabled, rows: 3 }
                TextAreaField { label: "X-Quang tim phổi".to_string(), field_key: "CLS_XQ_TIM_PHOI".to_string(), form: form, enabled: enabled, rows: 2 }
                TextAreaField { label: "Siêu âm / Điện tim / Khác".to_string(), field_key: "CLS_SIEU_AM".to_string(), form: form, enabled: enabled, rows: 2 }
            }
            div { class: "grid-4", style: "margin-bottom: 10px;",
                TextField { label: "Mã dịch vụ (CSV)".to_string(), field_key: "MA_DICH_VU".to_string(), form: form, enabled: enabled }
                TextField { label: "Mã chỉ số (CSV)".to_string(), field_key: "MA_CHI_SO".to_string(), form: form, enabled: enabled }
                TextField { label: "Giá trị (CSV)".to_string(), field_key: "GIA_TRI".to_string(), form: form, enabled: enabled }
                TextField { label: "Đơn vị đo".to_string(), field_key: "DON_VI_DO".to_string(), form: form, enabled: enabled }
            }
            div { class: "grid-2",
                TextField { label: "Kết luận Cận lâm sàng".to_string(), field_key: "KET_LUAN_CLS".to_string(), form: form, enabled: enabled }
                TextField { label: "Bác sĩ CLS".to_string(), field_key: "BS_CLS".to_string(), form: form, enabled: enabled }
            }
        }
    }
}

#[component]
pub fn TabConclusion(
    user: Option<User>,
    selected_record_id: Signal<Option<i64>>,
    selected_record_status: Signal<i64>,
    form: Signal<Map<String, Value>>,
    editing_sections: Signal<HashMap<String, bool>>,
    catalogs_cache: Signal<HashMap<String, Vec<CatalogItem>>>,
    on_save_section: EventHandler<String>,
    on_cancel_section: EventHandler<String>,
    on_lock_toggle: EventHandler<(i64, bool)>,
    on_export_docx: EventHandler<(i64, bool)>,
) -> Element {
    let Some(rec_id) = *selected_record_id.read() else {
        return rsx! { div { class: "card", "Vui lòng chọn một hồ sơ bên danh sách trái." } };
    };
    let rec_status = *selected_record_status.read();
    let eds = editing_sections.read().clone();
    let (can_role, enabled) =
        check_section_unlocked(&user, Some(rec_id), rec_status, &eds, "ketluan", "bs_ketluan");
    let is_editing = *eds.get("ketluan").unwrap_or(&false);

    let pl_keys = [
        "KHAM_THE_LUC_PL",
        "NOI_KHOA_TUAN_HOAN_PL",
        "NOI_KHOA_HO_HAP_PL",
        "NOI_KHOA_TIEU_HOA_PL",
        "NOI_KHOA_THAN_TN_SD_PL",
        "NOI_KHOA_NOI_TIET_PL",
        "NOI_KHOA_CO_XUONG_KHOP_PL",
        "NOI_KHOA_THAN_KINH_PL",
        "NOI_KHOA_TAM_THAN_PL",
        "KHAM_NGOAI_KHOA_PL",
        "KHAM_DA_LIEU_PL",
        "KHAM_SAN_PHU_KHOA_PL",
        "KHAM_MAT_PL",
        "KHAM_TAI_MUI_HONG_PL",
        "KHAM_RANG_HAM_MAT_PL",
    ];
    let max_pl = pl_keys
        .iter()
        .filter_map(|k| map_get(&form.read(), k).parse::<i32>().ok())
        .max();

    let icd_items = catalogs_cache.read().get("ICD10").cloned().unwrap_or_default();

    rsx! {
        div { class: "card",
            SectionHeader {
                title: "6. Kết luận sức khỏe & Ký khóa hồ sơ".to_string(),
                can_edit_role: can_role,
                is_new_record: false,
                is_locked_record: rec_status == 1,
                is_editing: is_editing,
                on_edit: move |_| { editing_sections.write().insert("ketluan".to_string(), true); },
                on_save: move |_| on_save_section.call("ketluan".to_string()),
                on_cancel: move |_| on_cancel_section.call("ketluan".to_string()),
            }

            if let Some(m) = max_pl {
                div { style: "background: #e0f2fe; border: 1px solid #7dd3fc; padding: 8px 12px; border-radius: 6px; margin-bottom: 10px; color: #0369a1; font-size: 12.5px;",
                    "💡 Gợi ý phân loại tổng hợp từ các chuyên khoa: Loại {m}"
                }
            }

            div { class: "grid-2", style: "margin-bottom: 10px;",
                SelectField { label: "Phân loại sức khỏe chung (*)".to_string(), field_key: "PHAN_LOAI_SK".to_string(), form: form, options: pl_opts(), enabled: enabled }
                TextField { label: "Bác sĩ Kết luận".to_string(), field_key: "BS_KET_LUAN".to_string(), form: form, enabled: enabled }
            }
            div { style: "margin-bottom: 10px;",
                TextAreaField { label: "Tình trạng sức khỏe / Các bệnh tật (nếu có)".to_string(), field_key: "KET_LUAN_BENH".to_string(), form: form, enabled: enabled, rows: 3 }
            }
            div { style: "margin-bottom: 10px;",
                IcdMultiField { label: "Mã bệnh tật kèm theo (ICD-10)".to_string(), field_key: "CAC_BENH_TAT_NEU_CO".to_string(), form: form, icd_items: icd_items, enabled: enabled }
            }
            div { style: "margin-bottom: 14px;",
                TextAreaField { label: "Đề nghị / Hướng xử trí".to_string(), field_key: "DE_NGHI".to_string(), form: form, enabled: enabled, rows: 2 }
            }

            div { style: "display: flex; gap: 10px; border-top: 1px solid #e2e8f0; padding-top: 12px;",
                if rec_status == 0 {
                    button {
                        class: "btn btn-success",
                        disabled: !can_role,
                        onclick: move |_| on_lock_toggle.call((rec_id, true)),
                        "🔒 Ký khóa hồ sơ"
                    }
                } else {
                    button {
                        class: "btn btn-danger",
                        disabled: !can_role,
                        onclick: move |_| on_lock_toggle.call((rec_id, false)),
                        "🔓 Mở khóa hồ sơ"
                    }
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| on_export_docx.call((rec_id, false)),
                    "📄 Xuất & Mở Phiếu KSK (.docx)"
                }
            }
        }
    }
}
