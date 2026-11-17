#![allow(dead_code)]

/// Định nghĩa 5 mẫu phiếu khám sức khỏe độc lập.
/// - `TREN_18`: Khám sức khỏe người từ đủ 18 tuổi trở lên (dùng cây danh mục 22 bệnh lý)
/// - `TU_6_DUOI_18`: Khám sức khỏe từ 6 đến dưới 18 tuổi (Import_KSK_6 den Duoi 18.xlsm)
/// - `DUOI_6`: Khám sức khỏe trẻ dưới 6 tuổi (Import_KSK_Duoi 6.xlsm)
/// - `MAU_GIAO`: Phiếu 4 - Kiểm tra sức khỏe Mẫu giáo (CSDLKSK)
/// - `HS_PHO_THONG`: Phiếu 5 - Kiểm tra sức khỏe Học sinh phổ thông (CSDLKSK)

pub const ALL_TEMPLATES: &[(&str, &str, &str)] = &[
    ("TREN_18", "Trên 18 tuổi", "KSK từ đủ 18 tuổi trở lên (có danh mục bệnh)"),
    ("LAI_XE", "Khám lái xe", "Giấy KSK người lái xe, điều khiển xe máy chuyên dùng"),
    ("TU_6_DUOI_18", "6 - dưới 18 tuổi", "KSK từ 6 đến dưới 18 tuổi"),
    ("DUOI_6", "Dưới 6 tuổi", "KSK trẻ em dưới 6 tuổi"),
    ("MAU_GIAO", "Mẫu giáo (Phiếu 4)", "Kiểm tra SK Mẫu giáo - CSDLKSK"),
    ("HS_PHO_THONG", "HS Phổ thông (Phiếu 5)", "Kiểm tra SK Học sinh phổ thông - CSDLKSK"),
];

pub fn template_short_label(code: &str) -> &'static str {
    for &(c, short, _) in ALL_TEMPLATES {
        if c == code {
            return short;
        }
    }
    "Trên 18 tuổi"
}

pub fn template_full_label(code: &str) -> &'static str {
    for &(c, _, full) in ALL_TEMPLATES {
        if c == code {
            return full;
        }
    }
    "KSK từ đủ 18 tuổi trở lên"
}

pub fn template_prefix(code: &str) -> &'static str {
    match code {
        "LAI_XE" => "LX_",
        "TU_6_DUOI_18" => "U18_",
        "DUOI_6" => "D6_",
        "MAU_GIAO" => "MG_",
        "HS_PHO_THONG" => "HS_",
        _ => "",
    }
}

pub fn namespaced_key(template_code: &str, raw_key: &str) -> String {
    let p = template_prefix(template_code);
    if p.is_empty() {
        raw_key.to_string()
    } else {
        format!("{}{}", p, raw_key)
    }
}

pub fn parse_template_list(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in raw.split(',') {
        let t = part.trim();
        if !t.is_empty()
            && ALL_TEMPLATES.iter().any(|&(c, _, _)| c == t)
            && !out.iter().any(|x| x == t)
        {
            out.push(t.to_string());
        }
    }
    if out.is_empty() {
        out.push("TREN_18".to_string());
    }
    out
}

pub fn format_template_list(list: &[String]) -> String {
    if list.is_empty() {
        "TREN_18".to_string()
    } else {
        list.join(",")
    }
}

pub fn toggle_template_in_list(raw: &str, code: &str) -> String {
    let mut list = parse_template_list(raw);
    if let Some(pos) = list.iter().position(|x| x == code) {
        if list.len() > 1 {
            list.remove(pos);
        }
    } else {
        list.push(code.to_string());
    }
    format_template_list(&list)
}

pub fn has_under18_template(raw: &str) -> bool {
    parse_template_list(raw)
        .iter()
        .any(|c| c != "TREN_18")
}

#[derive(Clone, Copy)]
pub enum TemplateFieldKind {
    Text,
    Date,
    Select(&'static [(&'static str, &'static str)]),
    Icd10,
}

#[derive(Clone, Copy)]
pub struct TemplateField {
    pub key: &'static str,
    pub label: &'static str,
    pub placeholder: &'static str,
    pub default_normal: &'static str,
    pub kind: TemplateFieldKind,
}

#[derive(Clone, Copy)]
pub struct TemplateGroup {
    pub title: &'static str,
    pub fields: &'static [TemplateField],
}

const OPT_KHONG_CO: &[(&str, &str)] = &[("Không", "Không"), ("Có", "Có")];
const OPT_CO_KHONG: &[(&str, &str)] = &[("Có", "Có"), ("Không", "Không")];
const OPT_CO_KHONG_RO: &[(&str, &str)] = &[("Có", "Có"), ("Không", "Không"), ("Không rõ", "Không rõ")];
const OPT_BT_KBT: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Không bình thường", "Không bình thường")];
const OPT_NHIET_DO: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Sốt", "Sốt"), ("Hạ thân nhiệt", "Hạ thân nhiệt")];
const OPT_MACH: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Nhanh", "Nhanh"), ("Chậm", "Chậm")];
const OPT_NHIP_THO: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Thở nhanh", "Thở nhanh"), ("Thở chậm", "Thở chậm")];
const OPT_VONG_DAU: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Đầu to", "Đầu to"), ("Đầu nhỏ", "Đầu nhỏ")];
const OPT_DINH_DUONG: &[(&str, &str)] = &[
    ("Bình thường", "Bình thường"),
    ("Phù dinh dưỡng", "Phù dinh dưỡng"),
    ("Dấu hiệu thiếu máu", "Dấu hiệu thiếu máu"),
    ("Dấu hiệu còi xương", "Dấu hiệu còi xương"),
    ("Suy dinh dưỡng", "Suy dinh dưỡng"),
    ("Thừa cân / béo phì", "Thừa cân / béo phì"),
];
const OPT_MAU_DA: &[(&str, &str)] = &[
    ("Hồng hào", "Hồng hào"),
    ("Nhợt", "Nhợt"),
    ("Tím", "Tím"),
    ("Vàng", "Vàng"),
    ("Sạm da", "Sạm da"),
];
const OPT_LONG_BAN_TAY: &[(&str, &str)] = &[
    ("Bình thường (không nhợt)", "Bình thường (không nhợt)"),
    ("Không bình thường (nhợt)", "Không bình thường (nhợt)"),
];
const OPT_VAN_DONG_CO: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Giới hạn", "Giới hạn")];
const OPT_VI_TRI_MAT: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Hai mắt xa nhau", "Hai mắt xa nhau")];
const OPT_MI_MAT: &[(&str, &str)] = &[
    ("Bình thường", "Bình thường"),
    ("Sưng/đỏ", "Sưng/đỏ"),
    ("Chảy ghèn/mủ", "Chảy ghèn/mủ"),
];
const OPT_MUI: &[(&str, &str)] = &[
    ("Bình thường", "Bình thường"),
    ("Mũi to, dày", "Mũi to, dày"),
    ("Bất sản xương mũi", "Bất sản xương mũi"),
];
const OPT_MIENG: &[(&str, &str)] = &[
    ("Bình thường", "Bình thường"),
    ("Sứt môi, chẻ vòm", "Sứt môi, chẻ vòm"),
];
const OPT_LUOI: &[(&str, &str)] = &[("Bình thường", "Bình thường"), ("Lưỡi to bè", "Lưỡi to bè")];
const OPT_PHAN_LOAI_SK: &[(&str, &str)] = &[
    ("Loại I", "Loại I (Rất khỏe)"),
    ("Loại II", "Loại II (Khỏe)"),
    ("Loại III", "Loại III (Trung bình)"),
    ("Loại IV", "Loại IV (Yếu)"),
    ("Loại V", "Loại V (Rất yếu)"),
];

// ==================== 1. MẪU GIÁO (PHIẾU 4 - CSDLKSK) ====================
const MG_HISTORY_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "II. Tiền sử bản thân & gia đình",
        fields: &[
            TemplateField { key: "TSBT_MAC_BENH", label: "Bản thân có bệnh tiền sử không?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_MA_BENH", label: "Mã/Tên bệnh bản thân (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "TSGD_MAC_BENH", label: "Gia đình có bệnh tiền sử không?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSGD_MA_BENH", label: "Mã/Tên bệnh gia đình (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "TS_TIEP_XUC_LAO", label: "Tiền sử tiếp xúc người bệnh lao", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "VII. Đánh giá tiêm chủng",
        fields: &[
            TemplateField { key: "TIEM_CHUNG_BCG_SS", label: "Lao (sơ sinh)", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "TIEM_CHUNG_VGB_SS_MUI1", label: "Viêm gan B mũi 1 (sơ sinh)", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "TIEM_CHUNG_DAY_DU_THEO_DO_TUOI", label: "Tiêm chủng đầy đủ theo độ tuổi", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
        ],
    },
];

const D6_PHYSICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "IV. Đánh giá dấu hiệu sinh tồn",
        fields: &[
            TemplateField { key: "NHIET_DO", label: "Nhiệt độ (°C)", placeholder: "36.8", default_normal: "36.8", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDHST_NHIET_DO", label: "Đánh giá nhiệt độ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_NHIET_DO) },
            TemplateField { key: "MACH", label: "Mạch (lần/phút)", placeholder: "90", default_normal: "90", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDHST_MACH", label: "Đánh giá mạch", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MACH) },
            TemplateField { key: "NHIP_THO", label: "Nhịp thở (lần/phút)", placeholder: "25", default_normal: "25", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDHST_NHIP_THO", label: "Đánh giá nhịp thở", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_NHIP_THO) },
        ],
    },
    TemplateGroup {
        title: "V. Đánh giá dinh dưỡng (Dưới 6 tuổi)",
        fields: &[
            TemplateField { key: "CHIEU_DAI", label: "Chiều dài / Chiều cao (cm)", placeholder: "95", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CHIEU_DAI_TUOI_SD", label: "Chiều dài/Tuổi (SD)", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG", label: "Cân nặng (kg)", placeholder: "14.5", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG_TUOI_SD", label: "Cân nặng/Tuổi (SD)", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "VONG_DAU", label: "Vòng đầu (cm)", placeholder: "48", default_normal: "48", kind: TemplateFieldKind::Text },
            TemplateField { key: "DG_VONG_DAU", label: "Đánh giá vòng đầu", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_VONG_DAU) },
            TemplateField { key: "CHU_VI_VONG_CANH_TAY", label: "Chu vi vòng cánh tay (mm)", placeholder: "145", default_normal: "145", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDD_BINH_THUONG", label: "Đánh giá dinh dưỡng: Bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "PHU_DINH_DUONG", label: "Phù dinh dưỡng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "DGDD_THIEU_MAU", label: "Dấu hiệu thiếu máu", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "DGDD_COI_XUONG", label: "Dấu hiệu còi xương", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "SUY_DINH_DUONG", label: "Suy dinh dưỡng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "THUA_CAN_BEO_PHI", label: "Thừa cân / béo phì", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "VI. Tinh thần - Vận động",
        fields: &[
            TemplateField { key: "PT_TTBT_THEO_DO_TUOI", label: "Phát triển tinh thần bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "PT_VDBT_THEO_DO_TUOI", label: "Phát triển vận động bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "NGUY_CO_TU_KY", label: "Nguy cơ tự kỷ (16-30 tháng)", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
];

const MG_PHYSICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "IV. Đánh giá dấu hiệu sinh tồn",
        fields: &[
            TemplateField { key: "NHIET_DO", label: "Nhiệt độ (°C)", placeholder: "36.8", default_normal: "36.8", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDHST_NHIET_DO", label: "Đánh giá nhiệt độ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_NHIET_DO) },
            TemplateField { key: "MACH", label: "Mạch (lần/phút)", placeholder: "90", default_normal: "90", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDHST_MACH", label: "Đánh giá mạch", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MACH) },
            TemplateField { key: "NHIP_THO", label: "Nhịp thở (lần/phút)", placeholder: "25", default_normal: "25", kind: TemplateFieldKind::Text },
            TemplateField { key: "DGDHST_NHIP_THO", label: "Đánh giá nhịp thở", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_NHIP_THO) },
        ],
    },
    TemplateGroup {
        title: "V. Đánh giá dinh dưỡng",
        fields: &[
            TemplateField { key: "CHIEU_DAI", label: "Chiều dài / Chiều cao (cm)", placeholder: "95", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CHIEU_DAI_TUOI_SD", label: "Chiều dài/Tuổi (SD)", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG", label: "Cân nặng (kg)", placeholder: "14.5", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG_TUOI_SD", label: "Cân nặng/Tuổi (SD)", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "VONG_DAU", label: "Vòng đầu (cm)", placeholder: "48", default_normal: "48", kind: TemplateFieldKind::Text },
            TemplateField { key: "DG_VONG_DAU", label: "Đánh giá vòng đầu", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_VONG_DAU) },
            TemplateField { key: "CHU_VI_VONG_CANH_TAY", label: "Chu vi vòng cánh tay (mm)", placeholder: "145", default_normal: "145", kind: TemplateFieldKind::Text },
            TemplateField { key: "TINH_TRANG_DINH_DUONG", label: "Tình trạng dinh dưỡng", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_DINH_DUONG) },
        ],
    },
    TemplateGroup {
        title: "VI. Tinh thần - Vận động",
        fields: &[
            TemplateField { key: "PT_TTBT_THEO_DO_TUOI", label: "Phát triển tinh thần bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "PT_VDBT_THEO_DO_TUOI", label: "Phát triển vận động bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "NGUY_CO_TU_KY", label: "Nguy cơ tự kỷ (16-30 tháng)", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
];

const MG_CLINICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Toàn trạng & 2.1. Đầu - cổ",
        fields: &[
            TemplateField { key: "MAU_SAC_DA", label: "Màu sắc da", placeholder: "", default_normal: "Hồng hào", kind: TemplateFieldKind::Select(OPT_MAU_DA) },
            TemplateField { key: "LONG_BAN_TAY", label: "Lòng bàn tay", placeholder: "", default_normal: "Bình thường (không nhợt)", kind: TemplateFieldKind::Select(OPT_LONG_BAN_TAY) },
            TemplateField { key: "HINH_DANG_DAU", label: "Kích thước và hình dạng đầu", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "VAN_DONG_CO", label: "Vận động cổ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_VAN_DONG_CO) },
            TemplateField { key: "KHOI_BAT_THUONG_DAU_CO", label: "Khối bất thường đầu cổ", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "2.2. Khám Mắt & 2.3. Khám Tai",
        fields: &[
            TemplateField { key: "VI_TRI_HAI_MAT", label: "Vị trí 2 mắt", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_VI_TRI_MAT) },
            TemplateField { key: "MI_MAT_KET_MAC", label: "Mí mắt và kết mạc", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MI_MAT) },
            TemplateField { key: "LAC_MAT", label: "Lác mắt", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "DAP_UNG_AM_THANH", label: "Đáp ứng với âm thanh", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "KHOI_SUNG_SAU_TAI", label: "Có khối sưng sau tai", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "CHAY_MU_NUOC_TAI", label: "Dấu hiệu chảy mủ, nước tai", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "2.4. Mũi - Họng & 2.5. Miệng - Răng",
        fields: &[
            TemplateField { key: "HINH_DANG_MUI", label: "Hình dạng mũi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MUI) },
            TemplateField { key: "CHAY_NUOC_MUI", label: "Chảy nước mũi", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "NGHET_MUI", label: "Nghẹt mũi", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "HINH_DANG_MIENG", label: "Hình dạng miệng", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MIENG) },
            TemplateField { key: "HINH_DANG_LUOI", label: "Hình dạng lưỡi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_LUOI) },
            TemplateField { key: "DINH_THANG_LUOI", label: "Dính thắng lưỡi", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "CAM_NHO_TUT_VE_SAU", label: "Cằm nhỏ, tụt về sau", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "SAU_MANG_BAM_LO", label: "Vết sâu, mảng bám, lỗ trên răng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "3. Cơ xương khớp & Thần kinh - Tâm thần",
        fields: &[
            TemplateField { key: "VAN_DONG_KHONG_DOI_XUNG", label: "Vận động không đối xứng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "KIEM_TRA_LUNG_COT_SONG", label: "Kiểm tra lưng, cột sống", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "QUAN_SAT_DANG_DI", label: "Quan sát dáng đi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "BIEU_HIEN_LO_AU_MAT_NGU", label: "Biểu hiện buồn bã, lo âu, mất ngủ", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "GHI_RO_LO_AU_MAT_NGU", label: "Ghi rõ biểu hiện buồn bã, lo âu", placeholder: "Ghi rõ nếu có...", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "HANH_VI_BAT_THUONG", label: "Hành vi bất thường / tự gây hại", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "GHI_RO_HANH_VI_BAT_THUONG", label: "Ghi rõ hành vi bất thường", placeholder: "Ghi rõ nếu có...", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const MG_CONCLUSION_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "IX. Kết luận và tư vấn",
        fields: &[
            TemplateField { key: "BINH_THUONG", label: "Sức khỏe bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "NGUY_CO_MAC_LAO", label: "Có nguy cơ mắc lao", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "VAN_DE_SUC_KHOE", label: "Có vấn đề về sức khỏe", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "KET_LUAN_BENH", label: "Mã/Tên bệnh kết luận (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "CAN_TU_VAN_CHUYEN_CK", label: "Cần tư vấn và chuyển khám CK", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "GHI_RO_CHUYEN_CSKCB", label: "Ghi rõ cơ sở / chỉ định chuyển", placeholder: "Ghi rõ cơ sở chuyển khám...", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const D6_CONCLUSION_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "IX. Kết luận và tư vấn (Dưới 6 tuổi)",
        fields: &[
            TemplateField { key: "BINH_THUONG", label: "Sức khỏe bình thường", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG) },
            TemplateField { key: "NGUY_CO_MAC_LAO", label: "Có nguy cơ mắc lao", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "VAN_DE_SUC_KHOE", label: "Có vấn đề về sức khỏe", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "KET_LUAN_BENH", label: "Kết luận bệnh (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "GHI_RO_VAN_DE_SUC_KHOE", label: "Ghi rõ vấn đề sức khỏe", placeholder: "Ghi rõ nếu có...", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "HEN_KHAM_LAN_SAU", label: "Hẹn khám lần sau", placeholder: "dd/mm/yyyy", default_normal: "", kind: TemplateFieldKind::Date },
            TemplateField { key: "CHUYEN_CSKCB", label: "Chuyển cơ sở KCB", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
];

// ==================== 2. DƯỚI 6 TUỔI (Import_KSK_Duoi 6.xlsm) ====================
const D6_CLINICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Toàn trạng & Đầu - Cổ",
        fields: &[
            TemplateField { key: "MAU_SAC_DA", label: "Màu sắc da", placeholder: "", default_normal: "Hồng hào", kind: TemplateFieldKind::Select(OPT_MAU_DA) },
            TemplateField { key: "LONG_BAN_TAY", label: "Lòng bàn tay", placeholder: "", default_normal: "Bình thường (không nhợt)", kind: TemplateFieldKind::Select(OPT_LONG_BAN_TAY) },
            TemplateField { key: "THOP", label: "Thóp (trẻ nhỏ còn thóp)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "HINH_DANG_DAU", label: "Kích thước và hình dạng đầu", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "VAN_DONG_CO", label: "Vận động cổ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_VAN_DONG_CO) },
            TemplateField { key: "KHOI_BAT_THUONG_DAU_CO", label: "Khối bất thường đầu cổ", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "2. Khám Mắt",
        fields: &[
            TemplateField { key: "VI_TRI_HAI_MAT", label: "Vị trí 2 mắt", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_VI_TRI_MAT) },
            TemplateField { key: "MI_MAT_KET_MAC", label: "Mí mắt và kết mạc", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MI_MAT) },
            TemplateField { key: "LAC_MAT", label: "Lác mắt", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "DONG_TU", label: "Đồng tử (kích thước, phản xạ)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
        ],
    },
    TemplateGroup {
        title: "3. Khám Tai",
        fields: &[
            TemplateField { key: "TAI_MANG_NHI", label: "Tai và màng nhĩ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "DAP_UNG_AM_THANH", label: "Đáp ứng với âm thanh", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "KHOI_SUNG_SAU_TAI", label: "Có khối sưng sau tai", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "CHAY_MU_NUOC_TAI", label: "Chảy mủ, nước tai", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "4. Khám Mũi - Họng",
        fields: &[
            TemplateField { key: "HINH_DANG_MUI", label: "Hình dạng mũi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MUI) },
            TemplateField { key: "CHAY_NUOC_MUI", label: "Chảy nước mũi", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "NGHET_MUI", label: "Nghẹt mũi", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "HONG", label: "Họng", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
        ],
    },
    TemplateGroup {
        title: "5. Khám Miệng - Răng",
        fields: &[
            TemplateField { key: "HINH_DANG_MIENG", label: "Hình dạng miệng", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_MIENG) },
            TemplateField { key: "RANG_SUA_SO_SINH", label: "Răng sữa sơ sinh", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "HINH_DANG_LUOI", label: "Hình dạng lưỡi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_LUOI) },
            TemplateField { key: "DINH_THANG_LUOI", label: "Dính thắng lưỡi", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "NAM_MIENG", label: "Nấm miệng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "CAM_NHO_TUT_VE_SAU", label: "Cằm nhỏ, tụt về sau", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "SAU_MANG_BAM_LO", label: "Vết sâu, mảng bám, lỗ trên răng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "6. Khám Hô hấp",
        fields: &[
            TemplateField { key: "NHIP_THO_KHONG_DEU", label: "Nhịp thở không đều", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "THO_RUT_LOM_LONG_NGUC", label: "Thở rút lõm lồng ngực", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TIENG_THO_BAT_THUONG", label: "Tiếng thở bất thường", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "SUY_HO_HAP", label: "Dấu hiệu suy hô hấp", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "NGHE_PHOI", label: "Nghe phổi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
        ],
    },
    TemplateGroup {
        title: "7. Khám Tim mạch",
        fields: &[
            TemplateField { key: "VI_TRI_MOM_TIM", label: "Vị trí mỏm tim", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "MACH_NGOAI_VI", label: "Mạch ngoại vi (quay-bẹn)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "TIENG_TIM", label: "Nghe tim (loạn nhịp, tiếng thổi)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
        ],
    },
    TemplateGroup {
        title: "8. Khám Bụng & Cơ quan sinh dục",
        fields: &[
            TemplateField { key: "HINH_DANG_BUNG_RON", label: "Hình dáng bụng, rốn", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "GAN_LACH_TO", label: "Gan, lách to", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "KHOI_BAT_THUONG", label: "Khối bất thường ở bụng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "LO_HAU_MON", label: "Lỗ hậu môn", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "CO_QUAN_SINH_DUC_NGOAI", label: "Cơ quan sinh dục ngoài", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
        ],
    },
    TemplateGroup {
        title: "9. Cơ xương khớp & Phản xạ - Thần kinh",
        fields: &[
            TemplateField { key: "VAN_DONG_KHONG_DOI_XUNG", label: "Vận động không đối xứng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "PHAN_XA_BU", label: "Phản xạ bú", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "PHAN_XA_NAM", label: "Phản xạ nắm", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "PHAN_XA_MORO", label: "Phản xạ Moro", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "TRUONG_LUC_CO", label: "Trương lực cơ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "KHOP_HANG", label: "Khớp háng", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "PHAN_XA_CO", label: "Phản xạ cơ", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "KIEM_TRA_LUNG_COT_SONG", label: "Kiểm tra lưng, cột sống", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "TU_CHI_KHOP", label: "Khám tứ chi và khớp", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "QUAN_SAT_DANG_DI", label: "Quan sát dáng đi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
        ],
    },
];

// ==================== 3. HỌC SINH PHỔ THÔNG (PHIẾU 5) & 6 ĐẾN DƯỚI 18 ====================
const HS_HISTORY_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Tiền sử gia đình",
        fields: &[
            TemplateField { key: "TSGD_MAC_BENH", label: "Gia đình mắc bệnh bẩm sinh / truyền nhiễm?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSGD_MA_BENH", label: "Tên / Mã bệnh gia đình (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
        ],
    },
    TemplateGroup {
        title: "2a. Tiền sử bản thân - Sản khoa",
        fields: &[
            TemplateField { key: "TSBT_SAN_KHOA", label: "Tiền sử bản thân - Sản khoa", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "MA_BENH_SAN_KHOA_KHONG_BT", label: "Mã bệnh sản khoa không BT (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
        ],
    },
    TemplateGroup {
        title: "2b. Tiền sử tiêm chủng",
        fields: &[
            TemplateField { key: "TC_BCG", label: "BCG (Lao)", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_BH_HG_UV", label: "Bạch hầu - Ho gà - Uốn ván", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_SOI", label: "Sởi", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_BAI_LIET", label: "Bại liệt", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_VNNB_B", label: "Viêm não Nhật Bản B", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_VGB", label: "Viêm gan B", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_KHAC_TEN", label: "Loại vắc xin khác (Tên vắc xin)", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "TC_KHAC_TINH_TRANG", label: "Tình trạng tiêm vắc xin khác", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
        ],
    },
    TemplateGroup {
        title: "2c & 2d. Bệnh bẩm sinh / mãn tính & Đang điều trị",
        fields: &[
            TemplateField { key: "TSBT_MAC_BENH", label: "2c. Bản thân mắc bệnh bẩm sinh / mãn tính?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_MA_BENH", label: "Mã/Tên bệnh bẩm sinh, mãn tính (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "DANG_DIEU_TRI_BENH", label: "2d. Hiện tại đang điều trị bệnh gì không?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "MA_BENH_DANG_DT", label: "Mã bệnh đang điều trị (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "GHI_RO_BENH_DANG_DT", label: "Ghi rõ bệnh và thuốc đang điều trị", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const U18_HISTORY_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Tiền sử gia đình",
        fields: &[
            TemplateField { key: "TSGD_MAC_BENH", label: "Gia đình mắc bệnh bẩm sinh / truyền nhiễm?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSGD_MA_BENH", label: "Tên / Mã bệnh gia đình (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
        ],
    },
    TemplateGroup {
        title: "2a. Tiền sử bản thân - Sản khoa",
        fields: &[
            TemplateField { key: "SAN_KHOA", label: "Tiền sử sản khoa", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "SAN_KHOA_KHONG_BT", label: "Sản khoa không bình thường?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "MA_BENH_SAN_KHOA_KHONG_BT", label: "Mã bệnh sản khoa không BT (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
        ],
    },
    TemplateGroup {
        title: "2b. Tiền sử tiêm chủng",
        fields: &[
            TemplateField { key: "TC_BCG", label: "BCG (Lao)", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_BH_HG_UV", label: "Bạch hầu - Ho gà - Uốn ván", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_SOI", label: "Sởi", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_BAI_LIET", label: "Bại liệt", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_VNNB_B", label: "Viêm não Nhật Bản B", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_VGB", label: "Viêm gan B", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
            TemplateField { key: "TC_KHAC_TEN", label: "Loại vắc xin khác (Tên vắc xin)", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "TC_KHAC_TINH_TRANG", label: "Tình trạng tiêm vắc xin khác", placeholder: "", default_normal: "Có", kind: TemplateFieldKind::Select(OPT_CO_KHONG_RO) },
        ],
    },
    TemplateGroup {
        title: "2c & 2d. Bệnh bẩm sinh / mãn tính & Đang điều trị",
        fields: &[
            TemplateField { key: "TSBT_MAC_BENH", label: "2c. Bản thân mắc bệnh bẩm sinh / mãn tính?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_MA_BENH", label: "Mã/Tên bệnh bẩm sinh, mãn tính (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "DANG_DIEU_TRI_BENH", label: "2d. Hiện tại đang điều trị bệnh gì không?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "MA_BENH_DANG_DT", label: "Mã bệnh đang điều trị (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "GHI_RO_BENH_DANG_DT", label: "Ghi rõ bệnh và thuốc đang điều trị", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const HS_PHYSICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "III. Khám thể lực",
        fields: &[
            TemplateField { key: "CHIEU_CAO", label: "1. Chiều cao (cm)", placeholder: "145", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG", label: "2. Cân nặng (kg)", placeholder: "42", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "BMI", label: "3. Chỉ số BMI", placeholder: "20.0", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "MACH", label: "4. Mạch (lần/phút)", placeholder: "80", default_normal: "80", kind: TemplateFieldKind::Text },
            TemplateField { key: "HUYET_AP", label: "5. Huyết áp (mmHg)", placeholder: "100/60", default_normal: "100/60", kind: TemplateFieldKind::Text },
        ],
    },
];

const U18_PHYSICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "III. Khám thể lực (6 - dưới 18 tuổi)",
        fields: &[
            TemplateField { key: "CHIEU_CAO", label: "1. Chiều cao (cm)", placeholder: "145", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG", label: "2. Cân nặng (kg)", placeholder: "42", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "BMI", label: "3. Chỉ số BMI", placeholder: "20.0", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "MACH", label: "4. Mạch (lần/phút)", placeholder: "80", default_normal: "80", kind: TemplateFieldKind::Text },
            TemplateField { key: "HUYET_AP", label: "5. Huyết áp (mmHg)", placeholder: "100/60", default_normal: "100/60", kind: TemplateFieldKind::Text },
            TemplateField { key: "KHAM_THE_LUC_PL", label: "6. Phân loại thể lực", placeholder: "", default_normal: "Loại I", kind: TemplateFieldKind::Select(OPT_PHAN_LOAI_SK) },
        ],
    },
];

const HS_CLINICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Khám Mắt",
        fields: &[
            TemplateField { key: "TL_KHONG_KINH_PHAI", label: "Thị lực không kính - Phải", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "TL_KHONG_KINH_TRAI", label: "Thị lực không kính - Trái", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "TL_CO_KINH_PHAI", label: "Thị lực có kính - Phải", placeholder: "10/10", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "TL_CO_KINH_TRAI", label: "Thị lực có kính - Trái", placeholder: "10/10", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "BENH_VE_MAT", label: "Các bệnh về mắt (nếu có)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "2. Khám Tai - Mũi - Họng",
        fields: &[
            TemplateField { key: "THINH_LUC_THUONG_PHAI", label: "Thính lực nói thường - Phải (m)", placeholder: "5", default_normal: "5", kind: TemplateFieldKind::Text },
            TemplateField { key: "THINH_LUC_THUONG_TRAI", label: "Thính lực nói thường - Trái (m)", placeholder: "5", default_normal: "5", kind: TemplateFieldKind::Text },
            TemplateField { key: "THINH_LUC_THAM_PHAI", label: "Thính lực nói thầm - Phải (m)", placeholder: "0.5", default_normal: "0.5", kind: TemplateFieldKind::Text },
            TemplateField { key: "THINH_LUC_THAM_TRAI", label: "Thính lực nói thầm - Trái (m)", placeholder: "0.5", default_normal: "0.5", kind: TemplateFieldKind::Text },
            TemplateField { key: "BENH_TAI_MUI_HONG", label: "Các bất thường về Tai - Mũi - Họng", placeholder: "", default_normal: "Hiện tại chưa phát hiện bất thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "3. Khám Răng - Hàm - Mặt",
        fields: &[
            TemplateField { key: "RHM_HAM_TREN", label: "Hàm trên", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "RHM_HAM_DUOI", label: "Hàm dưới", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BENH_RANG_HAM_MAT", label: "Các bệnh khác về Răng - Hàm - Mặt", placeholder: "", default_normal: "Hiện tại chưa phát hiện bất thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "4. Cơ xương khớp & Tâm thần - Vận động",
        fields: &[
            TemplateField { key: "VAN_DONG_KHONG_DOI_XUNG", label: "1. Vận động không đối xứng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "KIEM_TRA_LUNG_COT_SONG", label: "2. Kiểm tra lưng, cột sống", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "QUAN_SAT_DANG_DI", label: "3. Quan sát dáng đi", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_BT_KBT) },
            TemplateField { key: "BIEU_HIEN_LO_AU_MAT_NGU", label: "4. Biểu hiện buồn bã, lo âu, mất ngủ kéo dài?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "GHI_RO_LO_AU_MAT_NGU", label: "Ghi rõ biểu hiện buồn bã, lo âu, mất ngủ", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "HANH_VI_BAT_THUONG", label: "5. Hành vi bất thường / nguy cơ tự gây hại?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "GHI_RO_HANH_VI_BAT_THUONG", label: "Ghi rõ hành vi bất thường", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const U18_CLINICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Nhi khoa / Nội khoa (6 đến dưới 18 tuổi)",
        fields: &[
            TemplateField { key: "NHI_KHOA_TUAN_HOAN", label: "Tuần hoàn", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "NHI_KHOA_HO_HAP", label: "Hô hấp", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "NHI_KHOA_TIEU_HOA", label: "Tiêu hóa", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "NHI_KHOA_THAN_TN_SD", label: "Thận - Tiết niệu - Sinh dục", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "NHI_KHOA_THAN_KINH", label: "Thần kinh", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "NHI_KHOA_TAM_THAN", label: "Tâm thần", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "NHI_KHOA_LAM_SANG_KHAC", label: "Khám lâm sàng khác", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "2. Khám Mắt",
        fields: &[
            TemplateField { key: "TL_KHONG_KINH_PHAI", label: "Thị lực không kính - Phải", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "TL_KHONG_KINH_TRAI", label: "Thị lực không kính - Trái", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "TL_CO_KINH_PHAI", label: "Thị lực có kính - Phải", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "TL_CO_KINH_TRAI", label: "Thị lực có kính - Trái", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "BENH_VE_MAT", label: "Các bệnh về mắt (nếu có)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "3. Khám Tai - Mũi - Họng",
        fields: &[
            TemplateField { key: "THINH_LUC_THUONG_PHAI", label: "Thính lực nói thường - Phải (m)", placeholder: "5", default_normal: "5", kind: TemplateFieldKind::Text },
            TemplateField { key: "THINH_LUC_THUONG_TRAI", label: "Thính lực nói thường - Trái (m)", placeholder: "5", default_normal: "5", kind: TemplateFieldKind::Text },
            TemplateField { key: "THINH_LUC_THAM_PHAI", label: "Thính lực nói thầm - Phải (m)", placeholder: "0.5", default_normal: "0.5", kind: TemplateFieldKind::Text },
            TemplateField { key: "THINH_LUC_THAM_TRAI", label: "Thính lực nói thầm - Trái (m)", placeholder: "0.5", default_normal: "0.5", kind: TemplateFieldKind::Text },
            TemplateField { key: "BENH_TAI_MUI_HONG", label: "Các bệnh về Tai - Mũi - Họng", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "4. Khám Răng - Hàm - Mặt",
        fields: &[
            TemplateField { key: "RHM_HAM_TREN", label: "Hàm trên", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "RHM_HAM_DUOI", label: "Hàm dưới", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BENH_RANG_HAM_MAT", label: "Các bệnh về Răng - Hàm - Mặt", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "5. Dịch vụ cận lâm sàng",
        fields: &[
            TemplateField { key: "DICH_VU_CLS", label: "Dịch vụ cận lâm sàng (nếu có)", placeholder: "Ghi kết quả CLS...", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const HS_CONCLUSION_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "V. Kết luận và Tư vấn",
        fields: &[
            TemplateField { key: "KET_LUAN_TINH_TRANG_SK", label: "1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có)", placeholder: "Mô tả tình trạng sức khỏe...", default_normal: "Sức khỏe bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KET_LUAN_BENH", label: "Mã/Tên bệnh kết luận (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "CAN_TU_VAN", label: "2. Cần tư vấn", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "CHUYEN_CSKCB", label: "3. Chuyển cơ sở khám bệnh, chữa bệnh", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "GHI_RO_CHUYEN_CSKCB", label: "Ghi rõ cơ sở / chỉ định chuyển", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const U18_CONCLUSION_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "V. Kết luận (6 - dưới 18 tuổi)",
        fields: &[
            TemplateField { key: "PHAN_LOAI_SK", label: "Phân loại sức khỏe", placeholder: "", default_normal: "Loại I", kind: TemplateFieldKind::Select(OPT_PHAN_LOAI_SK) },
            TemplateField { key: "KET_LUAN_BENH", label: "Kết luận bệnh (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "CAC_VAN_DE_SUC_KHOE", label: "Các vấn đề sức khỏe cần lưu ý", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

// ==================== 5. KHÁM SỨC KHỎE LÁI XE (LAI_XE) ====================
const OPT_THI_TRUONG: &[(&str, &str)] = &[
    ("Bình thường", "Bình thường"),
    ("Hạn chế", "Hạn chế"),
];
const OPT_SAC_GIAC: &[(&str, &str)] = &[
    ("Bình thường", "Bình thường"),
    ("Mù màu toàn bộ", "Mù màu toàn bộ"),
    ("Mù màu đỏ", "Mù màu đỏ"),
    ("Mù màu xanh lá cây", "Mù màu xanh lá cây"),
    ("Mù màu vàng", "Mù màu vàng"),
];
const OPT_XN_AM_DUONG: &[(&str, &str)] = &[
    ("Âm tính", "Âm tính"),
    ("Dương tính", "Dương tính"),
];
const OPT_KL_LAI_XE: &[(&str, &str)] = &[
    ("Đủ điều kiện sức khỏe lái xe hạng", "Đủ điều kiện sức khỏe lái xe hạng"),
    ("Không đủ điều kiện sức khỏe lái xe hạng", "Không đủ điều kiện sức khỏe lái xe hạng"),
    ("Đạt tiêu chuẩn sức khỏe lái xe nhưng yêu cầu khám lại", "Đạt tiêu chuẩn nhưng yêu cầu khám lại"),
];

const LAI_XE_HISTORY_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Tiền sử gia đình & 3. Đang điều trị",
        fields: &[
            TemplateField { key: "TSGD_MAC_BENH", label: "1. Gia đình mắc bệnh truyền nhiễm, tim mạch, ĐTĐ, lao, hen, động kinh, rối loạn tâm thần...?", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSGD_TEN_BENH", label: "Nếu “Có”, ghi cụ thể tên bệnh gia đình (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "TSBT_DANG_DIEU_TRI", label: "3. Hiện đang điều trị bệnh gì? Liệt kê thuốc và liều lượng", placeholder: "Không (hoặc ghi rõ thuốc, liều lượng)...", default_normal: "Không", kind: TemplateFieldKind::Text },
            TemplateField { key: "TSBT_TEN_BENH", label: "Ghi cụ thể tên bệnh bản thân nếu có (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
        ],
    },
    TemplateGroup {
        title: "2a. Tiền sử bản thân (Mục 1 - 10)",
        fields: &[
            TemplateField { key: "TSBT_1", label: "1. Có bệnh hay bị thương trong 5 năm qua", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_2", label: "2. Có bệnh thần kinh hay bị thương ở đầu", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_3", label: "3. Bệnh mắt hoặc giảm thị lực (trừ đeo kính thuốc)", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_4", label: "4. Bệnh ở tai, giảm sức nghe hoặc thăng bằng", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_5", label: "5. Bệnh ở tim, nhồi máu cơ tim, bệnh tim mạch khác", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_6", label: "6. Phẫu thuật can thiệp tim - mạch (van, bắc cầu, stent...)", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_7", label: "7. Tăng huyết áp", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_8", label: "8. Khó thở", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_9", label: "9. Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn tính", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_10", label: "10. Bệnh thận, lọc máu", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
    TemplateGroup {
        title: "2b. Tiền sử bản thân (Mục 11 - 20)",
        fields: &[
            TemplateField { key: "TSBT_11", label: "11. Đái tháo đường hoặc kiểm soát tăng đường huyết", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_12", label: "12. Bệnh tâm thần", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_13", label: "13. Mất ý thức, rối loạn ý thức", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_14", label: "14. Ngất, chóng mặt, mệt mỏi, buồn nôn hoặc ngất xỉu", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_15", label: "15. Bệnh tiêu hóa", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_16", label: "16. Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ ban ngày", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_17", label: "17. Tai biến mạch máu não hoặc liệt", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_18", label: "18. Bệnh hoặc tổn thương cột sống", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_19", label: "19. Sử dụng rượu thường xuyên, liên tục", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
            TemplateField { key: "TSBT_20", label: "20. Sử dụng ma túy và chất gây nghiện", placeholder: "", default_normal: "Không", kind: TemplateFieldKind::Select(OPT_KHONG_CO) },
        ],
    },
];

const LAI_XE_PHYSICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "Chỉ số Sinh hiệu & Thể lực (Lái xe)",
        fields: &[
            TemplateField { key: "CHIEU_CAO", label: "Chiều cao (cm)", placeholder: "168", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CAN_NANG", label: "Cân nặng (kg)", placeholder: "62", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "CHI_SO_BMI", label: "Chỉ số BMI (tự tính)", placeholder: "22.0", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "MACH", label: "Mạch (lần/phút)", placeholder: "75", default_normal: "75", kind: TemplateFieldKind::Text },
            TemplateField { key: "HUYET_AP", label: "Huyết áp (mmHg)", placeholder: "120/80", default_normal: "120/80", kind: TemplateFieldKind::Text },
        ],
    },
];

const LAI_XE_CLINICAL_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "1. Tâm thần & 2. Thần kinh",
        fields: &[
            TemplateField { key: "KLS_TAM_THAN", label: "1. Tâm thần - Nội dung khám", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KL_TAM_THAN", label: "1. Tâm thần - Kết luận", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_TAM_THAN", label: "Bác sĩ khám Tâm thần", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "KLS_THAN_KINH", label: "2. Thần kinh - Nội dung khám", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KL_THAN_KINH", label: "2. Thần kinh - Kết luận", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_THAN_KINH", label: "Bác sĩ khám Thần kinh", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "3. Khám Mắt (Thị lực, Thị trường, Sắc giác)",
        fields: &[
            TemplateField { key: "MAT_KK_MP", label: "Thị lực nhìn xa Không kính - Mắt phải", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "MAT_KK_MT", label: "Thị lực nhìn xa Không kính - Mắt trái", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "MAT_CK_MP", label: "Thị lực nhìn xa Có kính - Mắt phải", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "MAT_CK_MT", label: "Thị lực nhìn xa Có kính - Mắt trái", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "MAT_2M_KK", label: "Thị lực nhìn xa Hai mắt - Không kính", placeholder: "10/10", default_normal: "10/10", kind: TemplateFieldKind::Text },
            TemplateField { key: "MAT_2M_CK", label: "Thị lực nhìn xa Hai mắt - Có kính", placeholder: "", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "MAT_TT_NGANG", label: "Thị trường ngang hai mắt (chiều mũi - thái dương)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_THI_TRUONG) },
            TemplateField { key: "MAT_TT_DUNG", label: "Thị trường đứng (chiều trên - dưới)", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_THI_TRUONG) },
            TemplateField { key: "MAT_SAC_GIAC", label: "Sắc giác", placeholder: "", default_normal: "Bình thường", kind: TemplateFieldKind::Select(OPT_SAC_GIAC) },
            TemplateField { key: "MAT_BENH", label: "Các bệnh về mắt (nếu có, ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "KL_MAT", label: "Kết luận khám Mắt", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_MAT", label: "Bác sĩ khám Mắt", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "4. Khám Tai - Mũi - Họng",
        fields: &[
            TemplateField { key: "TMH_TT_THUONG", label: "Tai trái - Nói thường (m)", placeholder: "5", default_normal: "5", kind: TemplateFieldKind::Text },
            TemplateField { key: "TMH_TT_THAM", label: "Tai trái - Nói thầm (m)", placeholder: "0.5", default_normal: "0.5", kind: TemplateFieldKind::Text },
            TemplateField { key: "TMH_TP_THUONG", label: "Tai phải - Nói thường (m)", placeholder: "5", default_normal: "5", kind: TemplateFieldKind::Text },
            TemplateField { key: "TMH_TP_THAM", label: "Tai phải - Nói thầm (m)", placeholder: "0.5", default_normal: "0.5", kind: TemplateFieldKind::Text },
            TemplateField { key: "TMH_BENH", label: "Các bệnh về Tai - Mũi - Họng (ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "KL_TMH", label: "Kết luận Tai - Mũi - Họng", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_TMH", label: "Bác sĩ khám TMH", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "5. Tim mạch & 6. Hô hấp",
        fields: &[
            TemplateField { key: "KLS_TIM_MACH", label: "5. Tim mạch - Nội dung khám", placeholder: "Nhịp tim đều, T1 T2 rõ", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KL_TIM_MACH", label: "5. Tim mạch - Kết luận", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_TIM_MACH", label: "Bác sĩ khám Tim mạch", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "KLS_HO_HAP", label: "6. Hô hấp - Nội dung khám", placeholder: "Rì rào phế nang êm dịu", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KL_HO_HAP", label: "6. Hô hấp - Kết luận", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_HO_HAP", label: "Bác sĩ khám Hô hấp", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "7. Cơ Xương Khớp & 8. Nội tiết",
        fields: &[
            TemplateField { key: "KLS_CXK", label: "7. Cơ Xương Khớp - Nội dung khám", placeholder: "Vận động các khớp bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KL_CXK", label: "7. Cơ Xương Khớp - Kết luận", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_CXK", label: "Bác sĩ khám Cơ Xương Khớp", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "KLS_NOI_TIET", label: "8. Nội tiết - Nội dung khám", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "KL_NOI_TIET", label: "8. Nội tiết - Kết luận", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_NOI_TIET", label: "Bác sĩ khám Nội tiết", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
    TemplateGroup {
        title: "III. Cận lâm sàng (Ma túy & Nồng độ cồn / Xét nghiệm khác)",
        fields: &[
            TemplateField { key: "XN_MA_TUY", label: "1. Xét nghiệm ma túy (Amphetamin, Marijuana, Morphin, Codein, Heroin)", placeholder: "", default_normal: "Âm tính", kind: TemplateFieldKind::Select(OPT_XN_AM_DUONG) },
            TemplateField { key: "BS_XN_MA_TUY", label: "Bác sĩ / Trưởng bộ phận XN Ma túy", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
            TemplateField { key: "XN_KHAC_KQ", label: "2a. XN Nồng độ cồn / Huyết học / Sinh hóa / X-quang (Kết quả)", placeholder: "Nồng độ cồn: 0 mg/100ml máu...", default_normal: "Bình thường (Nồng độ cồn: Âm tính)", kind: TemplateFieldKind::Text },
            TemplateField { key: "XN_KHAC_KL", label: "2b. Kết luận Xét nghiệm khác", placeholder: "Bình thường", default_normal: "Bình thường", kind: TemplateFieldKind::Text },
            TemplateField { key: "BS_XN_KHAC", label: "Bác sĩ / Trưởng bộ phận XN khác", placeholder: "Họ tên bác sĩ", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

const LAI_XE_CONCLUSION_GROUPS: &[TemplateGroup] = &[
    TemplateGroup {
        title: "IV. Kết luận Sức khỏe Người lái xe",
        fields: &[
            TemplateField { key: "KET_LUAN_SK", label: "Kết luận điều kiện sức khỏe lái xe", placeholder: "", default_normal: "Đủ điều kiện sức khỏe lái xe hạng", kind: TemplateFieldKind::Select(OPT_KL_LAI_XE) },
            TemplateField { key: "HANG_BANG_LAI_KL", label: "Hạng xe đủ điều kiện lái (A1, A, B1, B, C1, C, D...)", placeholder: "B1, B2, C...", default_normal: "B", kind: TemplateFieldKind::Text },
            TemplateField { key: "KET_LUAN_BENH", label: "Tình trạng bệnh, tật (nếu có, ICD-10)", placeholder: "Chọn mã bệnh ICD-10...", default_normal: "", kind: TemplateFieldKind::Icd10 },
            TemplateField { key: "BS_KET_LUAN", label: "Người kết luận (Họ tên Bác sĩ)", placeholder: "BS. Nguyễn Văn A", default_normal: "", kind: TemplateFieldKind::Text },
        ],
    },
];

pub fn get_template_groups(template_code: &str, tab: &str) -> &'static [TemplateGroup] {
    match (template_code, tab) {
        ("LAI_XE", "history") => LAI_XE_HISTORY_GROUPS,
        ("LAI_XE", "physical") => LAI_XE_PHYSICAL_GROUPS,
        ("LAI_XE", "clinical") => LAI_XE_CLINICAL_GROUPS,
        ("LAI_XE", "conclusion") => LAI_XE_CONCLUSION_GROUPS,
        ("MAU_GIAO", "history") | ("DUOI_6", "history") => MG_HISTORY_GROUPS,
        ("MAU_GIAO", "physical") => MG_PHYSICAL_GROUPS,
        ("DUOI_6", "physical") => D6_PHYSICAL_GROUPS,
        ("MAU_GIAO", "clinical") => MG_CLINICAL_GROUPS,
        ("DUOI_6", "clinical") => D6_CLINICAL_GROUPS,
        ("MAU_GIAO", "conclusion") => MG_CONCLUSION_GROUPS,
        ("DUOI_6", "conclusion") => D6_CONCLUSION_GROUPS,
        ("HS_PHO_THONG", "history") => HS_HISTORY_GROUPS,
        ("TU_6_DUOI_18", "history") => U18_HISTORY_GROUPS,
        ("HS_PHO_THONG", "physical") => HS_PHYSICAL_GROUPS,
        ("TU_6_DUOI_18", "physical") => U18_PHYSICAL_GROUPS,
        ("HS_PHO_THONG", "clinical") => HS_CLINICAL_GROUPS,
        ("TU_6_DUOI_18", "clinical") => U18_CLINICAL_GROUPS,
        ("HS_PHO_THONG", "conclusion") => HS_CONCLUSION_GROUPS,
        ("TU_6_DUOI_18", "conclusion") => U18_CONCLUSION_GROUPS,
        _ => &[],
    }
}

/// Tìm định nghĩa trường theo mã mẫu và khóa trường (không có prefix).
pub fn find_field(template_code: &str, raw_key: &str) -> Option<&'static TemplateField> {
    for tab in ["history", "physical", "clinical", "conclusion"] {
        for grp in get_template_groups(template_code, tab) {
            for field in grp.fields {
                if field.key == raw_key {
                    return Some(field);
                }
            }
        }
    }
    None
}

/// Trả về danh sách nhãn các lựa chọn của một trường Select trong mẫu.
pub fn field_options(template_code: &str, raw_key: &str) -> Vec<&'static str> {
    match find_field(template_code, raw_key) {
        Some(f) => match f.kind {
            TemplateFieldKind::Select(opts) => opts.iter().map(|(val, _)| *val).collect(),
            _ => Vec::new(),
        },
        None => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_templates_and_namespacing() {
        assert_eq!(ALL_TEMPLATES.len(), 6);
        assert_eq!(parse_template_list(""), vec!["TREN_18"]);
        let toggled = toggle_template_in_list("TREN_18", "MAU_GIAO");
        assert_eq!(toggled, "TREN_18,MAU_GIAO");
        assert!(has_under18_template(&toggled));
        let toggled_remove = toggle_template_in_list(&toggled, "TREN_18");
        assert_eq!(toggled_remove, "MAU_GIAO");
        // Không cho phép bỏ hết tất cả mẫu
        assert_eq!(toggle_template_in_list("MAU_GIAO", "MAU_GIAO"), "MAU_GIAO");

        assert_eq!(namespaced_key("TREN_18", "CHIEU_CAO"), "CHIEU_CAO");
        assert_eq!(namespaced_key("LAI_XE", "CHIEU_CAO"), "LX_CHIEU_CAO");
        assert_eq!(namespaced_key("DUOI_6", "CHIEU_CAO"), "D6_CHIEU_CAO");
        assert_eq!(namespaced_key("TU_6_DUOI_18", "CHIEU_CAO"), "U18_CHIEU_CAO");
        assert_eq!(namespaced_key("MAU_GIAO", "CHIEU_CAO"), "MG_CHIEU_CAO");
        assert_eq!(namespaced_key("HS_PHO_THONG", "CHIEU_CAO"), "HS_CHIEU_CAO");

        for code in ["LAI_XE", "DUOI_6", "TU_6_DUOI_18", "MAU_GIAO", "HS_PHO_THONG"] {
            for tab in ["history", "physical", "clinical", "conclusion"] {
                let groups = get_template_groups(code, tab);
                assert!(!groups.is_empty(), "Missing groups for {} / {}", code, tab);
            }
        }

        // Kiểm tra tách biệt đúng trường giữa DUOI_6 vs MAU_GIAO
        let d6_phys: Vec<&str> = get_template_groups("DUOI_6", "physical")
            .iter()
            .flat_map(|g| g.fields.iter().map(|f| f.key))
            .collect();
        assert!(d6_phys.contains(&"DGDD_BINH_THUONG"));
        assert!(!d6_phys.contains(&"TINH_TRANG_DINH_DUONG"));

        let mg_phys: Vec<&str> = get_template_groups("MAU_GIAO", "physical")
            .iter()
            .flat_map(|g| g.fields.iter().map(|f| f.key))
            .collect();
        assert!(mg_phys.contains(&"TINH_TRANG_DINH_DUONG"));
        assert!(!mg_phys.contains(&"DGDD_BINH_THUONG"));

        // Kiểm tra tách biệt đúng trường giữa TU_6_DUOI_18 vs HS_PHO_THONG
        let u18_conc: Vec<&str> = get_template_groups("TU_6_DUOI_18", "conclusion")
            .iter()
            .flat_map(|g| g.fields.iter().map(|f| f.key))
            .collect();
        assert!(u18_conc.contains(&"PHAN_LOAI_SK"));
        assert!(!u18_conc.contains(&"CAN_TU_VAN"));

        let hs_conc: Vec<&str> = get_template_groups("HS_PHO_THONG", "conclusion")
            .iter()
            .flat_map(|g| g.fields.iter().map(|f| f.key))
            .collect();
        assert!(hs_conc.contains(&"CAN_TU_VAN"));
        assert!(!hs_conc.contains(&"PHAN_LOAI_SK"));
    }
}
