use printpdf::{PdfDocumentReference, PdfLayerReference};
use serde_json::{Map, Value};

use crate::config::AppConfig;

use super::super::cursor::PdfPageCursor;
use super::super::primitives::{draw_checkbox, draw_text, CONTENT_W, FONT_BODY, LINE_H, MARGIN_L};
use super::super::values::FormValues;
use super::super::widgets::{
    draw_checklist_table, draw_exam_custom_row, draw_exam_group_banner, draw_exam_table_header,
    draw_inline_slot, draw_inline_two, max_content_x, option_row, write_lines,
};

/// Vẽ toàn bộ phiếu Giấy khám sức khỏe dùng cho người từ đủ 06 tuổi đến dưới 18 tuổi.
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
    let fv = FormValues::new(form, cat_map, is_reception_print, "TU_6_DUOI_18");
    let ma_tn = fv.admin("MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);

    let so_phieu = fv.admin("SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSK-TE18-{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        cfg.ma_cskcb,
        fv.exam_year_2d()
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG CHO",
        "NGƯỜI TỪ ĐỦ 06 TUỔI ĐẾN DƯỚI 18 TUỔI",
    );

    render_admin(&fv, &mut cur);
    render_history(&fv, &mut cur, cfg);
    render_physical_and_clinical(&fv, &mut cur);
    render_paraclinical_and_conclusion(&fv, &mut cur, cfg);
}

/// Vẽ thông tin hành chính 1..13 của mẫu 6 đến dưới 18 tuổi.
fn render_admin(fv: &FormValues, cur: &mut PdfPageCursor) {
    let max_x = max_content_x();
    cur.draw_field(
        "1. Họ và tên (viết chữ in hoa):",
        &fv.admin("HO_TEN").to_uppercase(),
        true,
    );
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

    cur.draw_field("10. Nơi ở hiện tại:", &fv.full_address(), false);
    cur.draw_field(
        "11. Họ tên người đi cùng trẻ / Người giám hộ (trẻ ≤16 tuổi):",
        fv.admin("HO_TEN_NGUOI_GIAM_HO"),
        false,
    );
    option_row(
        cur,
        MARGIN_L,
        max_x,
        "12. Mối quan hệ với trẻ:",
        &["Cha", "Mẹ", "Ông/bà", "Anh/chị", "Họ hàng", "Khác"],
        fv.admin("MOI_QUAN_HE_VOI_TRE"),
        6,
    );
    let ly_do = if fv.admin("LY_DO_VV").is_empty() {
        "Khám sức khỏe"
    } else {
        fv.admin("LY_DO_VV")
    };
    cur.draw_field("13. Lý do khám sức khỏe:", ly_do, false);
}

/// Vẽ Tiền sử bệnh tật (gia đình, sản khoa, bảng tiêm chủng 5 cột, mãn tính, đang điều trị).
fn render_history(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let max_x = max_content_x();
    cur.draw_centered_heading("TIỀN SỬ BỆNH TẬT");

    cur.ensure_space(18.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "1. Tiền sử gia đình:");
    cur.y -= LINE_H;
    draw_text(
        &cur.layer,
        &cur.fonts.regular,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "Có ai trong gia đình mắc các bệnh bẩm sinh hoặc bệnh truyền nhiễm không:",
    );
    cur.y -= LINE_H;
    let tsgd_yn = fv.ns_yes_no("TSGD_MAC_BENH");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, "a)");
    draw_checkbox(&cur.layer, MARGIN_L + 5.5, cur.y, tsgd_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 10.0, cur.y, "Không");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 26.0, cur.y, "b)");
    draw_checkbox(&cur.layer, MARGIN_L + 31.5, cur.y, tsgd_yn == Some(true));
    draw_inline_slot(
        cur,
        MARGIN_L + 36.0,
        max_x,
        cur.y,
        "Có; Nếu “có”, ghi cụ thể tên bệnh:",
        fv.ns("TSGD_MA_BENH"),
        false,
    );
    cur.y -= LINE_H;

    // 2. Tiền sử bản thân
    cur.ensure_space(20.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "2. Tiền sử bản thân:");
    cur.y -= 4.5;
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "a) Sản khoa:");
    draw_checkbox(&cur.layer, MARGIN_L + 28.0, cur.y, fv.ns_matches("SAN_KHOA", "Bình thường"));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 32.5, cur.y, "Bình thường.");
    let sk_kbt = fv.ns_matches("SAN_KHOA", "Không bình thường")
        || fv.ns_yes_no("SAN_KHOA_KHONG_BT") == Some(true);
    draw_checkbox(&cur.layer, MARGIN_L + 60.0, cur.y, sk_kbt);
    draw_inline_slot(
        cur,
        MARGIN_L + 64.5,
        max_x,
        cur.y,
        "Không BT (thiếu tháng/can thiệp/ngạt/mẹ bệnh):",
        fv.ns("MA_BENH_SAN_KHOA_KHONG_BT"),
        false,
    );
    cur.y -= LINE_H;

    // b) Tiêm chủng (bảng 5 cột: STT | Loại vắc xin | Có | Không | Không rõ)
    cur.ensure_space(16.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "b) Tiêm chủng:");
    cur.y -= LINE_H;
    let vac_state = |k: &str| -> Option<u8> {
        if fv.blank {
            return None;
        }
        let s = fv.ns(k);
        if s.eq_ignore_ascii_case("Có") {
            Some(0)
        } else if s.eq_ignore_ascii_case("Không") {
            Some(1)
        } else if s.eq_ignore_ascii_case("Không rõ") || s.eq_ignore_ascii_case("Không nhớ rõ") {
            Some(2)
        } else {
            None
        }
    };
    let khac_label = if fv.ns("TC_KHAC_TEN").is_empty() {
        "Các loại khác".to_string()
    } else {
        format!("Các loại khác: {}", fv.ns("TC_KHAC_TEN"))
    };
    draw_checklist_table(
        cur,
        "Loại vắc xin",
        true,
        true,
        &[
            ("1", "BCG", vac_state("TC_BCG")),
            ("2", "Bạch hầu, ho gà, uốn ván", vac_state("TC_BH_HG_UV")),
            ("3", "Sởi", vac_state("TC_SOI")),
            ("4", "Bại liệt", vac_state("TC_BAI_LIET")),
            ("5", "Viêm não Nhật Bản B", vac_state("TC_VNNB_B")),
            ("6", "Viêm gan B", vac_state("TC_VGB")),
            ("7", &khac_label, vac_state("TC_KHAC_TINH_TRANG")),
        ],
    );
    // c) & d)
    cur.ensure_space(32.0);
    if cur.y < super::super::primitives::TOP_Y - 1.0 {
        cur.y -= 6.5;
    }
    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "c) Tiền sử bệnh/tật (Các bệnh bẩm sinh và mãn tính):",
    );
    cur.y -= LINE_H;
    let tsbt_yn = fv.ns_yes_no("TSBT_MAC_BENH");
    draw_checkbox(&cur.layer, MARGIN_L, cur.y, tsbt_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 4.5, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 22.0, cur.y, tsbt_yn == Some(true));
    draw_inline_slot(cur, MARGIN_L + 26.5, max_x, cur.y, "Có (ghi cụ thể tên bệnh):", fv.ns("TSBT_MA_BENH"), false);
    cur.y -= LINE_H;

    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "d) Hiện tại có đang điều trị bệnh gì không?",
    );
    let dt_yn = fv.ns_yes_no("DANG_DIEU_TRI_BENH");
    draw_checkbox(&cur.layer, MARGIN_L + 82.0, cur.y, dt_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 86.5, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 104.0, cur.y, dt_yn == Some(true));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 108.5, cur.y, "Có");
    cur.y -= LINE_H;
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "Nếu có, ghi rõ tên bệnh và liệt kê các thuốc đang dùng:",
        fv.ns("GHI_RO_BENH_DANG_DT"),
        1,
    );

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        Some("Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật theo sự hiểu biết của tôi."),
        None,
        "",
        "Người đề nghị khám sức khỏe",
        "(Hoặc cha/mẹ hoặc thân nhân - Ký, ghi rõ họ tên)",
        "",
    );
}

/// Vẽ I. Khám thể lực & II. Khám lâm sàng (Nhi khoa, Mắt, TMH, RHM).
fn render_physical_and_clinical(fv: &FormValues, cur: &mut PdfPageCursor) {
    let max_x = max_content_x();
    cur.draw_section_heading("I. KHÁM THỂ LỰC");
    cur.ensure_space(18.0);
    let w3 = CONTENT_W / 3.0;
    draw_inline_slot(cur, MARGIN_L, MARGIN_L + w3 - 2.0, cur.y, "- Chiều cao (cm):", fv.ns("CHIEU_CAO"), false);
    draw_inline_slot(cur, MARGIN_L + w3, MARGIN_L + w3 * 2.0 - 2.0, cur.y, "Cân nặng (kg):", fv.ns("CAN_NANG"), false);
    draw_inline_slot(cur, MARGIN_L + w3 * 2.0, max_x, cur.y, "Chỉ số BMI:", fv.ns("BMI"), false);
    cur.y -= LINE_H;
    draw_inline_two(cur, MARGIN_L, max_x, "- Mạch (lần/phút):", fv.ns("MACH"), "- Huyết áp (mmHg):", fv.ns("HUYET_AP"));
    cur.draw_field("- Phân loại thể lực:", fv.ns("KHAM_THE_LUC_PL"), false);

    cur.draw_section_heading("II. KHÁM LÂM SÀNG");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên và chữ ký của Bác sỹ");
    draw_exam_group_banner(cur, col1_w, col2_w, "1. Nhi khoa");

    let nhi_items: &[(&str, &str)] = &[
        ("a) Tuần hoàn", "NHI_KHOA_TUAN_HOAN"),
        ("b) Hô hấp", "NHI_KHOA_HO_HAP"),
        ("c) Tiêu hóa", "NHI_KHOA_TIEU_HOA"),
        ("d) Thận - Tiết niệu, Sinh dục", "NHI_KHOA_THAN_TN_SD"),
        ("đ) Thần kinh", "NHI_KHOA_THAN_KINH"),
        ("e) Tâm thần", "NHI_KHOA_TAM_THAN"),
        ("g) Khám lâm sàng khác", "NHI_KHOA_LAM_SANG_KHAC"),
    ];
    for (lbl, key) in nhi_items {
        draw_exam_custom_row(cur, col1_w, col2_w, 12.0, "", |c, lx, rx| {
            draw_inline_slot(c, lx, rx, c.y, &format!("{}:", lbl), fv.ns(key), false);
        });
    }

    // 2. Mắt
    draw_exam_custom_row(cur, col1_w, col2_w, 24.0, "", |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "2. Mắt - Kết quả khám thị lực:");
        c.y -= LINE_H;
        draw_inline_two(c, lx, rx, "Không kính - Mắt phải:", fv.ns("TL_KHONG_KINH_PHAI"), "Mắt trái:", fv.ns("TL_KHONG_KINH_TRAI"));
        draw_inline_two(c, lx, rx, "Có kính - Mắt phải:", fv.ns("TL_CO_KINH_PHAI"), "Mắt trái:", fv.ns("TL_CO_KINH_TRAI"));
        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về mắt (nếu có):", fv.ns("BENH_VE_MAT"), false);
    });

    // 3. Tai - Mũi - Họng
    draw_exam_custom_row(cur, col1_w, col2_w, 24.0, "", |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "3. Tai - Mũi - Họng - Kết quả khám thính lực:");
        c.y -= LINE_H;
        draw_inline_two(c, lx, rx, "Tai trái - Nói thường (m):", fv.ns("THINH_LUC_THUONG_TRAI"), "Nói thầm (m):", fv.ns("THINH_LUC_THAM_TRAI"));
        draw_inline_two(c, lx, rx, "Tai phải - Nói thường (m):", fv.ns("THINH_LUC_THUONG_PHAI"), "Nói thầm (m):", fv.ns("THINH_LUC_THAM_PHAI"));
        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về tai mũi họng (nếu có):", fv.ns("BENH_TAI_MUI_HONG"), false);
    });

    // 4. Răng - Hàm - Mặt
    draw_exam_custom_row(cur, col1_w, col2_w, 22.0, "", |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "4. Răng - Hàm - Mặt - Kết quả khám:");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Hàm trên:", fv.ns("RHM_HAM_TREN"), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Hàm dưới:", fv.ns("RHM_HAM_DUOI"), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về răng hàm mặt (nếu có):", fv.ns("BENH_RANG_HAM_MAT"), false);
    });
}

/// Vẽ III. Khám cận lâm sàng & IV. Kết luận chung (6 đến dưới 18 tuổi).
fn render_paraclinical_and_conclusion(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let max_x = max_content_x();
    cur.draw_section_heading("III. KHÁM CẬN LÂM SÀNG");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký của Bác sỹ");
    draw_exam_custom_row(cur, col1_w, col2_w, 28.0, "", |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.regular, FONT_BODY, lx, c.y, "Cận lâm sàng thực hiện theo chỉ định của bác sỹ:");
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Kết quả:", fv.ns("DICH_VU_CLS"), false);
        for _ in 0..3 {
            c.y -= LINE_H;
            draw_inline_slot(c, lx, rx, c.y, "", "", false);
        }
    });

    cur.ensure_space(56.0);
    cur.draw_section_heading("IV. KẾT LUẬN CHUNG:");
    let kl_txt = [fv.ns("KET_LUAN_BENH"), fv.ns("CAC_VAN_DE_SUC_KHOE")]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" - ");
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có):",
        &kl_txt,
        4,
    );
    cur.draw_field(
        "2. Phân loại sức khỏe (theo quy định của Bộ Y tế):",
        fv.ns("PHAN_LOAI_SK"),
        true,
    );

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        None,
        None,
        "",
        "NGƯỜI KẾT LUẬN",
        "(Ký, ghi rõ họ tên và đóng dấu)",
        "",
    );
}

