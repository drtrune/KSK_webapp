use printpdf::{PdfDocumentReference, PdfLayerReference};
use serde_json::{Map, Value};

use crate::config::AppConfig;

use super::super::cursor::PdfPageCursor;
use super::super::primitives::{
    draw_checkbox, draw_text, wrap_lines, CONTENT_W, FONT_BODY, LINE_H, MARGIN_L,
};
use super::super::values::FormValues;
use super::super::widgets::{
    draw_exam_custom_row, draw_exam_group_banner, draw_exam_spec_row, draw_exam_table_header,
    draw_inline_slot, draw_inline_two, draw_ts_table_double, max_content_x, write_lines,
};

/// Vẽ toàn bộ phiếu Giấy khám sức khỏe dùng cho người từ đủ 18 tuổi trở lên.
pub fn render(
    doc: &PdfDocumentReference,
    fonts: &super::super::primitives::PdfFonts,
    form: &Map<String, Value>,
    cat_map: &Map<String, Value>,
    cfg: &AppConfig,
    is_reception_print: bool,
    is_first_page_of_doc: bool,
    first_layer: Option<PdfLayerReference>,
) {
    let fv = FormValues::new(form, cat_map, is_reception_print, "TREN_18");
    let ma_tn = fv.admin("MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);

    let so_phieu = fv.admin("SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSK-{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        cfg.ma_cskcb,
        fv.exam_year_2d()
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG CHO",
        "NGƯỜI TỪ ĐỦ 18 TUỔI TRỞ LÊN",
    );

    render_admin_section(&fv, &mut cur);
    render_history_section(&fv, &mut cur, cfg);

    // Trang mới cho Khám thể lực, Lâm sàng, Cận lâm sàng, Kết luận
    cur.next_page();
    render_physical_section(&fv, &mut cur);
    render_clinical_section(&fv, &mut cur);
    render_paraclinical_section(&fv, &mut cur);
    render_conclusion_section(&fv, &mut cur, cfg);
}

/// Vẽ mục Thông tin hành chính 1..13 của mẫu Trên 18 tuổi.
fn render_admin_section(fv: &FormValues, cur: &mut PdfPageCursor) {
    let ho_ten = fv.admin("HO_TEN").to_uppercase();
    cur.draw_field("1. Họ và tên (viết chữ in hoa):", &ho_ten, true);
    cur.draw_gender_dob_age(fv.admin("GIOI_TINH"), fv.admin("NGAY_SINH"), &fv.age());
    cur.draw_field("4. Số CCCD/Hộ chiếu/Mã định danh:", fv.admin("SO_CCCD"), false);
    cur.draw_two_fields(
        "5. Cấp ngày:",
        fv.admin("NGAYCAP_CCCD"),
        "Nơi cấp:",
        fv.admin("NOICAP_CCCD"),
    );

    let dantoc = fv.cat("DAN_TOC", fv.admin("MA_DAN_TOC"));
    let doituong = fv.cat("DOI_TUONG", fv.admin("DOI_TUONG"));
    cur.draw_two_fields("6. Dân tộc:", &dantoc, "7. Đối tượng:", &doituong);

    let nguon = fv.cat("NGUON_CHI_TRA", fv.admin("NGUON_CHI_TRA"));
    cur.draw_two_fields("8. Nguồn chi trả:", &nguon, "9. Nhóm máu:", fv.admin("NHOM_MAU"));

    let tinh = fv.cat("TINH_TP", fv.admin("MATINH_CU_TRU"));
    let xa = fv.cat("XA_PHUONG", fv.admin("MAXA_CU_TRU"));
    cur.draw_two_fields("10. Nơi ở hiện tại - Tỉnh/thành:", &tinh, "Phường/xã:", &xa);
    cur.draw_field("    Số nhà/thôn/xóm:", fv.admin("DIA_CHI"), false);

    let nghe = fv.cat("NGHE_NGHIEP", fv.admin("MA_NGHE_NGHIEP"));
    cur.draw_field("11. Nghề nghiệp:", &nghe, false);
    cur.draw_field("12. Nơi làm việc, học tập:", fv.admin("NOI_LAM_VIEC_HOC_TAP"), false);
    let ly_do = if fv.admin("LY_DO_VV").is_empty() {
        "Khám sức khỏe"
    } else {
        fv.admin("LY_DO_VV")
    };
    cur.draw_field("13. Lý do khám sức khỏe:", ly_do, false);
}

/// Vẽ mục Tiền sử bệnh (gia đình, bảng 8 cột bản thân, câu hỏi khác, cam đoan & chữ ký).
fn render_history_section(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let max_x = max_content_x();
    cur.draw_centered_heading("TIỀN SỬ BỆNH CỦA ĐỐI TƯỢNG KHÁM SỨC KHỎE");

    cur.ensure_space(22.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "1. Tiền sử gia đình:");
    cur.y -= LINE_H;
    let lead = "Có ai trong gia đình ông (bà) mắc một trong các bệnh: truyền nhiễm, tim mạch, đái tháo đường, lao, hen phế quản, ung thư, động kinh, rối loạn tâm thần, bệnh khác:";
    for l in wrap_lines(lead, CONTENT_W, FONT_BODY) {
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, &l);
        cur.y -= 4.7;
    }

    let ts_gd_detail = fv.text(&["TSGD_MA_BENH", "TS_GIA_DINH_CHI_TIET"]);
    let ts_gd_yn = if fv.blank {
        None
    } else if fv.yes_no("TSGD_MAC_BENH") == Some(true)
        || fv.yes_no("TS_GIA_DINH") == Some(true)
        || !ts_gd_detail.is_empty()
    {
        Some(true)
    } else {
        fv.yes_no("TSGD_MAC_BENH").or_else(|| fv.yes_no("TS_GIA_DINH"))
    };

    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, "a)");
    draw_checkbox(&cur.layer, MARGIN_L + 5.5, cur.y, ts_gd_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 10.0, cur.y, "Không");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 26.0, cur.y, "b)");
    draw_checkbox(&cur.layer, MARGIN_L + 31.5, cur.y, ts_gd_yn == Some(true));
    draw_inline_slot(
        cur,
        MARGIN_L + 36.0,
        max_x,
        cur.y,
        "Có; Nếu “có”, đề nghị ghi cụ thể tên bệnh:",
        ts_gd_detail,
        false,
    );
    cur.y -= LINE_H + 1.0;

    cur.ensure_space(16.0);
    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "2. Tiền sử bản thân: Ông (bà) đã/đang mắc bệnh, tình trạng bệnh nào sau đây không:",
    );
    cur.y -= LINE_H;

    let left_defs: &[(&str, &str, &str)] = &[
        ("1", "Có bệnh hay bị thương trong 5 năm qua", "TSBT_BENH_TRONG_5_NAM_QUA"),
        ("2", "Có bệnh thần kinh hay bị thương ở đầu", "TSBT_BENH_THAN_KINH"),
        ("3", "Bệnh mắt hoặc giảm thị lực (trừ đeo kính thuốc)", "TSBT_BENH_MAT"),
        ("4", "Bệnh ở tai, giảm sức nghe hoặc thăng bằng", "TSBT_BENH_TAI"),
        ("5", "Bệnh ở tim, hoặc nhồi máu cơ tim, bệnh tim mạch khác", "TSBT_BENH_TIM"),
        ("6", "Phẫu thuật can thiệp tim - mạch (van, bắc cầu, stent...)", "TSBT_PHAU_THUAT_TIM"),
        ("7", "Tăng huyết áp", "TSBT_TANG_HUYET_AP"),
        ("8", "Khó thở", "TSBT_KHO_THO"),
        ("9", "Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn tính", "TSBT_BENH_PHOI"),
        ("10", "Bệnh thận, lọc máu", "TSBT_BENH_THAN"),
        ("11", "Nghiện rượu, bia", "TSBT_NGHIEN_RUOU"),
    ];
    let right_defs: &[(&str, &str, &str)] = &[
        ("12", "Đái tháo đường hoặc kiểm soát tăng đường huyết", "TSBT_DAI_THAO_DUONG"),
        ("13", "Bệnh tâm thần", "TSBT_BENH_TAM_THAN"),
        ("14", "Mất ý thức, rối loạn ý thức", "TSBT_MAT_Y_THUC"),
        ("15", "Ngất, chóng mặt", "TSBT_NGAT"),
        ("16", "Bệnh tiêu hóa", "TSBT_BENH_TIEU_HOA"),
        ("17", "Rối loạn giấc ngủ, ngừng thở khi ngủ, ngáy to", "TSBT_ROI_LOAN_GIAC_NGU"),
        ("18", "Tai biến mạch máu não hoặc liệt", "TSBT_TAI_BIEN"),
        ("19", "Bệnh hoặc tổn thương cột sống", "TSBT_BENH_COT_SONG"),
        ("20", "Sử dụng rượu thường xuyên, liên tục", "TSBT_RUOU_THUONG_XUYEN"),
        ("21", "Sử dụng ma túy và chất gây nghiện", "TSBT_MA_TUY"),
        ("22", "Bệnh khác (ghi rõ):", "TSBT_BENH_KHAC"),
    ];
    let map_side = |defs: &[(&'static str, &'static str, &str)]| -> Vec<(&'static str, &'static str, Option<bool>)> {
        defs.iter()
            .map(|&(tt, lbl, k)| (tt, lbl, fv.yes_no(k)))
            .collect()
    };
    let left_rows = map_side(left_defs);
    let right_rows = map_side(right_defs);
    draw_ts_table_double(cur, true, &left_rows, &right_rows);
    cur.y -= 4.0;

    // 3. Câu hỏi khác
    cur.ensure_space(28.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "3. Câu hỏi khác (nếu có):");
    cur.y -= LINE_H;
    let q3a = fv.text(&["TSBT_TEN_THUOC_LIEU_LUONG", "TS_DANG_DIEU_TRI"]);
    let q3a_yn = if fv.blank {
        None
    } else if fv.yes_no("TSBT_DANG_DIEU_TRI_BENH") == Some(true) || !q3a.is_empty() {
        Some(true)
    } else {
        fv.yes_no("TSBT_DANG_DIEU_TRI_BENH")
    };
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, "a) Ông (bà) có đang điều trị bệnh gì không?");
    cur.y -= LINE_H;
    draw_checkbox(&cur.layer, MARGIN_L + 2.0, cur.y, q3a_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 7.0, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 24.0, cur.y, q3a_yn == Some(true));
    draw_text(
        &cur.layer,
        &cur.fonts.italic,
        FONT_BODY,
        MARGIN_L + 29.0,
        cur.y,
        "Có (Nếu có, xin hãy liệt kê tên bệnh, các thuốc đang dùng và liều lượng):",
    );
    cur.y -= LINE_H;
    write_lines(cur, MARGIN_L, max_x, "", q3a, 2);

    let q3b = fv.text(&["TSBT_TEN_THUOC_THAI_SAN", "TS_THAI_SAN"]);
    let gt_nu = fv.admin("GIOI_TINH") == "2";
    let q3b_yn = if fv.blank || !gt_nu {
        None
    } else if fv.yes_no("TSBT_THAI_SAN") == Some(true) || !q3b.is_empty() {
        Some(true)
    } else {
        fv.yes_no("TSBT_THAI_SAN")
    };
    cur.ensure_space(20.0);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, "b) Tiền sử thai sản (Đối với phụ nữ):");
    cur.y -= LINE_H;
    draw_checkbox(&cur.layer, MARGIN_L + 2.0, cur.y, q3b_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 7.0, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 24.0, cur.y, q3b_yn == Some(true));
    draw_text(
        &cur.layer,
        &cur.fonts.italic,
        FONT_BODY,
        MARGIN_L + 29.0,
        cur.y,
        "Có (Nếu có, xin hãy liệt kê tên bệnh, các thuốc đang dùng và liều lượng):",
    );
    cur.y -= LINE_H;
    write_lines(cur, MARGIN_L, max_x, "", q3b, 2);

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        Some("Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật theo sự hiểu biết của tôi."),
        None,
        "",
        "NGƯỜI ĐỀ NGHỊ KHÁM SỨC KHỎE",
        "(Ký và ghi rõ họ, tên)",
        "",
    );
}

/// Vẽ phần I. Khám thể lực.
fn render_physical_section(fv: &FormValues, cur: &mut PdfPageCursor) {
    cur.draw_section_heading("I. KHÁM THỂ LỰC");
    cur.ensure_space(18.0);
    let max_x = max_content_x();
    let w3 = CONTENT_W / 3.0;
    draw_inline_slot(cur, MARGIN_L, MARGIN_L + w3 - 2.0, cur.y, "- Chiều cao (cm):", fv.text(&["CHIEU_CAO"]), false);
    draw_inline_slot(cur, MARGIN_L + w3, MARGIN_L + w3 * 2.0 - 2.0, cur.y, "Cân nặng (kg):", fv.text(&["CAN_NANG"]), false);
    draw_inline_slot(cur, MARGIN_L + w3 * 2.0, max_x, cur.y, "Chỉ số BMI:", fv.text(&["CHI_SO_BMI", "BMI"]), false);
    cur.y -= LINE_H;

    let ha = if !fv.text(&["HUYET_AP"]).is_empty() {
        fv.text(&["HUYET_AP"]).to_string()
    } else if !fv.text(&["HUYET_AP_TAM_THU"]).is_empty() {
        format!("{}/{}", fv.text(&["HUYET_AP_TAM_THU"]), fv.text(&["HUYET_AP_TAM_TRUONG"]))
    } else {
        String::new()
    };
    draw_inline_two(
        cur,
        MARGIN_L,
        max_x,
        "- Mạch (lần/phút):",
        fv.text(&["MACH"]),
        "- Huyết áp (mmHg):",
        &ha,
    );
    cur.draw_field(
        "- Phân loại thể lực:",
        fv.text(&["KHAM_THE_LUC_PL", "PHAN_LOAI_THE_LUC"]),
        false,
    );
}

/// Vẽ phần II. Khám lâm sàng dạng bảng chuẩn có ô Kết quả, Phân loại và cột chữ ký BS.
fn render_clinical_section(fv: &FormValues, cur: &mut PdfPageCursor) {
    cur.draw_section_heading("II. KHÁM LÂM SÀNG");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên và chữ ký của Bác sỹ");
    draw_exam_group_banner(cur, col1_w, col2_w, "1. Nội khoa");

    let noi_items: &[(&str, &[&str], &[&str])] = &[
        ("a) Tuần hoàn", &["NOI_KHOA_TUAN_HOAN", "KHAM_NOI_TUAN_HOAN"], &["NOI_KHOA_TUAN_HOAN_PL", "PL_NOI_TUAN_HOAN"]),
        ("b) Hô hấp", &["NOI_KHOA_HO_HAP", "KHAM_NOI_HO_HAP"], &["NOI_KHOA_HO_HAP_PL", "PL_NOI_HO_HAP"]),
        ("c) Tiêu hóa", &["NOI_KHOA_TIEU_HOA", "KHAM_NOI_TIEU_HOA"], &["NOI_KHOA_TIEU_HOA_PL", "PL_NOI_TIEU_HOA"]),
        ("d) Thận - Tiết niệu - Sinh dục", &["NOI_KHOA_THAN_TN_SD", "KHAM_NOI_THAN_TN"], &["NOI_KHOA_THAN_TIETNIEU_PL", "PL_NOI_THAN_TN"]),
        ("đ) Nội tiết", &["NOI_KHOA_NOI_TIET", "KHAM_NOI_TIET"], &["NOI_KHOA_NOI_TIET_PL", "PL_NOI_TIET"]),
        ("e) Cơ - xương - khớp", &["NOI_KHOA_CO_XUONG_KHOP", "KHAM_NOI_CO_XUONG_KHOP"], &["NOI_KHOA_CO_XUONG_KHOP_PL", "PL_NOI_CO_XUONG_KHOP"]),
        ("g) Thần kinh", &["NOI_KHOA_THAN_KINH", "KHAM_NOI_THAN_KINH"], &["NOI_KHOA_THAN_KINH_PL", "PL_NOI_THAN_KINH"]),
        ("h) Tâm thần", &["NOI_KHOA_TAM_THAN", "KHAM_NOI_TAM_THAN"], &["NOI_KHOA_TAM_THAN_PL", "PL_NOI_TAM_THAN"]),
    ];
    let bs_noi = fv.text(&["BS_NOI", "BS_NOI_KHOA"]);
    for (lbl, vk, pk) in noi_items {
        draw_exam_spec_row(cur, col1_w, col2_w, lbl, fv.text(vk), "Phân loại:", fv.text(pk), bs_noi, 1);
    }

    let other_specs: &[(&str, &[&str], &[&str], &[&str])] = &[
        ("2. Ngoại khoa", &["KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA"], &["KHAM_NGOAI_KHOA_PL", "PL_NGOAI_KHOA"], &["BS_NGOAI", "BS_NGOAI_KHOA"]),
        ("3. Da liễu", &["KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU"], &["KHAM_DA_LIEU_PL", "PL_DA_LIEU"], &["BS_DA_LIEU"]),
        ("4. Sản phụ khoa (nữ)", &["KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA"], &["KHAM_SAN_PHU_KHOA_PL", "PL_SAN_PHU_KHOA"], &["BS_SAN", "BS_SAN_PHU_KHOA"]),
    ];
    for (lbl, vk, pk, bk) in other_specs {
        draw_exam_spec_row(cur, col1_w, col2_w, lbl, fv.text(vk), "Phân loại:", fv.text(pk), fv.text(bk), 1);
    }

    // 5. Mắt
    draw_exam_custom_row(cur, col1_w, col2_w, 30.0, fv.text(&["BS_MAT"]), |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "5. Mắt - Kết quả khám thị lực:");
        c.y -= LINE_H;
        draw_inline_two(
            c,
            lx,
            rx,
            "Không kính - Mắt phải:",
            fv.text(&["KHONG_KINH_MAT_PHAI", "MAT_KHONG_KINH_PHAI"]),
            "Mắt trái:",
            fv.text(&["KHONG_KINH_MAT_TRAI", "MAT_KHONG_KINH_TRAI"]),
        );
        draw_inline_two(
            c,
            lx,
            rx,
            "Có kính - Mắt phải:",
            fv.text(&["CO_KINH_MAT_PHAI", "MAT_CO_KINH_PHAI"]),
            "Mắt trái:",
            fv.text(&["CO_KINH_MAT_TRAI", "MAT_CO_KINH_TRAI"]),
        );
        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về mắt (nếu có):", fv.text(&["BENH_KHAC_MAT", "KHAM_MAT"]), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Phân loại:", fv.text(&["KHAM_MAT_PL", "PL_MAT"]), true);
    });

    // 6. Tai - Mũi - Họng
    draw_exam_custom_row(cur, col1_w, col2_w, 30.0, fv.text(&["BS_TMH"]), |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "6. Tai - Mũi - Họng - Kết quả khám thính lực:");
        c.y -= LINE_H;
        draw_inline_two(
            c,
            lx,
            rx,
            "Tai trái - Nói thường (m):",
            fv.text(&["TAI_TRAI_NOI_THUONG", "TMH_TAI_TRAI_THUONG"]),
            "Nói thầm (m):",
            fv.text(&["TAI_TRAI_NOI_THAM", "TMH_TAI_TRAI_THAM"]),
        );
        draw_inline_two(
            c,
            lx,
            rx,
            "Tai phải - Nói thường (m):",
            fv.text(&["TAI_PHAI_NOI_THUONG", "TMH_TAI_PHAI_THUONG"]),
            "Nói thầm (m):",
            fv.text(&["TAI_PHAI_NOI_THAM", "TMH_TAI_PHAI_THAM"]),
        );
        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về tai mũi họng (nếu có):", fv.text(&["BENH_KHAC_TAI_MUI_HONG", "KHAM_TMH"]), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Phân loại:", fv.text(&["KHAM_TAI_MUI_HONG_PL", "PL_TMH"]), true);
    });

    // 7. Răng - Hàm - Mặt
    draw_exam_custom_row(cur, col1_w, col2_w, 28.0, fv.text(&["BS_RHM"]), |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "7. Răng - Hàm - Mặt - Kết quả khám:");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Hàm trên:", fv.text(&["HAM_TREN", "RHM_HAM_TREN"]), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Hàm dưới:", fv.text(&["HAM_DUOI", "RHM_HAM_DUOI"]), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về răng hàm mặt (nếu có):", fv.text(&["BENH_KHAC_RANG_HAM_MAT", "KHAM_RHM"]), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Phân loại:", fv.text(&["KHAM_RANG_HAM_MAT_PL", "PL_RHM"]), true);
    });
}

/// Vẽ phần III. Khám cận lâm sàng (mục 1 đi học/đi làm và mục 2 khám định kỳ).
fn render_paraclinical_section(fv: &FormValues, cur: &mut PdfPageCursor) {
    cur.draw_section_heading("III. KHÁM CẬN LÂM SÀNG");
    cur.ensure_space(12.0);
    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "1. Khám phân loại sức khỏe để đi học, đi làm thực hiện khám cận lâm sàng sau:",
    );
    cur.y -= 5.2;

    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký của Bác sỹ");
    let bs_cls = fv.text(&["BS_CLS"]);

    draw_exam_custom_row(cur, col1_w, col2_w, 26.0, bs_cls, |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "a) Xét nghiệm máu:");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "- Huyết học (tổng phân tích tế bào máu):", fv.text(&["CLS_HUYET_HOC", "XN_HUYET_HOC"]), false);
        c.y -= LINE_H;
        draw_inline_two(c, lx, rx, "- Đường máu:", fv.text(&["CLS_DUONG_MAU", "XN_DUONG_MAU"]), "Urê:", fv.text(&["CLS_URE", "XN_URE"]));
        let got_gpt = format!(
            "{} {}",
            fv.text(&["CLS_ASAT_GOT", "XN_GOT"]),
            fv.text(&["CLS_ALAT_GPT", "XN_GPT"])
        );
        draw_inline_two(
            c,
            lx,
            rx,
            "  Creatinin:",
            fv.text(&["CLS_CREATININ", "XN_CREATININ"]),
            "ASAT(GOT)/ALAT(GPT):",
            got_gpt.trim(),
        );
    });

    draw_exam_custom_row(cur, col1_w, col2_w, 16.0, bs_cls, |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "b) Xét nghiệm nước tiểu:");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "- Tổng phân tích nước tiểu / Khác:", fv.text(&["CLS_NUOC_TIEU", "XN_NUOC_TIEU"]), false);
    });

    draw_exam_custom_row(cur, col1_w, col2_w, 20.0, bs_cls, |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "c) Chẩn đoán hình ảnh (XQ tim phổi thẳng):");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "", fv.text(&["CLS_XQ_TIM_PHOI", "CDHA_XQ"]), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "", "", false);
    });

    let cls_khac = fv.text(&["CLS_KHAC_MO_TA", "CLS_KHAC"]);
    let cls_khac_yn = if fv.blank {
        None
    } else if !cls_khac.is_empty() {
        Some(true)
    } else {
        None
    };
    draw_exam_custom_row(cur, col1_w, col2_w, 20.0, bs_cls, |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "d) Kết quả khám Cận lâm sàng khác:");
        c.y -= LINE_H;
        draw_checkbox(&c.layer, lx + 1.0, c.y, cls_khac_yn == Some(false));
        draw_text(&c.layer, &c.fonts.regular, FONT_BODY, lx + 5.5, c.y, "Không");
        draw_checkbox(&c.layer, lx + 22.0, c.y, cls_khac_yn == Some(true));
        draw_inline_slot(c, lx + 26.5, rx, c.y, "Có (liệt kê chi tiết):", cls_khac, false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "", "", false);
    });

    // Mục 2: Khám sức khỏe định kỳ
    cur.y -= 3.0;
    cur.ensure_space(42.0);
    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "2. Khám sức khỏe định kỳ (cận lâm sàng thực hiện theo chỉ định của bác sỹ):",
    );
    cur.y -= 5.2;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký của Bác sỹ");
    let kq_dk = fv.text(&["CLS_DINH_KY_KET_QUA", "CLS_THEO_CHI_DINH"]);
    draw_exam_custom_row(cur, col1_w, col2_w, 28.0, bs_cls, |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.regular, FONT_BODY, lx, c.y, "Cận lâm sàng thực hiện theo chỉ định của bác sỹ:");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Kết quả:", kq_dk, false);
        for _ in 0..3 {
            c.y -= LINE_H;
            draw_inline_slot(c, lx, rx, c.y, "", "", false);
        }
    });
}

/// Vẽ phần IV. Kết luận sức khỏe và chữ ký Người kết luận.
fn render_conclusion_section(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let max_x = max_content_x();
    cur.ensure_space(68.0);
    cur.draw_section_heading("IV. KẾT LUẬN");
    let kl_benh = format!(
        "{} {}",
        fv.text(&["KET_LUAN_BENH_TAT", "KET_LUAN_BENH"]),
        fv.text(&["CAC_BENH_TAT_NEU_CO"])
    )
    .trim()
    .to_string();
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có):",
        &kl_benh,
        5,
    );

    let pl_sk = fv.text(&["PHAN_LOAI_SK", "PHAN_LOAI_SUC_KHOE"]);
    let pl_disp = if pl_sk.is_empty() {
        String::new()
    } else if pl_sk.to_ascii_lowercase().contains("loại") {
        pl_sk.to_string()
    } else {
        format!("Loại {}", pl_sk)
    };
    cur.draw_field("2. Phân loại sức khỏe (theo quy định của Bộ Y tế):", &pl_disp, true);

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        None,
        None,
        "",
        "NGƯỜI KẾT LUẬN",
        "(Ký, ghi rõ họ tên và đóng dấu)",
        fv.text(&["BS_KET_LUAN", "NGUOI_KET_LUAN"]),
    );
}
