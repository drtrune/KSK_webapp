use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::FromRow;

/// Danh sách các Role trong hệ thống KSK (Mục 3)
pub const VALID_ROLES: &[&str] = &[
    "admin",
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
];

/// Danh sách các cột chuẩn xuất CSV theo Quyết định 3176/QĐ-BYT & Thông tư 25/2026/TT-BYT (Mục 4)
pub const CSV_EXPORT_COLUMNS: &[&str] = &[
    // 1-15: Hành chính
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
    // 16-21: Thông tin KCB
    "DOI_TUONG",
    "NGUON_CHI_TRA",
    "NOI_LAM_VIEC_HOC_TAP",
    "DIEN_THOAI",
    "LY_DO_VV",
    "MA_LOAI_KCB",
    // 22-51: Tiền sử bệnh (0: Không, 1: Có)
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
    // 52-89: Thể lực & Lâm sàng
    "CHIEU_CAO",
    "CAN_NANG",
    "CHI_SO_BMI",
    "MACH",
    "HUYET_AP",
    "KHAM_THE_LUC_PL",
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
    "KET_QUA_KHAM_NGOAI_KHOA",
    "KHAM_NGOAI_KHOA_PL",
    "KET_QUA_KHAM_DA_LIEU",
    "KHAM_DA_LIEU_PL",
    "KET_QUA_KHAM_SAN_PHU_KHOA",
    "KHAM_SAN_PHU_KHOA_PL",
    "KHONG_KINH_MAT_PHAI",
    "KHONG_KINH_MAT_TRAI",
    "CO_KINH_MAT_PHAI",
    "CO_KINH_MAT_TRAI",
    "BENH_KHAC_MAT",
    "KHAM_MAT_PL",
    "TAI_TRAI_NOI_THUONG",
    "TAI_TRAI_NOI_THAM",
    "TAI_PHAI_NOI_THUONG",
    "TAI_PHAI_NOI_THAM",
    "BENH_KHAC_TAI_MUI_HONG",
    "KHAM_TAI_MUI_HONG_PL",
    "HAM_TREN",
    "HAM_DUOI",
    "BENH_KHAC_RANG_HAM_MAT",
    "KHAM_RANG_HAM_MAT_PL",
    "PHAN_LOAI_SK",
    "KET_LUAN_BENH",
    "CAC_BENH_TAT_NEU_CO",
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
        "clinical_noi" => FIELDS_BS_NOI,
        "clinical_ngoai" => FIELDS_BS_NGOAI,
        "clinical_dalieu" => FIELDS_BS_DALIEU,
        "clinical_san" => FIELDS_BS_SAN,
        "clinical_mat" => FIELDS_BS_MAT,
        "clinical_tmh" => FIELDS_BS_TMH,
        "clinical_rhm" => FIELDS_BS_RHM,
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
        if let Some(allowed_fields) = filter_fields {
            if !allowed_fields.contains(&k.as_str()) {
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
    if let (Some(cc), Some(cn)) = (
        base.get("CHIEU_CAO").and_then(|v| v.as_str()).and_then(|s| s.replace(',', ".").parse::<f64>().ok()),
        base.get("CAN_NANG").and_then(|v| v.as_str()).and_then(|s| s.replace(',', ".").parse::<f64>().ok()),
    ) {
        if cc > 30.0 && cn > 2.0 {
            let height_m = cc / 100.0;
            let bmi = cn / (height_m * height_m);
            base.insert("CHI_SO_BMI".to_string(), Value::String(format!("{:.2}", bmi)));
        }
    }

    base
}

// ============================================================================
// BẢNG USERS & AUTHENTICATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
            payload,
            updated_by: row.updated_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveRecordRequest {
    pub id: Option<i64>,
    pub ma_tiep_nhan: Option<String>,
    pub payload: Map<String, Value>,
    pub section: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RecordFilterQuery {
    pub q: Option<String>,
    pub status: Option<i64>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

// ============================================================================
// BẢNG CATALOGS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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

#[derive(Debug, Serialize)]
pub struct DbInfo {
    pub current_db_path: String,
    pub file_size_bytes: u64,
    pub records_count: i64,
    pub users_count: i64,
    pub catalogs_count: i64,
    pub available_db_files: Vec<String>,
}
