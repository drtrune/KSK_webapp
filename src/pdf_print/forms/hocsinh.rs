use printpdf::{PdfDocumentReference, PdfLayerReference};
use serde_json::{Map, Value};

use crate::config::AppConfig;

use super::super::cursor::PdfPageCursor;
use super::super::primitives::{draw_checkbox, draw_text, CONTENT_W, FONT_BODY, LINE_H, MARGIN_L};
use super::super::values::FormValues;
use super::super::widgets::{
    draw_checklist_table, draw_inline_slot, draw_inline_two, max_content_x, option_row,
    write_lines, BoxedSection,
};

/// Vẽ toàn bộ phiếu Kiểm tra sức khỏe đầu năm học cho học sinh phổ thông.
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
    let fv = FormValues::new(form, cat_map, is_reception_print, "HS_PHO_THONG");
    let ma_tn = fv.admin("MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);

    let so_phieu = fv.admin("SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSK-HS-{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        cfg.ma_cskcb,
        fv.exam_year_2d()
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "KIỂM TRA SỨC KHỎE ĐẦU NĂM HỌC CHO HỌC SINH TRONG CÁC",
        "CƠ SỞ GIÁO DỤC PHỔ THÔNG",
    );

    render_admin(&fv, &mut cur);
    render_history(&fv, &mut cur);

    cur.next_page();
    render_physical_and_clinical_boxed(&fv, &mut cur);
    render_conclusion(&fv, &mut cur, cfg);
}

/// Vẽ Thông tin hành chính 1..11 của mẫu Học sinh phổ thông.
fn render_admin(fv: &FormValues, cur: &mut PdfPageCursor) {
    cur.draw_centered_heading("THÔNG TIN HÀNH CHÍNH");
    cur.draw_field(
        "1. Họ và tên (viết chữ in hoa):",
        &fv.admin("HO_TEN").to_uppercase(),
        true,
    );
    cur.draw_gender_dob_age(fv.admin("GIOI_TINH"), fv.admin("NGAY_SINH"), &fv.age());
    cur.draw_field("4. Số CC/CCCD/Hộ chiếu/Mã định danh:", fv.admin("SO_CCCD"), false);
    cur.draw_two_fields(
        "5. Cấp ngày:",
        fv.admin("NGAYCAP_CCCD"),
        "Nơi cấp:",
        fv.admin("NOICAP_CCCD"),
    );

    let dantoc = fv.cat("DAN_TOC", fv.admin("MA_DAN_TOC"));
    cur.draw_two_fields(
        "6. Dân tộc:",
        &dantoc,
        "7. Đối tượng:",
        "Học sinh trong các CSGD phổ thông",
    );

    let nguon = fv.cat("NGUON_CHI_TRA", fv.admin("NGUON_CHI_TRA"));
    cur.draw_two_fields("8. Nguồn chi trả:", &nguon, "9. Nhóm máu:", fv.admin("NHOM_MAU"));

    let tinh = fv.cat("TINH_TP", fv.admin("MATINH_CU_TRU"));
    let xa = fv.cat("XA_PHUONG", fv.admin("MAXA_CU_TRU"));
    cur.draw_two_fields("10. Nơi ở hiện tại - Tỉnh/thành:", &tinh, "Phường/xã:", &xa);
    cur.draw_field("    Số nhà/thôn/xóm:", fv.admin("DIA_CHI"), false);
    cur.draw_field("- Tên cơ sở giáo dục đang theo học:", fv.admin("TEN_CSGD"), false);
    let ly_do = if fv.admin("LY_DO_VV").is_empty() {
        "Kiểm tra sức khỏe đầu năm học"
    } else {
        fv.admin("LY_DO_VV")
    };
    cur.draw_field("11. Lý do khám sức khỏe:", ly_do, false);
}

/// Vẽ Tiền sử bệnh tật (gia đình, sản khoa, bảng tiêm chủng 5 cột, mãn tính, đang điều trị).
fn render_history(fv: &FormValues, cur: &mut PdfPageCursor) {
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
        "Có; Nếu “có”, đề nghị ghi cụ thể tên bệnh:",
        fv.ns("TSGD_MA_BENH"),
        false,
    );
    cur.y -= LINE_H;
    write_lines(cur, MARGIN_L, max_x, "", "", 1);

    cur.ensure_space(26.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "2. Tiền sử bản thân:");
    cur.y -= LINE_H;
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "a) Sản khoa:");
    cur.y -= LINE_H;
    let sk_bt = fv.ns_matches("TSBT_SAN_KHOA", "Bình thường");
    let sk_kbt = fv.ns_matches("TSBT_SAN_KHOA", "Không bình thường");
    draw_checkbox(&cur.layer, MARGIN_L, cur.y, sk_bt);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 4.5, cur.y, "Bình thường.");
    cur.y -= LINE_H;
    draw_checkbox(&cur.layer, MARGIN_L, cur.y, sk_kbt);
    draw_inline_slot(
        cur,
        MARGIN_L + 4.5,
        max_x,
        cur.y,
        "Không bình thường (Đẻ thiếu tháng/thừa cân/can thiệp/ngạt; Mẹ bệnh khi mang thai):",
        "",
        false,
    );
    cur.y -= LINE_H;
    write_lines(cur, MARGIN_L, max_x, "", "", 1);

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
    cur.y -= 3.0;

    cur.ensure_space(28.0);
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
    cur.y -= LINE_H;
    let dt_yn = fv.ns_yes_no("DANG_DIEU_TRI_BENH");
    draw_checkbox(&cur.layer, MARGIN_L, cur.y, dt_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 4.5, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 22.0, cur.y, dt_yn == Some(true));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 26.5, cur.y, "Có");
    cur.y -= LINE_H;
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "Nếu “có”, ghi rõ tên bệnh và liệt kê các thuốc đang dùng:",
        fv.ns("GHI_RO_BENH_DANG_DT"),
        2,
    );
}

/// Vẽ I. Khám thể lực & II. Khám lâm sàng trong khung viền (Học sinh phổ thông).
fn render_physical_and_clinical_boxed(fv: &FormValues, cur: &mut PdfPageCursor) {
    let max_x = max_content_x();
    cur.draw_section_heading("I. KHÁM THỂ LỰC");
    cur.ensure_space(16.0);
    let w3 = CONTENT_W / 3.0;
    draw_inline_slot(cur, MARGIN_L, MARGIN_L + w3 - 2.0, cur.y, "- Chiều cao (cm):", fv.ns("CHIEU_CAO"), false);
    draw_inline_slot(cur, MARGIN_L + w3, MARGIN_L + w3 * 2.0 - 2.0, cur.y, "Cân nặng (kg):", fv.ns("CAN_NANG"), false);
    draw_inline_slot(cur, MARGIN_L + w3 * 2.0, max_x, cur.y, "Chỉ số BMI:", fv.ns("BMI"), false);
    cur.y -= LINE_H;
    draw_inline_two(cur, MARGIN_L, max_x, "- Mạch (lần/phút):", fv.ns("MACH"), "- Huyết áp (mmHg):", fv.ns("HUYET_AP"));

    cur.draw_section_heading("II. KHÁM LÂM SÀNG:");
    cur.ensure_space(14.0);
    draw_text(
        &cur.layer,
        &cur.fonts.italic,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "Quan sát: Nét mặt/tư thế/tỷ lệ, sự đối xứng với các bộ phận cơ thể/sự chuyển động của trẻ. Tìm dấu hiệu bất thường.",
    );
    cur.y -= LINE_H + 1.0;

    let mut bx = BoxedSection::begin(cur, Some("1. Mắt:"));
    let lx = bx.x + 2.5;
    let rx = bx.x + bx.w - 2.5;
    draw_inline_two(c_ref(cur), lx, rx, "Không kính - Mắt phải:", fv.ns("TL_KHONG_KINH_PHAI"), "Mắt trái:", fv.ns("TL_KHONG_KINH_TRAI"));
    draw_inline_two(c_ref(cur), lx, rx, "Có kính - Mắt phải:", fv.ns("TL_CO_KINH_PHAI"), "Mắt trái:", fv.ns("TL_CO_KINH_TRAI"));
    write_lines(cur, lx, rx, "Các bệnh về mắt (nếu có):", fv.ns("BENH_VE_MAT"), 2);

    bx.divider(cur, Some("2. Tai - Mũi - Họng:"));
    draw_inline_two(cur, lx, rx, "Tai trái - Nói thường (m):", fv.ns("THINH_LUC_THUONG_TRAI"), "Nói thầm (m):", fv.ns("THINH_LUC_THAM_TRAI"));
    draw_inline_two(cur, lx, rx, "Tai phải - Nói thường (m):", fv.ns("THINH_LUC_THUONG_PHAI"), "Nói thầm (m):", fv.ns("THINH_LUC_THAM_PHAI"));
    write_lines(cur, lx, rx, "Các bất thường về tai mũi họng (nếu có):", fv.ns("BENH_TAI_MUI_HONG"), 2);

    bx.divider(cur, Some("3. Răng - Hàm - Mặt:"));
    draw_inline_slot(cur, lx, rx, cur.y, "Hàm trên:", fv.ns("RHM_HAM_TREN"), false);
    cur.y -= LINE_H;
    draw_inline_slot(cur, lx, rx, cur.y, "Hàm dưới:", fv.ns("RHM_HAM_DUOI"), false);
    cur.y -= LINE_H;
    write_lines(cur, lx, rx, "- Các bất thường khác về răng hàm mặt (nếu có):", fv.ns("BENH_RANG_HAM_MAT"), 2);

    bx.divider(cur, Some("4. Cơ xương, thần kinh và tâm thần:"));
    bx.ensure_space(cur, 34.0);
    option_row(cur, lx, rx, "- Vận động không đối xứng:", &["Không", "Có"], fv.ns("VAN_DONG_KHONG_DOI_XUNG"), 2);
    option_row(cur, lx, rx, "- Kiểm tra lưng, cột sống:", &["Bình thường", "Không bình thường"], fv.ns("KIEM_TRA_LUNG_COT_SONG"), 2);
    option_row(cur, lx, rx, "- Quan sát dáng đi:", &["Bình thường", "Không bình thường"], fv.ns("QUAN_SAT_DANG_DI"), 2);
    option_row(cur, lx, rx, "- Biểu hiện buồn bã, lo âu, mất ngủ kéo dài:", &["Có", "Không"], fv.ns("BIEU_HIEN_LO_AU_MAT_NGU"), 2);
    option_row(cur, lx, rx, "- Hành vi bất thường / nguy cơ tự gây hại:", &["Có", "Không"], fv.ns("HANH_VI_BAT_THUONG"), 2);
    let ghi_ro = [fv.ns("GHI_RO_LO_AU_MAT_NGU"), fv.ns("GHI_RO_HANH_VI_BAT_THUONG")]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ; ");
    write_lines(cur, lx, rx, "Nếu có đề nghị ghi rõ:", &ghi_ro, 1);

    bx.end(cur);
}

/// Trả lại tham chiếu khả biến của con trỏ (giúp gọi các hàm tiện ích rõ ràng).
fn c_ref<'a, 'b>(cur: &'a mut PdfPageCursor<'b>) -> &'a mut PdfPageCursor<'b> {
    cur
}

/// Vẽ III. Kết luận chung & chữ ký 2 cột (Người kết luận / Giám đốc chuyên môn).
fn render_conclusion(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let max_x = max_content_x();
    cur.ensure_space(56.0);
    cur.draw_section_heading("III. KẾT LUẬN CHUNG:");

    let kl_sk = [fv.ns("KET_LUAN_TINH_TRANG_SK"), fv.ns("KET_LUAN_BENH")]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" - ");
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "Tình trạng sức khỏe, mắc các bệnh tật (nếu có):",
        &kl_sk,
        2,
    );

    let tv_parts = [
        if fv.ns_yes_no("CAN_TU_VAN") == Some(true) { "Cần tư vấn" } else { "" },
        if fv.ns_yes_no("CHUYEN_CSKCB") == Some(true) { "Chuyển cơ sở KCB" } else { "" },
        fv.ns("GHI_RO_CHUYEN_CSKCB"),
    ]
    .into_iter()
    .filter(|s| !s.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" - ");
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "Cần được tư vấn/ chuyển khám chuyên khoa:",
        &tv_parts,
        2,
    );

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        None,
        Some("NGƯỜI KẾT LUẬN"),
        "(Ký, ghi rõ họ tên)",
        "GIÁM ĐỐC CHUYÊN MÔN",
        "(Ký, ghi rõ họ tên và đóng dấu)",
        "",
    );
}

