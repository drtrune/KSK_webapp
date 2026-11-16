use printpdf::{PdfDocumentReference, PdfLayerReference};
use serde_json::{Map, Value};

use crate::config::AppConfig;

use super::super::cursor::{draw_label_val_wrapped, PdfPageCursor};
use super::super::primitives::{
    draw_checkbox, draw_dashed_line, draw_rect_stroke, draw_text, draw_text_centered, wrap_lines,
    CONTENT_W, FONT_BODY, FONT_SMALL, LINE_H, MARGIN_L, MARGIN_R, PAGE_W,
};
use super::super::values::FormValues;
use super::super::widgets::{
    draw_exam_custom_row, draw_exam_spec_row, draw_exam_table_header, draw_inline_slot,
    draw_inline_two, draw_ts_table_double, max_content_x, write_lines,
};

/// Vẽ toàn bộ phiếu Giấy khám sức khỏe của người lái xe, người điều khiển xe máy chuyên dùng.
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
    let fv = FormValues::new(form, cat_map, is_reception_print, "LAI_XE");
    let ma_tn = fv.admin("MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);

    let so_phieu = fv.admin("SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSKLX/{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        cfg.ma_cskcb,
        fv.exam_year_2d()
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "MẪU GIẤY KHÁM SỨC KHỎE",
        "CỦA NGƯỜI LÁI XE, NGƯỜI ĐIỀU KHIỂN XE MÁY CHUYÊN DÙNG",
    );

    render_photo_and_admin(&fv, &mut cur);
    render_history(&fv, &mut cur, cfg);

    cur.next_page();
    render_clinical(&fv, &mut cur);
    render_paraclinical_and_conclusion(&fv, &mut cur, cfg);
}

/// Vẽ khung ảnh 4x6 bên trái có chú thích và 7 dòng thông tin hành chính bên phải.
fn render_photo_and_admin(fv: &FormValues, cur: &mut PdfPageCursor) {
    cur.ensure_space(54.0);
    let photo_w = 34.0;
    let photo_h = 46.0;
    let photo_top = cur.y;
    draw_rect_stroke(&cur.layer, MARGIN_L, photo_top - photo_h, photo_w, photo_h, 0.6);

    let caption = "Ảnh (4 x 6 cm) đóng dấu giáp lai hoặc scan ảnh chụp nền trắng trong 06 tháng";
    let cap_lines = wrap_lines(caption, photo_w - 4.0, FONT_SMALL);
    let mut cy = photo_top - 10.0;
    for cl in &cap_lines {
        draw_text_centered(
            &cur.layer,
            &cur.fonts.regular,
            FONT_SMALL,
            MARGIN_L + photo_w / 2.0,
            cy,
            cl,
        );
        cy -= 4.2;
    }

    let rx = MARGIN_L + photo_w + 5.0;
    let max_x = PAGE_W - MARGIN_R;
    let mut ry = photo_top;
    ry = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        rx,
        ry,
        max_x,
        "1. Họ và tên (viết chữ in hoa):",
        &fv.admin("HO_TEN").to_uppercase(),
        FONT_BODY,
        true,
    );

    let gt = fv.admin("GIOI_TINH");
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, rx, ry, "2. Giới:");
    draw_checkbox(&cur.layer, rx + 16.0, ry, gt == "1");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, rx + 20.5, ry, "Nam");
    draw_checkbox(&cur.layer, rx + 33.0, ry, gt == "2");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, rx + 37.5, ry, "Nữ");
    ry = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        rx + 49.0,
        ry,
        max_x,
        "3. Ngày sinh:",
        &format!("{} (Tuổi: {})", fv.admin("NGAY_SINH"), fv.age()),
        FONT_BODY,
        false,
    );

    ry = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        rx,
        ry,
        max_x,
        "4. Số CCCD/Hộ chiếu/Định danh:",
        fv.admin("SO_CCCD"),
        FONT_BODY,
        false,
    );
    let cap_info = if fv.admin("NGAYCAP_CCCD").is_empty() && fv.admin("NOICAP_CCCD").is_empty() {
        String::new()
    } else {
        format!("{} - {}", fv.admin("NGAYCAP_CCCD"), fv.admin("NOICAP_CCCD"))
    };
    ry = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        rx,
        ry,
        max_x,
        "5. Cấp ngày / Tại:",
        &cap_info,
        FONT_BODY,
        false,
    );

    ry = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        rx,
        ry,
        max_x,
        "6. Chỗ ở hiện tại:",
        &fv.full_address(),
        FONT_BODY,
        false,
    );
    let hang_lx = fv.admin_any(&["HANG_BANG_LAI", "LX_HANG_BANG_LAI_KL"]);
    ry = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        rx,
        ry,
        max_x,
        "7. Đề nghị khám sức khỏe để lái xe hạng:",
        hang_lx,
        FONT_BODY,
        true,
    );

    cur.y = ry.min(photo_top - photo_h - 4.0);
}

/// Vẽ mục I. Tiền sử bệnh (gia đình, bảng 6 cột 20 mục, câu hỏi đang điều trị, cam đoan).
fn render_history(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    let max_x = max_content_x();
    cur.draw_section_heading("I. TIỀN SỬ BỆNH CỦA ĐỐI TƯỢNG KHÁM SỨC KHỎE");
    cur.ensure_space(20.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "1. Tiền sử gia đình:");
    cur.y -= LINE_H;
    let lead = "Có ai trong gia đình ông (bà) mắc một trong các bệnh: truyền nhiễm, tim mạch, đái tháo đường, lao, hen phế quản, ung thư, động kinh, rối loạn tâm thần, bệnh khác:";
    for l in wrap_lines(lead, CONTENT_W, FONT_BODY) {
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, &l);
        cur.y -= 4.7;
    }

    let tsgd_yn = fv.ns_yes_no("TSGD_MAC_BENH");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L, cur.y, "a) Không");
    draw_checkbox(&cur.layer, MARGIN_L + 18.0, cur.y, tsgd_yn == Some(false));
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 28.0, cur.y, "b) Có");
    draw_checkbox(&cur.layer, MARGIN_L + 40.0, cur.y, tsgd_yn == Some(true));
    cur.y -= LINE_H;
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "Nếu “có”, đề nghị ghi cụ thể tên bệnh:",
        fv.ns("TSGD_TEN_BENH"),
        1,
    );

    cur.ensure_space(14.0);
    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "2. Tiền sử, bệnh sử bản thân (Bác sỹ hỏi bệnh và đánh dấu X vào ô tương ứng):",
    );
    cur.y -= LINE_H;

    let left_defs: &[(&str, &str)] = &[
        ("Có bệnh hay bị thương trong 5 năm qua", "TSBT_1"),
        ("Có bệnh thần kinh hay bị thương ở đầu", "TSBT_2"),
        ("Bệnh mắt hoặc giảm thị lực (trừ đeo kính thuốc)", "TSBT_3"),
        ("Bệnh ở tai, giảm sức nghe hoặc thăng bằng", "TSBT_4"),
        ("Bệnh ở tim, nhồi máu cơ tim, bệnh tim mạch khác", "TSBT_5"),
        ("Phẫu thuật can thiệp tim - mạch (van, bắc cầu, stent...)", "TSBT_6"),
        ("Tăng huyết áp", "TSBT_7"),
        ("Khó thở", "TSBT_8"),
        ("Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn", "TSBT_9"),
        ("Bệnh thận, lọc máu", "TSBT_10"),
    ];
    let right_defs: &[(&str, &str)] = &[
        ("Đái tháo đường hoặc kiểm soát tăng đường huyết", "TSBT_11"),
        ("Bệnh tâm thần", "TSBT_12"),
        ("Mất ý thức, rối loạn ý thức", "TSBT_13"),
        ("Ngất, chóng mặt, mệt mỏi, buồn nôn hoặc ngất xỉu", "TSBT_14"),
        ("Bệnh tiêu hóa", "TSBT_15"),
        ("Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ, ngáy to", "TSBT_16"),
        ("Tai biến mạch máu não hoặc liệt", "TSBT_17"),
        ("Bệnh hoặc tổn thương cột sống", "TSBT_18"),
        ("Sử dụng rượu thường xuyên, liên tục", "TSBT_19"),
        ("Sử dụng ma túy và chất gây nghiện", "TSBT_20"),
    ];
    let map_side = |defs: &[(&'static str, &str)]| -> Vec<(&'static str, &'static str, Option<bool>)> {
        defs.iter().map(|&(lbl, k)| ("", lbl, fv.ns_yes_no(k))).collect()
    };
    let left_rows = map_side(left_defs);
    let right_rows = map_side(right_defs);
    draw_ts_table_double(cur, false, &left_rows, &right_rows);
    cur.y -= 3.5;

    write_lines(cur, MARGIN_L, max_x, "Nếu “có”, đề nghị ghi cụ thể tên bệnh:", fv.ns("TSBT_TEN_BENH"), 1);
    write_lines(
        cur,
        MARGIN_L,
        max_x,
        "3. Câu hỏi khác: Đang điều trị bệnh gì? Liệt kê thuốc & liều lượng:",
        fv.ns("TSBT_DANG_DIEU_TRI"),
        1,
    );

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        Some("Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật. Nếu sai tôi xin chịu trách nhiệm trước pháp luật."),
        None,
        "",
        "NGƯỜI ĐỀ NGHỊ KHÁM SỨC KHỎE",
        "(Ký và ghi rõ họ, tên)",
        "",
    );
}

/// Vẽ mục II. Khám lâm sàng lái xe gồm 8 chuyên khoa và bảng Thị trường / Sắc giác.
fn render_clinical(fv: &FormValues, cur: &mut PdfPageCursor) {
    cur.draw_section_heading("II. KHÁM LÂM SÀNG");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký của Bác sỹ");

    draw_exam_spec_row(cur, col1_w, col2_w, "1. Tâm thần:", fv.ns("KLS_TAM_THAN"), "Kết luận:", fv.ns("KL_TAM_THAN"), fv.ns("BS_TAM_THAN"), 2);
    draw_exam_spec_row(cur, col1_w, col2_w, "2. Thần kinh:", fv.ns("KLS_THAN_KINH"), "Kết luận:", fv.ns("KL_THAN_KINH"), fv.ns("BS_THAN_KINH"), 2);

    // 3. Mắt (bao gồm thị lực, bảng thị trường, sắc giác)
    draw_exam_custom_row(cur, col1_w, col2_w, 56.0, fv.ns("BS_MAT"), |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "3. Mắt - Thị lực nhìn xa từng mắt:");
        c.y -= LINE_H;
        draw_inline_two(c, lx, rx, "+ Không kính - MP:", fv.ns("MAT_KK_MP"), "MT:", fv.ns("MAT_KK_MT"));
        draw_inline_two(c, lx, rx, "+ Có kính - MP:", fv.ns("MAT_CK_MP"), "MT:", fv.ns("MAT_CK_MT"));
        draw_inline_two(c, lx, rx, "- Hai mắt - Không kính:", fv.ns("MAT_2M_KK"), "Có kính:", fv.ns("MAT_2M_CK"));

        // Bảng thị trường thu gọn 4 ô
        let tt_w = (rx - lx) / 2.0;
        let sub_w = tt_w / 2.0;
        let y_b = c.y + 1.0;
        draw_rect_stroke(&c.layer, lx, y_b - 6.0, tt_w, 6.0, 0.45);
        draw_rect_stroke(&c.layer, lx + tt_w, y_b - 6.0, tt_w, 6.0, 0.45);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, lx + tt_w / 2.0, y_b - 4.3, "Thị trường ngang hai mắt");
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, lx + tt_w + tt_w / 2.0, y_b - 4.3, "Thị trường đứng");
        for i in 0..4 {
            draw_rect_stroke(&c.layer, lx + (i as f32) * sub_w, y_b - 12.5, sub_w, 6.5, 0.45);
        }
        let y_opt = y_b - 10.6;
        draw_checkbox(&c.layer, lx + 2.0, y_opt, fv.ns_matches("MAT_TT_NGANG", "Bình thường"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 6.5, y_opt, "Bình thường");
        draw_checkbox(&c.layer, lx + sub_w + 2.0, y_opt, fv.ns_matches("MAT_TT_NGANG", "Hạn chế"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + sub_w + 6.5, y_opt, "Hạn chế");
        draw_checkbox(&c.layer, lx + tt_w + 2.0, y_opt, fv.ns_matches("MAT_TT_DUNG", "Bình thường"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + tt_w + 6.5, y_opt, "Bình thường");
        draw_checkbox(&c.layer, lx + tt_w + sub_w + 2.0, y_opt, fv.ns_matches("MAT_TT_DUNG", "Hạn chế"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + tt_w + sub_w + 6.5, y_opt, "Hạn chế");
        c.y = y_b - 17.5;

        // Sắc giác
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "- Sắc giác:");
        draw_checkbox(&c.layer, lx + 22.0, c.y, fv.ns_matches("MAT_SAC_GIAC", "Bình thường"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 26.5, c.y, "Bình thường");
        draw_checkbox(&c.layer, lx + 52.0, c.y, fv.ns_matches("MAT_SAC_GIAC", "Mù màu toàn bộ"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 56.5, c.y, "Mù màu toàn bộ");
        c.y -= LINE_H;
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 4.0, c.y, "Mù màu:");
        draw_checkbox(&c.layer, lx + 22.0, c.y, fv.ns_matches("MAT_SAC_GIAC", "Mù màu đỏ"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 26.5, c.y, "Đỏ");
        draw_checkbox(&c.layer, lx + 42.0, c.y, fv.ns_matches("MAT_SAC_GIAC", "Mù màu xanh lá cây"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 46.5, c.y, "Xanh lá cây");
        draw_checkbox(&c.layer, lx + 74.0, c.y, fv.ns_matches("MAT_SAC_GIAC", "Mù màu vàng"));
        draw_text(&c.layer, &c.fonts.regular, FONT_SMALL, lx + 78.5, c.y, "Vàng");
        c.y -= LINE_H;

        draw_inline_slot(c, lx, rx, c.y, "Các bệnh về mắt (nếu có):", fv.ns("MAT_BENH"), false);
        c.y -= LINE_H;
        draw_dashed_line(&c.layer, lx - 1.0, c.y + 3.8, rx + 1.0, 1.6, 1.2, 0.35);
        draw_inline_slot(c, lx, rx, c.y, "Kết luận:", fv.ns("KL_MAT"), true);
    });

    // 4. Tai - Mũi - Họng
    draw_exam_custom_row(cur, col1_w, col2_w, 28.0, fv.ns("BS_TMH"), |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "4. Tai - Mũi - Họng - Kết quả khám thính lực:");
        c.y -= LINE_H;
        draw_inline_two(c, lx, rx, "+ Tai trái - Nói thường (m):", fv.ns("TMH_TT_THUONG"), "Nói thầm (m):", fv.ns("TMH_TT_THAM"));
        draw_inline_two(c, lx, rx, "+ Tai phải - Nói thường (m):", fv.ns("TMH_TP_THUONG"), "Nói thầm (m):", fv.ns("TMH_TP_THAM"));
        draw_inline_slot(c, lx, rx, c.y, "- Các bệnh về tai mũi họng (nếu có):", fv.ns("TMH_BENH"), false);
        c.y -= LINE_H;
        draw_dashed_line(&c.layer, lx - 1.0, c.y + 3.8, rx + 1.0, 1.6, 1.2, 0.35);
        draw_inline_slot(c, lx, rx, c.y, "Kết luận:", fv.ns("KL_TMH"), true);
    });

    // 5. Tim mạch
    draw_exam_custom_row(cur, col1_w, col2_w, 24.0, fv.ns("BS_TIM_MACH"), |c, lx, rx| {
        draw_text(&c.layer, &c.fonts.bold, FONT_BODY, lx, c.y, "5. Tim mạch:");
        c.y -= LINE_H;
        draw_inline_two(c, lx, rx, "+ Mạch (lần/phút):", fv.ns("MACH"), "+ Huyết áp (mmHg):", fv.ns("HUYET_AP"));
        draw_inline_slot(c, lx, rx, c.y, "Khám tim mạch:", fv.ns("KLS_TIM_MACH"), false);
        c.y -= LINE_H;
        draw_dashed_line(&c.layer, lx - 1.0, c.y + 3.8, rx + 1.0, 1.6, 1.2, 0.35);
        draw_inline_slot(c, lx, rx, c.y, "Kết luận:", fv.ns("KL_TIM_MACH"), true);
    });

    draw_exam_spec_row(cur, col1_w, col2_w, "6. Hô hấp:", fv.ns("KLS_HO_HAP"), "Kết luận:", fv.ns("KL_HO_HAP"), fv.ns("BS_HO_HAP"), 2);
    draw_exam_spec_row(cur, col1_w, col2_w, "7. Cơ Xương Khớp:", fv.ns("KLS_CXK"), "Kết luận:", fv.ns("KL_CXK"), fv.ns("BS_CXK"), 2);
    draw_exam_spec_row(cur, col1_w, col2_w, "8. Nội tiết:", fv.ns("KLS_NOI_TIET"), "Kết luận:", fv.ns("KL_NOI_TIET"), fv.ns("BS_NOI_TIET"), 2);
}

/// Vẽ mục III. Khám cận lâm sàng và mục IV. Kết luận lái xe.
fn render_paraclinical_and_conclusion(fv: &FormValues, cur: &mut PdfPageCursor, cfg: &AppConfig) {
    cur.draw_section_heading("III. KHÁM CẬN LÂM SÀNG");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký\nBS / Trưởng BP");

    draw_exam_custom_row(cur, col1_w, col2_w, 18.0, fv.ns("BS_XN_MA_TUY"), |c, lx, rx| {
        draw_text(
            &c.layer,
            &c.fonts.bold,
            FONT_BODY,
            lx,
            c.y,
            "1. Xét nghiệm ma túy (Amphetamin, Marijuana, Morphin, Codein, Heroin):",
        );
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "Kết quả:", fv.ns("XN_MA_TUY"), true);
    });

    draw_exam_custom_row(cur, col1_w, col2_w, 22.0, fv.ns("BS_XN_KHAC"), |c, lx, rx| {
        draw_text(
            &c.layer,
            &c.fonts.bold,
            FONT_BODY,
            lx,
            c.y,
            "2. XN nồng độ cồn / huyết học / sinh hóa / X-quang (khi có chỉ định):",
        );
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "a) Kết quả:", fv.ns("XN_KHAC_KQ"), false);
        c.y -= LINE_H;
        draw_inline_slot(c, lx, rx, c.y, "b) Kết luận:", fv.ns("XN_KHAC_KL"), true);
    });

    let max_x = max_content_x();
    cur.ensure_space(56.0);
    cur.draw_section_heading("IV. KẾT LUẬN");
    let hang_kl = fv.ns("HANG_BANG_LAI_KL");
    let kl_full = [fv.ns("KET_LUAN_SK"), hang_kl, fv.ns("KET_LUAN_BENH")]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    write_lines(cur, MARGIN_L, max_x, "", &kl_full, 3);

    cur.ensure_space(34.0);
    draw_text(
        &cur.layer,
        &cur.fonts.italic,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "(Giấy khám sức khỏe này có giá trị sử dụng trong vòng 12 tháng kể từ ngày ký kết luận)",
    );
    cur.y -= 5.0;

    let date_parts = fv.exam_date_parts();
    cur.draw_signature_block(
        cfg,
        &date_parts,
        None,
        None,
        "",
        "NGƯỜI KẾT LUẬN",
        "(Ký, ghi rõ họ tên và đóng dấu)",
        fv.ns("BS_KET_LUAN"),
    );
}

