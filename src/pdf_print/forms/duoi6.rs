use printpdf::{PdfDocumentReference, PdfLayerReference};
use serde_json::{Map, Value};

use crate::config::AppConfig;

use super::super::cursor::PdfPageCursor;
use super::super::primitives::{draw_checkbox, draw_text, FONT_BODY, LINE_H};
use super::super::values::FormValues;
use super::super::widgets::{
    draw_boxed_wrapped_slot, draw_checklist_table, draw_inline_slot, draw_inline_two, option_row,
    write_lines, BoxedSection,
};

/// Vẽ một hàng ô tick cho Dưới 6 tuổi; hỗ trợ đối chiếu nhãn hiển thị với giá trị lưu cũ.
fn d6_opt_row(
    cls_box: &mut BoxedSection,
    cur: &mut PdfPageCursor,
    lx: f32,
    rx: f32,
    label: &str,
    field_key: &str,
    fv: &FormValues,
    display_opts: &[&str],
    cols: usize,
) {
    cls_box.ensure_space(cur, 11.0);
    let val = fv.ns(field_key);
    let selected = if !val.is_empty() && display_opts.len() == 2 && val.eq_ignore_ascii_case("Có") {
        // Nếu DB lưu "Có" nhưng mẫu hiển thị mô tả cụ thể (vd: "Có cơn ngưng thở...", "Trật khớp háng")
        display_opts[1]
    } else if !val.is_empty() && display_opts.len() == 2 && val.eq_ignore_ascii_case("Không bình thường") {
        display_opts[1]
    } else {
        val
    };
    option_row(cur, lx, rx, label, display_opts, selected, cols);
}

/// Vẽ toàn bộ phiếu Giấy khám sức khỏe dùng cho trẻ em dưới 06 tuổi theo template6.docx.
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
    let fv = FormValues::new(form, cat_map, is_reception_print, "DUOI_6");
    let ma_tn = fv.admin("MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);

    let so_phieu = fv.admin("SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSK-{}/{}",
        if so_phieu.is_empty() { "......................" } else { so_phieu },
        cfg.ma_cskcb,
        fv.exam_year_2d()
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG",
        "CHO TRẺ EM DƯỚI 06 TUỔI",
    );

    render_admin_and_history_boxed(&fv, &mut cur, cfg);
    render_exam_boxed(&fv, &mut cur, cfg);
}

/// Vẽ khung Thông tin hành chính & Tiền sử + Cam đoan trên trang 1 (Dưới 6 tuổi).
fn render_admin_and_history_boxed(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let mut bx = BoxedSection::begin(cur, Some("THÔNG TIN HÀNH CHÍNH"));
    let lx = bx.x + 2.5;
    let rx = bx.x + bx.w - 2.5;
    let mid = lx + (rx - lx) * 0.50;

    draw_inline_slot(
        cur,
        lx,
        rx,
        cur.y,
        "1. Họ và tên (In hoa):",
        &fv.admin("HO_TEN").to_uppercase(),
        true,
    );
    cur.y -= LINE_H;
    draw_inline_slot(
        cur,
        lx,
        rx,
        cur.y,
        "2. Số CCCD/Hộ chiếu/Mã định danh:",
        fv.admin("SO_CCCD"),
        false,
    );
    cur.y -= LINE_H;

    draw_inline_slot(cur, lx, mid - 3.0, cur.y, "3. Ngày sinh:", fv.admin("NGAY_SINH"), false);
    let gt = fv.admin("GIOI_TINH");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, mid, cur.y, "4. Giới:");
    draw_checkbox(&cur.layer, mid + 15.0, cur.y, gt == "1");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, mid + 19.5, cur.y, "Nam");
    draw_checkbox(&cur.layer, mid + 31.0, cur.y, gt == "2");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, mid + 35.5, cur.y, "Nữ");
    cur.y -= LINE_H;

    let sn = fv.admin("SINH_NON");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx, cur.y, "5. Sinh non:");
    draw_checkbox(&cur.layer, lx + 23.0, cur.y, sn == "Không" || sn == "0");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx + 27.5, cur.y, "Không");
    draw_checkbox(&cur.layer, lx + 44.0, cur.y, sn == "Có" || sn == "1");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx + 48.5, cur.y, "Có");
    draw_inline_slot(
        cur,
        mid,
        rx,
        cur.y,
        "6. Tuần thai khi sinh (tuần):",
        fv.admin("TUAN_THAI_KHI_SINH"),
        false,
    );
    cur.y -= LINE_H;

    let dantoc = fv.cat("DAN_TOC", fv.admin("MA_DAN_TOC"));
    let doituong = fv.cat("DOI_TUONG", fv.admin("DOI_TUONG"));
    draw_inline_two(cur, lx, rx, "7. Dân tộc:", &dantoc, "8. Đối tượng:", &doituong);

    let nguon = fv.cat("NGUON_CHI_TRA", fv.admin("NGUON_CHI_TRA"));
    draw_inline_two(cur, lx, rx, "9. Nguồn chi trả:", &nguon, "10. Nhóm máu:", fv.admin("NHOM_MAU"));

    draw_boxed_wrapped_slot(&mut bx, cur, lx, rx, "11. Nơi ở hiện tại:", &fv.full_address());

    draw_inline_slot(
        cur,
        lx,
        rx,
        cur.y,
        "12. Họ tên người đi cùng trẻ / Người giám hộ:",
        fv.admin("HO_TEN_NGUOI_GIAM_HO"),
        false,
    );
    cur.y -= LINE_H;

    option_row(
        cur,
        lx,
        rx,
        "13. Mối quan hệ với trẻ:",
        &["Cha", "Mẹ", "Ông/bà", "Anh/chị", "Họ hàng", "Khác"],
        fv.admin("MOI_QUAN_HE_VOI_TRE"),
        6,
    );

    draw_inline_two(
        cur,
        lx,
        rx,
        "14. Số điện thoại liên hệ:",
        fv.admin_any(&["DIEN_THOAI_GIAM_HO", "DIEN_THOAI"]),
        "15. CCCD người đi cùng:",
        fv.admin("SO_CCCD_NGUOI_GIAM_HO"),
    );
    let cskcb = if cfg.ten_cskcb.trim().is_empty() {
        "Bệnh viện Bình Định"
    } else {
        cfg.ten_cskcb.trim()
    };
    draw_inline_two(
        cur,
        lx,
        rx,
        "16. Ngày khám:",
        fv.admin_any(&["NGAY_KHAM", "NGAY_VAO"]),
        "17. Cơ sở khám:",
        cskcb,
    );

    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, lx, cur.y, "18. Tiền sử:");
    cur.y -= LINE_H;

    let tsbt_yn = fv.ns_yes_no("TSBT_MAC_BENH");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx, cur.y, "- Bản thân:");
    draw_checkbox(&cur.layer, lx + 23.0, cur.y, tsbt_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx + 27.5, cur.y, "Không");
    draw_checkbox(&cur.layer, lx + 43.0, cur.y, tsbt_yn == Some(true));
    draw_inline_slot(cur, lx + 47.5, rx, cur.y, "Có (ghi rõ tên bệnh):", fv.ns("TSBT_MA_BENH"), false);
    cur.y -= LINE_H;

    let tsgd_yn = fv.ns_yes_no("TSGD_MAC_BENH");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx, cur.y, "- Gia đình:");
    draw_checkbox(&cur.layer, lx + 23.0, cur.y, tsgd_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx + 27.5, cur.y, "Không");
    draw_checkbox(&cur.layer, lx + 43.0, cur.y, tsgd_yn == Some(true));
    draw_inline_slot(cur, lx + 47.5, rx, cur.y, "Có (ghi rõ tên bệnh):", fv.ns("TSGD_MA_BENH"), false);
    cur.y -= LINE_H;

    let lao_yn = fv.ns_yes_no("TS_TIEP_XUC_LAO");
    draw_text(
        &cur.layer,
        &cur.fonts.regular,
        FONT_BODY,
        lx,
        cur.y,
        "- Tiền sử tiếp xúc với người bệnh lao:",
    );
    draw_checkbox(&cur.layer, lx + 68.0, cur.y, lao_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx + 72.5, cur.y, "Không");
    draw_checkbox(&cur.layer, lx + 90.0, cur.y, lao_yn == Some(true));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx + 94.5, cur.y, "Có");
    cur.y -= LINE_H;

    bx.divider(cur, None);
    draw_text(
        &cur.layer,
        &cur.fonts.regular,
        FONT_BODY,
        lx,
        cur.y,
        "Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật theo sự hiểu biết của tôi.",
    );
    cur.y -= LINE_H;
    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        None,
        None,
        "",
        "Người đề nghị khám sức khỏe",
        "(Hoặc cha/mẹ hoặc thân nhân - Ký, ghi rõ họ tên)",
        "",
    );
    bx.end(cur);
}

/// Vẽ Sinh tồn, Dinh dưỡng, Phát triển, Tiêm chủng, Khám lâm sàng 1..6 & Kết luận (Dưới 6).
fn render_exam_boxed(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let mut bx = BoxedSection::begin(cur, Some("ĐÁNH GIÁ DẤU HIỆU SINH TỒN"));
    let lx = bx.x + 2.5;
    let rx = bx.x + bx.w - 2.5;

    // Sinh tồn theo đúng template6.docx
    draw_inline_slot(cur, lx, lx + 42.0, cur.y, "Nhiệt độ (°C):", fv.ns("NHIET_DO"), false);
    let mut sx = lx + 45.0;
    for opt in ["Bình thường", "Sốt", "Hạ thân nhiệt"] {
        draw_checkbox(&cur.layer, sx, cur.y, fv.ns_matches("DGDHST_NHIET_DO", opt));
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, sx + 4.5, cur.y, opt);
        sx += 36.0;
    }
    cur.y -= LINE_H;

    draw_inline_slot(cur, lx, lx + 42.0, cur.y, "Mạch (lần/phút):", fv.ns("MACH"), false);
    sx = lx + 45.0;
    for opt in ["Bình thường", "Nhanh"] {
        draw_checkbox(&cur.layer, sx, cur.y, fv.ns_matches("DGDHST_MACH", opt));
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, sx + 4.5, cur.y, opt);
        sx += 36.0;
    }
    cur.y -= LINE_H;

    draw_inline_slot(cur, lx, lx + 42.0, cur.y, "Nhịp thở (lần/phút):", fv.ns("NHIP_THO"), false);
    sx = lx + 45.0;
    for opt in ["Bình thường", "Thở nhanh", "Thở chậm"] {
        draw_checkbox(&cur.layer, sx, cur.y, fv.ns_matches("DGDHST_NHIP_THO", opt));
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, sx + 4.5, cur.y, opt);
        sx += 36.0;
    }
    cur.y -= LINE_H;

    // Dinh dưỡng
    bx.divider(cur, Some("ĐÁNH GIÁ DINH DƯỠNG"));
    draw_inline_two(cur, lx, rx, "Chiều dài (cm):", fv.ns("CHIEU_DAI"), "Chiều dài/Tuổi (SD):", fv.ns("CHIEU_DAI_TUOI_SD"));
    draw_inline_two(cur, lx, rx, "Cân nặng (kg):", fv.ns("CAN_NANG"), "Cân nặng/Tuổi (SD):", fv.ns("CAN_NANG_TUOI_SD"));

    draw_inline_slot(cur, lx, lx + 42.0, cur.y, "Vòng đầu (cm):", fv.ns("VONG_DAU"), false);
    sx = lx + 45.0;
    for opt in ["Bình thường", "Đầu to", "Đầu nhỏ"] {
        draw_checkbox(&cur.layer, sx, cur.y, fv.ns_matches("DG_VONG_DAU", opt));
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, sx + 4.5, cur.y, opt);
        sx += 36.0;
    }
    cur.y -= LINE_H;

    draw_inline_slot(cur, lx, rx, cur.y, "Chu vi vòng cánh tay (mm):", fv.ns("CHU_VI_VONG_CANH_TAY"), false);
    cur.y -= LINE_H;

    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, lx, cur.y, "Tình trạng dinh dưỡng:");
    cur.y -= LINE_H;
    let dd_checks: &[(&str, &str)] = &[
        ("Bình thường", "DGDD_BINH_THUONG"),
        ("Phù dinh dưỡng", "PHU_DINH_DUONG"),
        ("Dấu hiệu thiếu máu", "DGDD_THIEU_MAU"),
        ("Dấu hiệu còi xương", "DGDD_COI_XUONG"),
        ("Suy dinh dưỡng", "SUY_DINH_DUONG"),
        ("Thừa cân / béo phì", "THUA_CAN_BEO_PHI"),
    ];
    let col_w = (rx - lx - 4.0) / 3.0;
    for (i, (lbl, key)) in dd_checks.iter().enumerate() {
        let col = i % 3;
        if i > 0 && col == 0 {
            cur.y -= LINE_H;
        }
        let cx = lx + 2.0 + (col as f32) * col_w;
        draw_checkbox(&cur.layer, cx, cur.y, fv.ns_yes_no(key) == Some(true));
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, cx + 4.5, cur.y, lbl);
    }
    cur.y -= LINE_H;
    bx.end(cur);

    let yn_to_u8 = |b: Option<bool>| -> Option<u8> { b.map(|v| if v { 0 } else { 1 }) };
    cur.draw_section_heading("ĐÁNH GIÁ PHÁT TRIỂN TINH THẦN - VẬN ĐỘNG");
    draw_checklist_table(
        cur,
        "Hành vi và năng lực trẻ theo độ tuổi",
        false,
        false,
        &[
            ("", "- Phát triển tinh thần bình thường của trẻ theo độ tuổi", yn_to_u8(fv.ns_yes_no("PT_TTBT_THEO_DO_TUOI"))),
            ("", "- Phát triển vận động bình thường của trẻ theo độ tuổi", yn_to_u8(fv.ns_yes_no("PT_VDBT_THEO_DO_TUOI"))),
            ("", "- Trẻ có nguy cơ tự kỷ (với trẻ từ 16-30 tháng tuổi)", yn_to_u8(fv.ns_yes_no("NGUY_CO_TU_KY"))),
        ],
    );

    cur.draw_section_heading("ĐÁNH GIÁ TIÊM CHỦNG");
    draw_checklist_table(
        cur,
        "Kiểm tra sổ tiêm chủng",
        false,
        false,
        &[
            ("", "- Lao (sơ sinh)", yn_to_u8(fv.ns_yes_no("TIEM_CHUNG_BCG_SS"))),
            ("", "- Viêm gan B mũi 1 (sơ sinh)", yn_to_u8(fv.ns_yes_no("TIEM_CHUNG_VGB_SS_MUI1"))),
            ("", "- Tiêm chủng đầy đủ các loại vắc xin theo độ tuổi", yn_to_u8(fv.ns_yes_no("TIEM_CHUNG_DAY_DU_THEO_DO_TUOI"))),
        ],
    );
    cur.y -= 2.0;

    // Khám lâm sàng theo đúng đề mục 1..6 của template6.docx
    let mut cls_box = BoxedSection::begin(cur, Some("KHÁM LÂM SÀNG"));
    let clx = cls_box.x + 2.5;
    let crx = cls_box.x + cls_box.w - 2.5;

    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, clx, cur.y, "1. Toàn trạng");
    cur.y -= LINE_H;
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Màu sắc da:", "MAU_SAC_DA", fv, &["Hồng hào", "Nhợt", "Tím", "Vàng", "Sạm da"], 3);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Lòng bàn tay:", "LONG_BAN_TAY", fv, &["Bình thường (không nhợt)", "Không bình thường (nhợt)"], 2);

    cls_box.divider(cur, Some("2. Đầu - cổ"));
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, clx, cur.y, "2.1. Khám đầu - cổ");
    cur.y -= LINE_H;
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Thóp (trẻ nhỏ còn thóp):", "THOP", fv, &["Bình thường", "Rộng", "Hẹp", "Thóp phồng"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Kích thước và hình dạng đầu:", "HINH_DANG_DAU", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Vận động cổ:", "VAN_DONG_CO", fv, &["Bình thường", "Giới hạn"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Khối bất thường:", "KHOI_BAT_THUONG_DAU_CO", fv, &["Không", "Có"], 2);

    cls_box.divider(cur, Some("2.2. Khám mắt"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Vị trí 2 mắt:", "VI_TRI_HAI_MAT", fv, &["Bình thường", "Hai mắt xa nhau"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Mí mắt và kết mạc:", "MI_MAT_KET_MAC", fv, &["Bình thường", "Sưng/đỏ", "Chảy ghèn/mủ"], 3);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Đồng tử (kích thước, phản xạ):", "DONG_TU", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Lác mắt:", "LAC_MAT", fv, &["Không", "Có"], 2);

    cls_box.divider(cur, Some("2.3. Khám tai"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Tai và màng nhĩ:", "TAI_MANG_NHI", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Đáp ứng với âm thanh:", "DAP_UNG_AM_THANH", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Có khối sưng sau tai:", "KHOI_SUNG_SAU_TAI", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Dấu hiệu chảy mủ, nước tai:", "CHAY_MU_NUOC_TAI", fv, &["Không", "Có"], 2);

    cls_box.divider(cur, Some("2.4. Khám mũi - họng"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Hình dạng mũi:", "HINH_DANG_MUI", fv, &["Bình thường", "Mũi to, dày", "Bất sản xương mũi"], 3);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Chảy nước mũi:", "CHAY_NUOC_MUI", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Nghẹt mũi:", "NGHET_MUI", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Họng:", "HONG", fv, &["Bình thường", "Không bình thường"], 2);

    cls_box.divider(cur, Some("2.5. Khám miệng, răng (với trẻ đã có răng)"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Hình dạng miệng:", "HINH_DANG_MIENG", fv, &["Bình thường", "Sứt môi, chẻ vòm"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Răng sữa sơ sinh:", "RANG_SUA_SO_SINH", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Hình dạng lưỡi:", "HINH_DANG_LUOI", fv, &["Bình thường", "Lưỡi to bè"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Dính thắng lưỡi:", "DINH_THANG_LUOI", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Nấm miệng:", "NAM_MIENG", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Cằm nhỏ, tụt về sau:", "CAM_NHO_TUT_VE_SAU", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Vết sâu, mảng bám, lỗ trên răng:", "SAU_MANG_BAM_LO", fv, &["Không", "Có"], 2);

    cls_box.divider(cur, Some("3. Hô hấp"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Nhịp thở không đều:", "NHIP_THO_KHONG_DEU", fv, &["Không", "Có cơn ngưng thở trên 5 giây"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Thở rút lõm lồng ngực:", "THO_RUT_LOM_LONG_NGUC", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Tiếng thở bất thường:", "TIENG_THO_BAT_THUONG", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Dấu hiệu suy hô hấp:", "SUY_HO_HAP", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Nghe phổi:", "NGHE_PHOI", fv, &["Bình thường", "Không bình thường"], 2);

    cls_box.divider(cur, Some("4. Tim mạch"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Vị trí mỏm tim:", "VI_TRI_MOM_TIM", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Mạch ngoại vi (quay-bẹn):", "MACH_NGOAI_VI", fv, &["Bắt rõ", "Mạch nhẹ", "Không bắt được"], 3);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Nghe tim (loạn nhịp, tiếng thổi):", "TIENG_TIM", fv, &["Không", "Có"], 2);

    cls_box.divider(cur, Some("5. Bụng và cơ quan sinh dục"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Hình dáng bụng, rốn:", "HINH_DANG_BUNG_RON", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Gan, lách to:", "GAN_LACH_TO", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Khối bất thường:", "KHOI_BAT_THUONG", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Lỗ hậu môn:", "LO_HAU_MON", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Cơ quan sinh dục ngoài:", "CO_QUAN_SINH_DUC_NGOAI", fv, &["Bình thường", "Không bình thường"], 2);

    cls_box.divider(cur, Some("6. Cơ xương và thần kinh"));
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Vận động không đối xứng:", "VAN_DONG_KHONG_DOI_XUNG", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Phản xạ bú:", "PHAN_XA_BU", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Phản xạ nắm:", "PHAN_XA_NAM", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Phản xạ Moro:", "PHAN_XA_MORO", fv, &["Không", "Có"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Trương lực cơ:", "TRUONG_LUC_CO", fv, &["Bình thường", "Tăng", "Giảm"], 3);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Khớp háng:", "KHOP_HANG", fv, &["Bình thường", "Trật khớp háng"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Phản xạ cơ:", "PHAN_XA_CO", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Kiểm tra lưng, cột sống:", "KIEM_TRA_LUNG_COT_SONG", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Khám tứ chi và khớp:", "TU_CHI_KHOP", fv, &["Bình thường", "Không bình thường"], 2);
    d6_opt_row(&mut cls_box, cur, clx, crx, "- Quan sát dáng đi:", "QUAN_SAT_DANG_DI", fv, &["Bình thường", "Không bình thường"], 2);

    // Kết luận & tư vấn (ngăn tiêu đề đứng mồ côi)
    cls_box.divider_with_min_space(cur, Some("KẾT LUẬN VÀ TƯ VẤN"), 42.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, clx, cur.y, "Kết luận về sức khỏe:");
    cur.y -= LINE_H;
    draw_checkbox(&cur.layer, clx, cur.y, fv.ns_yes_no("BINH_THUONG") == Some(true));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, clx + 4.5, cur.y, "Bình thường.");
    cur.y -= LINE_H;
    draw_checkbox(&cur.layer, clx, cur.y, fv.ns_yes_no("NGUY_CO_MAC_LAO") == Some(true));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, clx + 4.5, cur.y, "Có nguy cơ mắc lao (tiền sử tiếp xúc).");
    cur.y -= LINE_H;
    draw_checkbox(&cur.layer, clx, cur.y, fv.ns_yes_no("VAN_DE_SUC_KHOE") == Some(true));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, clx + 4.5, cur.y, "Có vấn đề về sức khỏe.");
    cur.y -= LINE_H;

    let ghi_ro = [fv.ns("KET_LUAN_BENH"), fv.ns("GHI_RO_VAN_DE_SUC_KHOE")]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" - ");
    write_lines(cur, clx, crx, "Ghi rõ:", &ghi_ro, 2);
    for line in super::super::primitives::wrap_lines(
        "Tư vấn và hẹn khám lần sau hoặc chuyển cơ sở khám bệnh, chữa bệnh để khám chuyên khoa, điều trị bệnh:",
        crx - clx - 6.0,
        FONT_BODY,
    ) {
        draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, clx, cur.y, &line);
        cur.y -= LINE_H;
    }
    draw_inline_two(
        cur,
        clx,
        crx,
        "Hẹn khám lần sau:",
        fv.ns("HEN_KHAM_LAN_SAU"),
        "Chuyển cơ sở KCB:",
        if fv.blank { "" } else { fv.ns("CHUYEN_CSKCB") },
    );
    cls_box.end(cur);

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
