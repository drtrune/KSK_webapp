// Legacy HTTP request/response DTOs and helpers are retained for compatibility.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::FromRow;

/// Danh sách các Role trong hệ thống KSK (Mục 3)
pub const VALID_ROLES: &[&str] = &[
    "admin",
    "business",
    "reception",
    "tiensu",
    "bs_theluc",
    "bs_noi",
    "bs_ngoai",
    "bs_dalieu",
    "bs_san",
    "bs_mat",
    "bs_tmh",
    "bs_rhm",
    "bs_cls",
    "bs_ketluan",
    "bs_nhi",
];

/// Danh sách 123 cột chuẩn xuất CSV theo Quyết định 3176/QĐ-BYT & Quyết định 1804/QĐ-BYT
pub const CSV_EXPORT_COLUMNS: &[&str] = &[
    // I. Thông tin hành chính (1-15)
    "HO_TEN",
    "GIOI_TINH",
    "NGAY_SINH",
    "MA_DAN_TOC",
    "SO_CCCD",
    "NGAYCAP_CCCD",
    "NOICAP_CCCD",
    "DIA_CHI",
    "MATINH_CU_TRU",
    "MAXA_CU_TRU",
    "MA_NGHE_NGHIEP",
    "NOI_LAM_VIEC_HOC_TAP",
    "DIEN_THOAI",
    "NHOM_MAU",
    "LY_DO_VV",
    // II. Thông tin chung về lần khám (16-22)
    "MA_LK",
    "MA_CSKCB",
    "MA_GTIN_CSKCB",
    "DOI_TUONG",
    "NGUON_CHI_TRA",
    "MA_LOAI_KCB",
    "NGAY_VAO",
    // III.A. Tiền sử gia đình (23-24)
    "TSGD_MAC_BENH",
    "TSGD_MA_BENH",
    // III.B. Tiền sử bản thân (25-53)
    "TSBT_BENH_TRONG_5_NAM_QUA",
    "TSBT_BENH_THAN_KINH",
    "TSBT_BENH_MAT",
    "TSBT_BENH_TAI",
    "TSBT_BENH_TIM",
    "TSBT_PHAU_THUAT_TIM",
    "TSBT_TANG_HUYET_AP",
    "TSBT_KHO_THO",
    "TSBT_BENH_PHOI",
    "TSBT_BENH_THAN",
    "TSBT_NGHIEN_RUOU",
    "TSBT_DAI_THAO_DUONG",
    "TSBT_BENH_TAM_THAN",
    "TSBT_MAT_Y_THUC",
    "TSBT_NGAT",
    "TSBT_BENH_TIEU_HOA",
    "TSBT_ROI_LOAN_GIAC_NGU",
    "TSBT_TAI_BIEN",
    "TSBT_BENH_COT_SONG",
    "TSBT_RUOU_THUONG_XUYEN",
    "TSBT_MA_TUY",
    "TSBT_BENH_KHAC",
    "TSBT_MA_BENH_KHAC",
    "TSBT_DANG_DIEU_TRI_BENH",
    "TSBT_MA_BENH",
    "TSBT_TEN_THUOC_LIEU_LUONG",
    "TSBT_THAI_SAN",
    "TSBT_MA_BENH_THAI_SAN",
    "TSBT_TEN_THUOC_THAI_SAN",
    // IV. Khám thể lực (54-59)
    "CHIEU_CAO",
    "CAN_NANG",
    "CHI_SO_BMI",
    "MACH",
    "HUYET_AP",
    "KHAM_THE_LUC_PL",
    // V.A. Nội khoa (60-83)
    "NOI_KHOA_TUAN_HOAN",
    "NOI_KHOA_TUAN_HOAN_PL",
    "CKDT_NOI_KHOA_TUAN_HOAN",
    "NOI_KHOA_HO_HAP",
    "NOI_KHOA_HO_HAP_PL",
    "CKDT_NOI_KHOA_HO_HAP",
    "NOI_KHOA_TIEU_HOA",
    "NOI_KHOA_TIEU_HOA_PL",
    "CKDT_NOI_KHOA_TIEU_HOA",
    "NOI_KHOA_THAN_TN_SD",
    "NOI_KHOA_THAN_TN_SD_PL",
    "CKDT_NOI_KHOA_THAN_TN_SD",
    "NOI_KHOA_NOI_TIET",
    "NOI_KHOA_NOI_TIET_PL",
    "CKDT_NOI_KHOA_NOI_TIET",
    "NOI_KHOA_CO_XUONG_KHOP",
    "NOI_KHOA_CO_XUONG_KHOP_PL",
    "CKDT_NOI_KHOA_CO_XUONG_KHOP",
    "NOI_KHOA_THAN_KINH",
    "NOI_KHOA_THAN_KINH_PL",
    "CKDT_NOI_KHOA_THAN_KINH",
    "NOI_KHOA_TAM_THAN",
    "NOI_KHOA_TAM_THAN_PL",
    "CKDT_NOI_KHOA_TAM_THAN",
    // V.B. Ngoại khoa (84-86)
    "KET_QUA_KHAM_NGOAI_KHOA",
    "KHAM_NGOAI_KHOA_PL",
    "CKDT_KHAM_NGOAI_KHOA",
    // V.C. Da liễu (87-89)
    "KET_QUA_KHAM_DA_LIEU",
    "KHAM_DA_LIEU_PL",
    "CKDT_KHAM_DA_LIEU",
    // V.D. Sản phụ khoa (90-92)
    "KET_QUA_KHAM_SAN_PHU_KHOA",
    "KHAM_SAN_PHU_KHOA_PL",
    "CKDT_KHAM_SAN_PHU_KHOA",
    // V.E. Mắt (93-99)
    "KHONG_KINH_MAT_PHAI",
    "KHONG_KINH_MAT_TRAI",
    "CO_KINH_MAT_PHAI",
    "CO_KINH_MAT_TRAI",
    "BENH_KHAC_MAT",
    "KHAM_MAT_PL",
    "CKDT_KHAM_MAT",
    // V.F. Tai - Mũi - Họng (100-106)
    "TAI_TRAI_NOI_THUONG",
    "TAI_TRAI_NOI_THAM",
    "TAI_PHAI_NOI_THUONG",
    "TAI_PHAI_NOI_THAM",
    "BENH_KHAC_TAI_MUI_HONG",
    "KHAM_TAI_MUI_HONG_PL",
    "CKDT_KHAM_TAI_MUI_HONG",
    // V.G. Răng - Hàm - Mặt (107-111)
    "HAM_TREN",
    "HAM_DUOI",
    "BENH_KHAC_RANG_HAM_MAT",
    "KHAM_RANG_HAM_MAT_PL",
    "CKDT_KHAM_RANG_HAM_MAT",
    // VI. Khám cận lâm sàng (112-118)
    "MA_DICH_VU",
    "MA_CHI_SO",
    "GIA_TRI",
    "DON_VI_DO",
    "MO_TA",
    "KET_LUAN",
    "CKDT_KET_QUA_CLS",
    // VII. Kết luận (119-121)
    "PHAN_LOAI_SK",
    "KET_LUAN_BENH",
    "CAC_BENH_TAT_NEU_CO",
    // VIII. Chữ ký số (122-123)
    "CKS_NGUOI_KET_LUAN",
    "CKS_BENH_VIEN",
];

// Các nhóm trường theo từng Role chuyên khoa để kiểm soát quyền sửa (RBAC)
pub const FIELDS_RECEPTION: &[&str] = &[
    "MA_TIEP_NHAN",
    "MA_Y_TE",
    "SO_PHIEU",
    "MA_CSKCB",
    "MA_GTIN_CSKCB",
    "NGAY_VAO",
    "HO_TEN",
    "GIOI_TINH",
    "NGAY_SINH",
    "MA_DAN_TOC",
    "SO_CCCD",
    "NGAYCAP_CCCD",
    "NOICAP_CCCD",
    "MATINH_CU_TRU",
    "MAXA_CU_TRU",
    "DIA_CHI",
    "MA_NGHE_NGHIEP",
    "NHOM_MAU",
    "DOI_TUONG",
    "NGUON_CHI_TRA",
    "NOI_LAM_VIEC_HOC_TAP",
    "DIEN_THOAI",
    "LY_DO_VV",
    "MA_LOAI_KCB",
    "NGUOI_GIAM_HO",
    "MOI_QUAN_HE_VOI_TRE",
    "DIEN_THOAI_NGH",
    "SO_CCCD_NGH",
    "TEN_CSGD",
    "MA_CSGD",
    "SINH_NON",
    "TUAN_THAI",
    "GIO_KHAM",
    "BAC_SI_KHAM",
];

pub const FIELDS_TIENSU: &[&str] = &[
    "TSGD_MAC_BENH",
    "TSGD_MA_BENH",
    "TSBT_BENH_TRONG_5_NAM_QUA",
    "TSBT_BENH_THAN_KINH",
    "TSBT_BENH_MAT",
    "TSBT_BENH_TAI",
    "TSBT_BENH_TIM",
    "TSBT_PHAU_THUAT_TIM",
    "TSBT_TANG_HUYET_AP",
    "TSBT_KHO_THO",
    "TSBT_BENH_PHOI",
    "TSBT_BENH_THAN",
    "TSBT_NGHIEN_RUOU",
    "TSBT_DAI_THAO_DUONG",
    "TSBT_BENH_TAM_THAN",
    "TSBT_MAT_Y_THUC",
    "TSBT_NGAT",
    "TSBT_BENH_TIEU_HOA",
    "TSBT_ROI_LOAN_GIAC_NGU",
    "TSBT_TAI_BIEN",
    "TSBT_BENH_COT_SONG",
    "TSBT_RUOU_THUONG_XUYEN",
    "TSBT_MA_TUY",
    "TSBT_BENH_KHAC",
    "TSBT_MA_BENH_KHAC",
    "TSBT_DANG_DIEU_TRI_BENH",
    "TSBT_MA_BENH",
    "TSBT_TEN_THUOC_LIEU_LUONG",
    "TSBT_THAI_SAN",
    "TSBT_MA_BENH_THAI_SAN",
    "TSBT_TEN_THUOC_THAI_SAN",
];

pub const FIELDS_BS_THELUC: &[&str] = &[
    "CHIEU_CAO",
    "CAN_NANG",
    "CHI_SO_BMI",
    "MACH",
    "HUYET_AP",
    "KHAM_THE_LUC_PL",
    "BS_THE_LUC",
];

pub const FIELDS_BS_NOI: &[&str] = &[
    "NOI_KHOA_TUAN_HOAN",
    "NOI_KHOA_TUAN_HOAN_PL",
    "NOI_KHOA_HO_HAP",
    "NOI_KHOA_HO_HAP_PL",
    "NOI_KHOA_TIEU_HOA",
    "NOI_KHOA_TIEU_HOA_PL",
    "NOI_KHOA_THAN_TN_SD",
    "NOI_KHOA_THAN_TIETNIEU_PL",
    "NOI_KHOA_NOI_TIET",
    "NOI_KHOA_NOI_TIET_PL",
    "NOI_KHOA_CO_XUONG_KHOP",
    "NOI_KHOA_CO_XUONG_KHOP_PL",
    "NOI_KHOA_THAN_KINH",
    "NOI_KHOA_THAN_KINH_PL",
    "NOI_KHOA_TAM_THAN",
    "NOI_KHOA_TAM_THAN_PL",
    "BS_NOI",
];

pub const FIELDS_BS_NGOAI: &[&str] = &[
    "KET_QUA_KHAM_NGOAI_KHOA",
    "KHAM_NGOAI_KHOA_PL",
    "BS_NGOAI",
];

pub const FIELDS_BS_DALIEU: &[&str] = &[
    "KET_QUA_KHAM_DA_LIEU",
    "KHAM_DA_LIEU_PL",
    "BS_DA_LIEU",
];

pub const FIELDS_BS_SAN: &[&str] = &[
    "KET_QUA_KHAM_SAN_PHU_KHOA",
    "KHAM_SAN_PHU_KHOA_PL",
    "BS_SAN",
];

pub const FIELDS_BS_MAT: &[&str] = &[
    "KHONG_KINH_MAT_PHAI",
    "KHONG_KINH_MAT_TRAI",
    "CO_KINH_MAT_PHAI",
    "CO_KINH_MAT_TRAI",
    "BENH_KHAC_MAT",
    "KHAM_MAT_PL",
    "BS_MAT",
];

pub const FIELDS_BS_TMH: &[&str] = &[
    "TAI_TRAI_NOI_THUONG",
    "TAI_TRAI_NOI_THAM",
    "TAI_PHAI_NOI_THUONG",
    "TAI_PHAI_NOI_THAM",
    "BENH_KHAC_TAI_MUI_HONG",
    "KHAM_TAI_MUI_HONG_PL",
    "BS_TMH",
];

pub const FIELDS_BS_RHM: &[&str] = &[
    "HAM_TREN",
    "HAM_DUOI",
    "BENH_KHAC_RANG_HAM_MAT",
    "KHAM_RANG_HAM_MAT_PL",
    "BS_RHM",
];

pub const FIELDS_BS_CLS: &[&str] = &[
    "CLS_HUYET_HOC",
    "CLS_DUONG_MAU",
    "CLS_URE",
    "CLS_CREATININ",
    "CLS_ASAT_GOT",
    "CLS_ALAT_GPT",
    "CLS_NUOC_TIEU",
    "CLS_XQ_TIM_PHOI",
    "CLS_KHAC_CO",
    "CLS_KHAC_MO_TA",
    "CLS_DINH_KY_KQ",
    "CLS_DINH_KY_KET_QUA",
    "BS_CLS",
];

pub const FIELDS_BS_KETLUAN: &[&str] = &[
    "PHAN_LOAI_SK",
    "KET_LUAN_BENH",
    "CAC_BENH_TAT_NEU_CO",
    "BS_KET_LUAN",
];

pub const FIELDS_CLINICAL_ALL: &[&str] = &[
    // NOI
    "NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL",
    "NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TIETNIEU_PL",
    "NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL",
    "NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL", "BS_NOI",
    // NGOAI
    "KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA_PL", "BS_NGOAI",
    // DA LIEU
    "KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU_PL", "BS_DA_LIEU",
    // SAN
    "KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA_PL", "BS_SAN",
    // MAT
    "KHONG_KINH_MAT_PHAI", "KHONG_KINH_MAT_TRAI", "CO_KINH_MAT_PHAI", "CO_KINH_MAT_TRAI",
    "BENH_KHAC_MAT", "KHAM_MAT_PL", "BS_MAT",
    // TMH
    "TAI_TRAI_NOI_THUONG", "TAI_TRAI_NOI_THAM", "TAI_PHAI_NOI_THUONG", "TAI_PHAI_NOI_THAM",
    "BENH_KHAC_TAI_MUI_HONG", "KHAM_TAI_MUI_HONG_PL", "BS_TMH",
    // RHM
    "HAM_TREN", "HAM_DUOI", "BENH_KHAC_RANG_HAM_MAT", "KHAM_RANG_HAM_MAT_PL", "BS_RHM",
];

pub fn get_section_label(sec: &str) -> &'static str {
    match sec {
        "reception" => "1. Thông tin Hành chính & Tiếp nhận",
        "history" => "2. Tiền sử bệnh",
        "physical" => "3. Thể lực & Sinh tồn",
        "clinical_noi" => "4.1 Khám Nội khoa",
        "clinical_ngoai" => "4.2 Khám Ngoại khoa",
        "clinical_dalieu" => "4.3 Khám Da liễu",
        "clinical_san" => "4.4 Khám Sản phụ khoa",
        "clinical_mat" => "4.5 Khám Mắt",
        "clinical_tmh" => "4.6 Khám Tai Mũi Họng",
        "clinical_rhm" => "4.7 Khám Răng Hàm Mặt",
        "clinical_all" => "4. Khám lâm sàng chuyên khoa",
        "paraclinical" => "5. Cận lâm sàng",
        "conclusion" => "6. Kết luận",
        _ => "Chuyên khoa",
    }
}

pub fn get_section_required_role(sec: &str) -> Option<&'static str> {
    match sec {
        "reception" => Some("reception"),
        "history" => Some("tiensu"),
        "physical" => Some("bs_theluc"),
        "clinical_noi" => Some("bs_noi"),
        "clinical_ngoai" => Some("bs_ngoai"),
        "clinical_dalieu" => Some("bs_dalieu"),
        "clinical_san" => Some("bs_san"),
        "clinical_mat" => Some("bs_mat"),
        "clinical_tmh" => Some("bs_tmh"),
        "clinical_rhm" => Some("bs_rhm"),
        "paraclinical" => Some("bs_cls"),
        "conclusion" => Some("bs_ketluan"),
        _ => None,
    }
}

pub fn get_section_fields(sec: &str) -> &'static [&'static str] {
    match sec {
        "reception" => FIELDS_RECEPTION,
        "history" => FIELDS_TIENSU,
        "physical" => FIELDS_BS_THELUC,
        "clinical_noi" | "clin_noi" => FIELDS_BS_NOI,
        "clinical_ngoai" | "clin_ngoai" => FIELDS_BS_NGOAI,
        "clinical_dalieu" | "clin_dalieu" => FIELDS_BS_DALIEU,
        "clinical_san" | "clin_san" => FIELDS_BS_SAN,
        "clinical_mat" | "clin_mat" => FIELDS_BS_MAT,
        "clinical_tmh" | "clin_tmh" => FIELDS_BS_TMH,
        "clinical_rhm" | "clin_rhm" => FIELDS_BS_RHM,
        "clinical_all" => FIELDS_CLINICAL_ALL,
        "paraclinical" => FIELDS_BS_CLS,
        "conclusion" => FIELDS_BS_KETLUAN,
        _ => &[],
    }
}

pub fn field_section_name(field: &str) -> &'static str {
    if FIELDS_RECEPTION.contains(&field) {
        "Hành chính & Tiếp nhận"
    } else if FIELDS_TIENSU.contains(&field) {
        "Tiền sử bệnh"
    } else if FIELDS_BS_THELUC.contains(&field) {
        "Thể lực & Sinh tồn"
    } else if FIELDS_BS_NOI.contains(&field) {
        "Khám Nội khoa"
    } else if FIELDS_BS_NGOAI.contains(&field) {
        "Khám Ngoại khoa"
    } else if FIELDS_BS_DALIEU.contains(&field) {
        "Khám Da liễu"
    } else if FIELDS_BS_SAN.contains(&field) {
        "Khám Sản phụ khoa"
    } else if FIELDS_BS_MAT.contains(&field) {
        "Khám Mắt"
    } else if FIELDS_BS_TMH.contains(&field) {
        "Khám Tai Mũi Họng"
    } else if FIELDS_BS_RHM.contains(&field) {
        "Khám Răng Hàm Mặt"
    } else if FIELDS_BS_CLS.contains(&field) {
        "Cận lâm sàng"
    } else if FIELDS_BS_KETLUAN.contains(&field) {
        "Kết luận"
    } else {
        "Thông tin chung"
    }
}

/// Kiểm tra một trường dữ liệu có được phép sửa bởi danh sách các Role (phân tách bởi dấu phẩy) hay không
pub fn can_user_roles_edit_field(user_roles_str: &str, field: &str) -> bool {
    let roles: Vec<&str> = user_roles_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
    if roles.contains(&"admin") {
        return true;
    }
    if field == "BAC_SI_KHAM" && roles.contains(&"bs_nhi") {
        return true;
    }
    if field.starts_with("D6_")
        || field.starts_with("U18_")
        || field.starts_with("MG_")
        || field.starts_with("HS_")
        || field.starts_with("LX_")
    {
        return !roles.is_empty();
    }
    for r in roles {
        let allowed = match r {
            "reception" => FIELDS_RECEPTION.contains(&field),
            "tiensu" => FIELDS_TIENSU.contains(&field),
            "bs_theluc" => FIELDS_BS_THELUC.contains(&field),
            "bs_noi" => FIELDS_BS_NOI.contains(&field),
            "bs_ngoai" => FIELDS_BS_NGOAI.contains(&field),
            "bs_dalieu" => FIELDS_BS_DALIEU.contains(&field),
            "bs_san" => FIELDS_BS_SAN.contains(&field),
            "bs_mat" => FIELDS_BS_MAT.contains(&field),
            "bs_tmh" => FIELDS_BS_TMH.contains(&field),
            "bs_rhm" => FIELDS_BS_RHM.contains(&field),
            "bs_cls" => FIELDS_BS_CLS.contains(&field),
            "bs_ketluan" => FIELDS_BS_KETLUAN.contains(&field),
            _ => false,
        };
        if allowed {
            return true;
        }
    }
    false
}

/// Trộn (merge) dữ liệu gửi lên vào payload hiện tại theo đúng danh sách quyền (Multi-Role) và Section
pub fn merge_payload_by_roles(
    existing_json: Option<&str>,
    incoming: &Map<String, Value>,
    user_roles_str: &str,
    default_ma_cskcb: &str,
    default_ma_gtin: &str,
    section: Option<&str>,
) -> Map<String, Value> {
    let mut base: Map<String, Value> = existing_json
        .and_then(|s| serde_json::from_str::<Map<String, Value>>(s).ok())
        .unwrap_or_default();

    // Khởi tạo giá trị cơ sở KCB cho hồ sơ mới nếu chưa có
    if !base.contains_key("MA_CSKCB") || base.get("MA_CSKCB").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
        base.insert("MA_CSKCB".to_string(), Value::String(default_ma_cskcb.to_string()));
    }
    if !base.contains_key("MA_GTIN_CSKCB") || base.get("MA_GTIN_CSKCB").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
        base.insert("MA_GTIN_CSKCB".to_string(), Value::String(default_ma_gtin.to_string()));
    }

    let filter_fields = section.and_then(|sec| {
        let f = get_section_fields(sec);
        if f.is_empty() { None } else { Some(f) }
    });

    // Chỉ cập nhật các trường thuộc quyền hạn của người dùng (Multi-role), KHÔNG tự tạo dữ liệu giả ở backend
    for (k, v) in incoming {
        if k == "SELECTED_CLASSIFICATIONS_JSON" || k == "_EXAMINED_SECTIONS" || k == "_EXAM_TEMPLATES" {
            match v {
                Value::String(s) => {
                    base.insert(k.clone(), Value::String(s.clone()));
                }
                Value::Array(_) | Value::Object(_) => {
                    base.insert(k.clone(), v.clone());
                }
                _ => {}
            }
            continue;
        }
        if let Some(allowed_fields) = filter_fields {
            let is_custom_tpl_field = k.starts_with("D6_")
                || k.starts_with("U18_")
                || k.starts_with("MG_")
                || k.starts_with("HS_")
                || k.starts_with("LX_");
            if !is_custom_tpl_field && !allowed_fields.contains(&k.as_str()) {
                continue;
            }
        }
        if can_user_roles_edit_field(user_roles_str, k) {
            let val_str = match v {
                Value::String(s) => s.trim().to_string(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                Value::Null => String::new(),
                other => other.to_string(),
            };
            base.insert(k.clone(), Value::String(val_str));
        }
    }

    // Tự động tính lại BMI nếu có chiều cao và cân nặng hợp lệ
    for prefix in ["", "D6_", "U18_", "MG_", "HS_"] {
        let cc_key = format!("{}CHIEU_CAO", prefix);
        let cn_key = format!("{}CAN_NANG", prefix);
        let bmi_key = format!("{}CHI_SO_BMI", prefix);
        if let (Some(cc), Some(cn)) = (
            base.get(&cc_key).and_then(|v| v.as_str()).and_then(|s| s.replace(',', ".").parse::<f64>().ok()),
            base.get(&cn_key).and_then(|v| v.as_str()).and_then(|s| s.replace(',', ".").parse::<f64>().ok()),
        ) {
            if cc > 30.0 && cn > 2.0 {
                let height_m = cc / 100.0;
                let bmi = cn / (height_m * height_m);
                base.insert(bmi_key, Value::String(format!("{:.2}", bmi)));
            }
        }
    }

    base
}

// ============================================================================
// BẢNG USERS & AUTHENTICATION
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub role: String, // Lưu chuỗi csv: "reception,bs_theluc"
    pub is_locked: i64,
    pub created_at: String,
}

impl User {
    pub fn roles(&self) -> Vec<String> {
        self.role.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
    }

    pub fn has_role(&self, r: &str) -> bool {
        self.roles().iter().any(|role| role == r || role == "admin")
    }

    pub fn has_any_role(&self, allowed: &[&str]) -> bool {
        allowed.iter().any(|r| self.has_role(r))
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub full_name: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub full_name: String,
    pub roles: Vec<String>,
    pub password: Option<String>,
    pub is_locked: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
    #[serde(default)]
    pub revoke_other_sessions: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfileRequest {
    pub full_name: String,
}

// ============================================================================
// TÍCH HỢP API HIS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HisRequest {
    pub ma_tiep_nhan: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HisPatientData {
    #[serde(default)]
    pub ma_tn: String,
    #[serde(default)]
    pub ma_y_te: String,
    #[serde(default)]
    pub ho_ten: String,
    #[serde(default)]
    pub gioi_tinh: i32,
    #[serde(default)]
    pub ngay_sinh: String,
    #[serde(default)]
    pub so_cccd: String,
    #[serde(default)]
    pub ngaycap_cccd: String,
    #[serde(default)]
    pub noicap_cccd: String,
    #[serde(default)]
    pub ma_dan_toc: String,
    #[serde(default)]
    pub ma_nghe_nghiep: String,
    #[serde(default)]
    pub so_dien_thoai: String,
    #[serde(default)]
    pub dia_chi: String,
    #[serde(default)]
    pub matinh_cu_tru: String,
    #[serde(default)]
    pub maxa_cu_tru: String,
    #[serde(default)]
    pub ngay_vao: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HisResponse {
    pub status: i32,
    pub message: String,
    pub data: Option<HisPatientData>,
}

#[derive(Debug, Deserialize)]
pub struct HisQuery {
    pub ma_tn: String,
}

#[derive(Debug, Deserialize)]
pub struct TestHisRequest {
    pub ma_tn: String,
    pub api_url: Option<String>,
    pub http_method: Option<String>,
    pub auth_header: Option<String>,
    pub api_key: Option<String>,
    pub payload_template: Option<String>,
}

// ============================================================================
// BẢNG KSK_RECORDS (HỒ SƠ KHÁM SỨC KHỎE)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct KskRecordRow {
    pub id: i64,
    pub ma_tiep_nhan: String,
    pub ma_y_te: String,
    pub ho_ten: String,
    pub gioi_tinh: String,
    pub ngay_sinh: String,
    pub so_cccd: String,
    pub ngay_vao: String,
    pub status: i64,
    #[sqlx(default)]
    pub exam_mode: String,
    #[sqlx(default)]
    pub contract_id: Option<i64>,
    #[sqlx(default)]
    pub exam_templates: String,
    pub payload_json: String,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KskRecordDto {
    pub id: i64,
    pub ma_tiep_nhan: String,
    pub ma_y_te: String,
    pub ho_ten: String,
    pub gioi_tinh: String,
    pub ngay_sinh: String,
    pub so_cccd: String,
    pub ngay_vao: String,
    pub status: i64,
    pub exam_mode: String,
    pub contract_id: Option<i64>,
    pub exam_templates: String,
    pub payload: Value,
    pub updated_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<KskRecordRow> for KskRecordDto {
    fn from(row: KskRecordRow) -> Self {
        let payload = serde_json::from_str::<Value>(&row.payload_json)
            .unwrap_or_else(|_| Value::Object(Map::new()));
        Self {
            id: row.id,
            ma_tiep_nhan: row.ma_tiep_nhan,
            ma_y_te: row.ma_y_te,
            ho_ten: row.ho_ten,
            gioi_tinh: row.gioi_tinh,
            ngay_sinh: row.ngay_sinh,
            so_cccd: row.so_cccd,
            ngay_vao: row.ngay_vao,
            status: row.status,
            exam_mode: if row.exam_mode.is_empty() {
                "le".to_string()
            } else {
                row.exam_mode
            },
            contract_id: row.contract_id,
            exam_templates: row.exam_templates,
            payload,
            updated_by: row.updated_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub tax_code: String,
    pub address: String,
    pub contact_person: String,
    pub contact_phone: String,
    pub note: String,
    pub is_active: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExamContract {
    pub id: i64,
    pub company_id: i64,
    pub contract_code: String,
    pub title: String,
    pub exam_date: String,
    pub expected_count: i64,
    pub status: i64,
    #[sqlx(default)]
    pub scan_file_path: String,
    #[sqlx(default)]
    pub default_templates: String,
    pub note: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExamContractSummary {
    pub id: i64,
    pub company_id: i64,
    pub company_code: String,
    pub company_name: String,
    pub contract_code: String,
    pub title: String,
    pub exam_date: String,
    pub expected_count: i64,
    pub status: i64,
    #[sqlx(default)]
    pub scan_file_path: String,
    #[sqlx(default)]
    pub default_templates: String,
    pub note: String,
    pub total_records: i64,
    pub done_records: i64,
    pub signed_records: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupImportRow {
    pub row_index: usize,
    pub ma_tiep_nhan: String,
    pub ma_y_te: String,
    pub ho_ten: String,
    pub ngay_sinh: String,
    pub gioi_tinh: String,
    pub so_cccd: String,
    pub ngaycap_cccd: String,
    pub noicap_cccd: String,
    pub dien_thoai: String,
    pub matinh_cu_tru: String,
    pub maxa_cu_tru: String,
    pub dia_chi: String,
    pub ma_dan_toc: String,
    pub ma_nghe_nghiep: String,
    pub noi_lam_viec_hoc_tap: String,
    pub ma_loai_kcb: String,
    pub ngay_vao: String,
    pub nhom_mau: String,
    pub ly_do_vv: String,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SaveRecordRequest {
    pub id: Option<i64>,
    pub ma_tiep_nhan: Option<String>,
    pub payload: Map<String, Value>,
    pub section: Option<String>,
    pub exam_templates: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordFilterQuery {
    pub q: Option<String>,
    pub status: Option<i64>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub workplace: Option<String>,
    pub classification: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CsvExportFilterQuery {
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub status: Option<i64>,
    pub q: Option<String>,
    pub workplace: Option<String>,
    pub classification: Option<String>,
    pub selected_ids: Option<String>,
    pub token: Option<String>,
}

// ============================================================================
// BẢNG CATALOGS
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, FromRow)]
pub struct CatalogItem {
    pub id: i64,
    pub cat_type: String,
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub parent_code: String,
    #[serde(default = "default_one")]
    pub is_active: i64,
    #[serde(default)]
    pub is_default: i64,
}

fn default_one() -> i64 {
    1
}

#[derive(Debug, Deserialize)]
pub struct CatalogQuery {
    pub cat_type: Option<String>,
    pub parent_code: Option<String>,
    pub q: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BulkCatalogImportRequest {
    pub cat_type: String,
    pub raw_text: String,
    #[serde(default)]
    pub replace_all: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCatalogRequest {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub parent_code: String,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteCatalogRequest {
    pub ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct BulkStatusCatalogRequest {
    pub ids: Vec<i64>,
    pub is_active: i64,
}

// ============================================================================
// ADMIN SETTINGS & DATABASE MANAGEMENT
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SwitchDbRequest {
    pub db_path: String,
    #[serde(default)]
    pub create_if_missing: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct DbInfo {
    pub current_db_path: String,
    pub file_size_bytes: u64,
    pub records_count: i64,
    pub users_count: i64,
    pub catalogs_count: i64,
    pub available_db_files: Vec<String>,
}

// ============================================================================
// HIS AHK V2 UIA BRIDGE PAYLOAD
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HisPushRequest {
    #[serde(default, alias = "ma_tn", alias = "MA_TIEP_NHAN")]
    pub ma_tiep_nhan: String,
    #[serde(default, alias = "MA_Y_TE")]
    pub ma_y_te: String,
    #[serde(default, alias = "HO_TEN")]
    pub ho_ten: String,
    #[serde(default, alias = "GIOI_TINH")]
    pub gioi_tinh: String,
    #[serde(default, alias = "NGAY_SINH")]
    pub ngay_sinh: String,
    #[serde(default, alias = "SO_CCCD")]
    pub so_cccd: String,
    #[serde(default, alias = "NGAYCAP_CCCD")]
    pub ngaycap_cccd: String,
    #[serde(default, alias = "NOICAP_CCCD")]
    pub noicap_cccd: String,
    #[serde(default, alias = "DIEN_THOAI", alias = "dien_thoai")]
    pub so_dien_thoai: String,
    #[serde(default, alias = "DIA_CHI")]
    pub dia_chi: String,
    #[serde(default)]
    pub so_nha: String,
    #[serde(default)]
    pub ten_dan_toc: String,
    #[serde(default)]
    pub ten_nghe_nghiep: String,
    #[serde(default)]
    pub ten_xa_phuong_tinh: String,
    #[serde(default, alias = "NGAY_VAO")]
    pub ngay_vao: String,
    #[serde(default, alias = "LY_DO_VV")]
    pub ly_do_vv: String,
    #[serde(default)]
    pub tien_su_benh_raw: String,
}

// ============================================================================
// AUTO-UPDATE MANIFEST (SHARED NETWORK DRIVE)
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    #[serde(default = "default_exe_name")]
    pub exe_name: String,
    #[serde(default)]
    pub changelog: String,
    #[serde(default)]
    pub updated_at: String,
}

fn default_exe_name() -> String {
    "ksk_webapp.exe".to_string()
}

// ============================================================================
// BẢNG PHÂN LOẠI SỨC KHỎE (TỪ asset/phan_loai_config.json)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecialtyDef {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiseaseDef {
    pub id: String,
    pub name: String,
    pub specialty_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassificationDef {
    pub id: String,
    pub disease_id: String,
    pub name: String,
    #[serde(default)]
    pub score: Option<i64>,
    pub ref_rule: Option<String>,
    pub ref_disease_id: Option<String>,
    #[serde(default)]
    pub adjust: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhanLoaiConfig {
    pub specialties: Vec<SpecialtyDef>,
    pub diseases: Vec<DiseaseDef>,
    pub classifications: Vec<ClassificationDef>,
}

pub fn phan_loai_config() -> &'static PhanLoaiConfig {
    static CONFIG: std::sync::OnceLock<PhanLoaiConfig> = std::sync::OnceLock::new();
    CONFIG.get_or_init(|| {
        let raw = include_str!("../asset/phan_loai_config.json");
        serde_json::from_str(raw).expect("Invalid embedded asset/phan_loai_config.json")
    })
}

/// Đọc cấu hình phân loại bệnh từ `asset/phan_loai_config.json` trên ổ đĩa nếu có,
/// fallback về bản nhúng trong binary nếu chưa tồn tại hoặc lỗi đọc.
pub fn load_phan_loai_config_live() -> PhanLoaiConfig {
    if let Ok(raw) = std::fs::read_to_string("asset/phan_loai_config.json") {
        if let Ok(cfg) = serde_json::from_str::<PhanLoaiConfig>(&raw) {
            return cfg;
        }
    }
    phan_loai_config().clone()
}

pub fn parse_adjust_delta(adjust: &Option<Value>) -> i64 {
    match adjust {
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(Value::String(s)) => {
            let digits: String = s
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
                .collect();
            digits.parse::<i64>().unwrap_or(0)
        }
        _ => 0,
    }
}

pub fn parse_first_number(raw: &str) -> Option<f64> {
    let trimmed = raw.trim().replace(',', ".");
    if trimmed.is_empty() {
        return None;
    }
    let mut num = String::new();
    let mut started = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_digit() || (ch == '.' && started) {
            num.push(ch);
            started = true;
        } else if started {
            break;
        }
    }
    num.parse::<f64>().ok()
}

pub fn parse_bp_values(raw: &str) -> Option<(i32, i32)> {
    let parts: Vec<&str> = raw.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    let sys = parse_first_number(parts[0])? as i32;
    let dia = parse_first_number(parts[1])? as i32;
    if sys > 30 && dia > 20 {
        Some((sys, dia))
    } else {
        None
    }
}

pub fn calc_patient_age(ngay_sinh: &str) -> Option<i32> {
    let s = ngay_sinh.trim();
    if s.is_empty() {
        return None;
    }
    let year: i32 = if s.len() >= 8 && s.chars().take(4).all(|c| c.is_ascii_digit()) {
        s[0..4].parse().ok()?
    } else if let Some(last) = s.split('/').last().filter(|p| p.len() == 4) {
        last.parse().ok()?
    } else {
        return None;
    };
    let cur_year = chrono::Local::now().format("%Y").to_string().parse::<i32>().ok()?;
    Some((cur_year - year).max(0))
}

pub fn auto_classify_vision(
    r_nokinh: &str,
    l_nokinh: &str,
    r_kinh: &str,
    l_kinh: &str,
) -> (&'static str, i64) {
    let r_nk = parse_first_number(r_nokinh).unwrap_or(10.0);
    let l_nk = parse_first_number(l_nokinh).unwrap_or(10.0);
    let r_k = parse_first_number(r_kinh).filter(|&v| v > 0.0);
    let l_k = parse_first_number(l_kinh).filter(|&v| v > 0.0);
    let best_r = r_k.unwrap_or(r_nk);
    let best_l = l_k.unwrap_or(l_nk);
    let total = best_r + best_l;
    let min_eye = best_r.min(best_l);

    if total >= 19.0 && min_eye >= 9.0 {
        ("CT_TL_01", 1)
    } else if total >= 16.0 && min_eye >= 6.0 {
        ("CT_TL_02", 2)
    } else if total >= 14.0 && min_eye >= 5.0 {
        ("CT_TL_03", 3)
    } else if total >= 11.0 && min_eye >= 3.0 {
        ("CT_TL_04", 4)
    } else {
        ("CT_TL_05", 5)
    }
}

pub fn auto_classify_hearing(
    r_norm: &str,
    r_whisp: &str,
    l_norm: &str,
    l_whisp: &str,
) -> (&'static str, i64) {
    let rn = parse_first_number(r_norm).unwrap_or(5.0);
    let ln = parse_first_number(l_norm).unwrap_or(5.0);
    let rw = parse_first_number(r_whisp).unwrap_or(0.5);
    let lw = parse_first_number(l_whisp).unwrap_or(0.5);
    let min_norm = rn.min(ln);
    let min_whisp = rw.min(lw);

    if min_norm >= 5.0 && min_whisp >= 0.5 {
        ("CT_TT_01", 1)
    } else if min_norm >= 4.0 {
        ("CT_TT_02", 2)
    } else if min_norm >= 2.5 {
        ("CT_TT_03", 3)
    } else if min_norm >= 1.0 {
        ("CT_TT_04", 4)
    } else {
        ("CT_TT_05", 5)
    }
}

pub fn auto_classify_bp(bp_str: &str, age: Option<i32>) -> (&'static str, i64) {
    let Some((sys, dia)) = parse_bp_values(bp_str) else {
        return ("CT_HA_01", 1);
    };
    let is_30_plus = age.unwrap_or(25) >= 30;
    if !is_30_plus {
        if sys > 140 || dia > 95 {
            ("CT_HA_04", 4)
        } else if sys < 100 || dia < 60 {
            ("CT_HA_05", 4)
        } else if sys >= 136 || dia >= 90 {
            ("CT_HA_03", 3)
        } else if sys >= 126 || dia > 85 {
            ("CT_HA_02", 2)
        } else {
            ("CT_HA_01", 1)
        }
    } else if sys > 150 || dia >= 96 {
        ("CT_HA_08", 4)
    } else if sys >= 140 || dia >= 90 {
        ("CT_HA_07", 3)
    } else {
        ("CT_HA_06", 1)
    }
}

pub fn auto_classify_pulse(mach_str: &str) -> (&'static str, i64) {
    let Some(bpm) = parse_first_number(mach_str) else {
        return ("CT_MC_01", 1);
    };
    let b = bpm as i32;
    if (60..=75).contains(&b) {
        ("CT_MC_01", 1)
    } else if (76..=85).contains(&b) {
        ("CT_MC_02", 2)
    } else if (86..=95).contains(&b) || (55..=59).contains(&b) {
        ("CT_MC_03", 3)
    } else {
        ("CT_MC_04", 4)
    }
}

pub fn auto_classify_bmi(bmi_str: &str, height_str: &str, weight_str: &str) -> (&'static str, i64) {
    let bmi = parse_first_number(bmi_str).or_else(|| {
        let h = parse_first_number(height_str)?;
        let w = parse_first_number(weight_str)?;
        if h > 30.0 && w > 2.0 {
            let hm = h / 100.0;
            Some(w / (hm * hm))
        } else {
            None
        }
    });
    let Some(val) = bmi else {
        return ("CT_BP_01", 1);
    };
    if val >= 28.0 {
        ("CT_BP_04", 4)
    } else if val >= 26.0 {
        ("CT_BP_03", 3)
    } else if val >= 25.0 {
        ("CT_BP_02", 2)
    } else {
        ("CT_BP_01", 1)
    }
}

pub fn resolve_classification_score(
    cls_id: &str,
    selected: &std::collections::HashMap<String, String>,
    auto_scores: &std::collections::HashMap<String, (String, i64)>,
    depth: u8,
) -> i64 {
    if depth > 5 {
        return 1;
    }
    let cfg = phan_loai_config();
    let Some(c) = cfg.classifications.iter().find(|x| x.id == cls_id) else {
        return 1;
    };
    if let Some(sc) = c.score {
        return sc;
    }
    if let Some(ref ref_did) = c.ref_disease_id {
        let base = if let Some((_, sc)) = auto_scores.get(ref_did.as_str()) {
            *sc
        } else if let Some(ref_cls_id) = selected.get(ref_did.as_str()) {
            resolve_classification_score(ref_cls_id, selected, auto_scores, depth + 1)
        } else {
            1
        };
        let delta = parse_adjust_delta(&c.adjust);
        return (base + delta).clamp(1, 6);
    }
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phan_loai_config_parses() {
        let cfg = phan_loai_config();
        assert!(!cfg.specialties.is_empty());
        assert!(!cfg.diseases.is_empty());
        assert!(!cfg.classifications.is_empty());
    }

    #[test]
    fn test_auto_classification_and_ref_rules() {
        assert_eq!(auto_classify_vision("10/10", "10/10", "", ""), ("CT_TL_01", 1));
        assert_eq!(auto_classify_vision("8/10", "8/10", "", ""), ("CT_TL_02", 2));
        assert_eq!(auto_classify_bp("145/95", Some(25)), ("CT_HA_04", 4));
        assert_eq!(auto_classify_bp("120/80", Some(25)), ("CT_HA_01", 1));
        assert_eq!(auto_classify_pulse("100"), ("CT_MC_04", 4));
        assert_eq!(auto_classify_hearing("5m", "0.5m", "5m", "0.5m"), ("CT_TT_01", 1));
        assert_eq!(auto_classify_bmi("26.5", "", ""), ("CT_BP_03", 3));

        let mut selected = std::collections::HashMap::new();
        let mut auto_scores = std::collections::HashMap::new();
        auto_scores.insert("BT_MAT_THI_LUC".to_string(), ("CT_TL_02".to_string(), 2));
        // CT_SGM_02: "Theo thị lực chuẩn hạ 1 loại", adjust: "+1 loại" => 2 + 1 = 3
        assert_eq!(resolve_classification_score("CT_SGM_02", &selected, &auto_scores, 0), 3);
        // CT_MT_02: "Theo thị lực chuẩn 2 mắt", adjust: 0 => 2
        assert_eq!(resolve_classification_score("CT_MT_02", &selected, &auto_scores, 0), 2);
        selected.insert("BT_RHM_MAT_RANG".to_string(), "CT_MR_02".to_string());
        let mr_score = resolve_classification_score("CT_MR_02", &selected, &auto_scores, 0);
        assert_eq!(resolve_classification_score("CT_VQR_05", &selected, &auto_scores, 0), mr_score);
    }
}
