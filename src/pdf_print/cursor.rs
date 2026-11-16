use printpdf::{Mm, PdfDocumentReference, PdfLayerReference};

use crate::config::AppConfig;

use super::primitives::{
    approx_text_w_mm, draw_barcode_header, draw_checkbox, draw_dotted_line, draw_footer,
    draw_text, draw_text_centered, wrap_lines, PdfFonts, BOTTOM_Y, CONTENT_W, FONT_BODY,
    FONT_SUBTITLE, FONT_TITLE, LINE_H, MARGIN_L, MARGIN_R, PAGE_H, PAGE_W, TOP_Y,
};

/// Con trỏ theo dõi vị trí vẽ hiện tại trên trang A4 và tự động phân trang.
pub struct PdfPageCursor<'a> {
    pub doc: &'a PdfDocumentReference,
    pub fonts: &'a PdfFonts,
    pub layer: PdfLayerReference,
    pub page_no: usize,
    pub y: f32,
    pub ma_tn: &'a str,
}

impl<'a> PdfPageCursor<'a> {
    /// Khởi tạo con trỏ ở đầu trang 1 cho một hồ sơ.
    pub fn new(
        doc: &'a PdfDocumentReference,
        fonts: &'a PdfFonts,
        is_first_page_of_doc: bool,
        first_layer: Option<PdfLayerReference>,
        ma_tn: &'a str,
    ) -> Self {
        let layer = if is_first_page_of_doc {
            first_layer.unwrap()
        } else {
            let (p, l) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
            doc.get_page(p).get_layer(l)
        };
        draw_barcode_header(&layer, fonts, ma_tn);
        draw_footer(&layer, fonts, 1);
        Self {
            doc,
            fonts,
            layer,
            page_no: 1,
            y: TOP_Y,
            ma_tn,
        }
    }

    /// Sang trang mới, vẽ lại mã vạch ở đầu trang và số trang ở chân trang.
    pub fn next_page(&mut self) {
        self.page_no += 1;
        let (p, l) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
        self.layer = self.doc.get_page(p).get_layer(l);
        draw_barcode_header(&self.layer, self.fonts, self.ma_tn);
        draw_footer(&self.layer, self.fonts, self.page_no);
        self.y = TOP_Y;
    }

    /// Đảm bảo còn đủ khoảng trống theo chiều dọc (mm); nếu thiếu thì sang trang mới.
    pub fn ensure_space(&mut self, needed_mm: f32) {
        if self.y - needed_mm < BOTTOM_Y {
            self.next_page();
        }
    }

    /// Vẽ phần Quốc hiệu, tên cơ sở KCB, số phiếu và tiêu đề mẫu phiếu.
    pub fn draw_national_header(
        &mut self,
        cfg: &AppConfig,
        so_line: &str,
        title1: &str,
        title2: &str,
    ) {
        let left_cx = MARGIN_L + 36.0;
        let right_cx = MARGIN_L + 130.0;
        draw_text_centered(
            &self.layer,
            &self.fonts.regular,
            FONT_BODY,
            left_cx,
            self.y,
            "BỘ Y TẾ",
        );
        draw_text_centered(
            &self.layer,
            &self.fonts.bold,
            FONT_BODY,
            right_cx,
            self.y,
            "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM",
        );
        self.y -= 5.2;

        let cskcb = if cfg.ten_cskcb.trim().is_empty() {
            "BỆNH VIỆN BÌNH ĐỊNH"
        } else {
            cfg.ten_cskcb.trim()
        };
        draw_text_centered(
            &self.layer,
            &self.fonts.bold,
            FONT_BODY,
            left_cx,
            self.y,
            &cskcb.to_uppercase(),
        );
        draw_text_centered(
            &self.layer,
            &self.fonts.bold,
            FONT_BODY,
            right_cx,
            self.y,
            "Độc lập - Tự do - Hạnh phúc",
        );
        self.y -= 4.4;

        draw_text_centered(
            &self.layer,
            &self.fonts.regular,
            FONT_BODY,
            left_cx,
            self.y,
            "-----------",
        );
        draw_text_centered(
            &self.layer,
            &self.fonts.regular,
            FONT_BODY,
            right_cx,
            self.y,
            "----------------",
        );
        self.y -= 5.0;

        draw_text_centered(
            &self.layer,
            &self.fonts.regular,
            FONT_BODY,
            left_cx,
            self.y,
            so_line,
        );
        self.y -= 7.2;

        for line in wrap_lines(title1, CONTENT_W, FONT_TITLE) {
            draw_text_centered(
                &self.layer,
                &self.fonts.bold,
                FONT_TITLE,
                PAGE_W / 2.0,
                self.y,
                &line,
            );
            self.y -= 5.6;
        }
        if !title2.trim().is_empty() {
            for line in wrap_lines(title2, CONTENT_W, FONT_TITLE) {
                draw_text_centered(
                    &self.layer,
                    &self.fonts.bold,
                    FONT_TITLE,
                    PAGE_W / 2.0,
                    self.y,
                    &line,
                );
                self.y -= 5.6;
            }
        }
        self.y -= 2.0;
    }

    /// Vẽ tiêu đề mục lớn (ví dụ I. KHÁM THỂ LỰC), tự chừa khoảng cách phía trên.
    pub fn draw_section_heading(&mut self, heading: &str) {
        self.ensure_space(26.0);
        if self.y < TOP_Y - 1.0 {
            self.y -= 6.0;
        }
        for line in wrap_lines(heading, CONTENT_W, FONT_SUBTITLE) {
            draw_text(
                &self.layer,
                &self.fonts.bold,
                FONT_SUBTITLE,
                MARGIN_L,
                self.y,
                &line,
            );
            self.y -= 6.0;
        }
    }

    /// Vẽ tiêu đề mục căn giữa (ví dụ TIỀN SỬ BỆNH CỦA ĐỐI TƯỢNG KHÁM SỨC KHỎE).
    pub fn draw_centered_heading(&mut self, heading: &str) {
        self.ensure_space(24.0);
        if self.y < TOP_Y - 1.0 {
            self.y -= 5.5;
        }
        for line in wrap_lines(heading, CONTENT_W, FONT_SUBTITLE) {
            draw_text_centered(
                &self.layer,
                &self.fonts.bold,
                FONT_SUBTITLE,
                PAGE_W / 2.0,
                self.y,
                &line,
            );
            self.y -= 6.0;
        }
    }

    /// Vẽ 1 trường (nhãn + dòng kẻ điền tay hoặc giá trị), tự xuống dòng nếu dài.
    pub fn draw_field(&mut self, label: &str, val: &str, bold_val: bool) {
        let max_x = PAGE_W - MARGIN_R;
        let est_lines = (wrap_lines(val, CONTENT_W * 0.55, FONT_BODY).len()).max(1) as f32;
        self.ensure_space(est_lines * LINE_H + 2.0);
        self.y = draw_label_val_wrapped(
            &self.layer,
            self.fonts,
            MARGIN_L,
            self.y,
            max_x,
            label,
            val,
            FONT_BODY,
            bold_val,
        );
    }

    /// Vẽ 2 trường cạnh nhau trên cùng 1 hàng (tự rớt xuống 2 hàng nếu quá dài).
    pub fn draw_two_fields(&mut self, l_lbl: &str, l_val: &str, r_lbl: &str, r_val: &str) {
        let max_x = PAGE_W - MARGIN_R;
        let split_x = MARGIN_L + CONTENT_W * 0.50;
        let l_need =
            approx_text_w_mm(l_lbl, FONT_BODY) + approx_text_w_mm(l_val, FONT_BODY) + 16.0;
        let r_need =
            approx_text_w_mm(r_lbl, FONT_BODY) + approx_text_w_mm(r_val, FONT_BODY) + 16.0;
        if l_need <= (split_x - MARGIN_L - 3.0) && r_need <= (max_x - split_x) {
            self.ensure_space(LINE_H + 2.0);
            let y1 = draw_label_val_wrapped(
                &self.layer,
                self.fonts,
                MARGIN_L,
                self.y,
                split_x - 3.0,
                l_lbl,
                l_val,
                FONT_BODY,
                false,
            );
            let y2 = draw_label_val_wrapped(
                &self.layer,
                self.fonts,
                split_x,
                self.y,
                max_x,
                r_lbl,
                r_val,
                FONT_BODY,
                false,
            );
            self.y = y1.min(y2);
        } else {
            self.draw_field(l_lbl, l_val, false);
            self.draw_field(r_lbl, r_val, false);
        }
    }

    /// Vẽ hàng Giới tính (ô tick Nam/Nữ) + Ngày sinh + Tuổi chuẩn hành chính.
    pub fn draw_gender_dob_age(&mut self, gt: &str, ns: &str, tuoi: &str) {
        self.ensure_space(LINE_H * 2.0);
        draw_text(
            &self.layer,
            &self.fonts.bold,
            FONT_BODY,
            MARGIN_L,
            self.y,
            "2. Giới tính:",
        );
        draw_checkbox(&self.layer, MARGIN_L + 24.0, self.y, gt == "1");
        draw_text(
            &self.layer,
            &self.fonts.regular,
            FONT_BODY,
            MARGIN_L + 29.0,
            self.y,
            "Nam",
        );
        draw_checkbox(&self.layer, MARGIN_L + 42.0, self.y, gt == "2");
        draw_text(
            &self.layer,
            &self.fonts.regular,
            FONT_BODY,
            MARGIN_L + 47.0,
            self.y,
            "Nữ",
        );

        let max_x = PAGE_W - MARGIN_R;
        let y_dob = draw_label_val_wrapped(
            &self.layer,
            self.fonts,
            MARGIN_L + 58.0,
            self.y,
            MARGIN_L + 138.0,
            "3. Ngày sinh:",
            ns,
            FONT_BODY,
            false,
        );
        let y_age = draw_label_val_wrapped(
            &self.layer,
            self.fonts,
            MARGIN_L + 140.0,
            self.y,
            max_x,
            "Tuổi:",
            tuoi,
            FONT_BODY,
            true,
        );
        self.y = y_dob.min(y_age);
    }

    /// Vẽ dòng cam đoan (nếu có) bên trái và khối ký tên (1 hoặc 2 cột) ở cuối phần/trang.
    pub fn draw_signature_block(
        &mut self,
        cfg: &AppConfig,
        date_parts: &(String, String, String),
        cam_doan_text: Option<&str>,
        left_title: Option<&str>,
        left_sub: &str,
        right_title: &str,
        right_sub: &str,
        right_name: &str,
    ) {
        let need_h = if !right_name.trim().is_empty() {
            34.0
        } else if cam_doan_text.is_some() {
            22.0
        } else {
            28.0
        };
        self.ensure_space(need_h);
        if self.y < TOP_Y - 1.0 {
            self.y -= 4.0;
        }

        let dia_danh = if cfg.dia_danh.trim().is_empty() {
            "Gia Lai"
        } else {
            cfg.dia_danh.trim()
        };
        let date_line = format!(
            "{}, ngày {} tháng {} năm {}",
            dia_danh, date_parts.0, date_parts.1, date_parts.2
        );
        let sig_right_cx = MARGIN_L + 132.0;
        let sig_left_cx = MARGIN_L + 42.0;

        if let Some(cd) = cam_doan_text {
            // Bố cục 2 cột: trái là lời cam đoan, phải là ngày tháng + chữ ký người đề nghị
            let cd_lines = wrap_lines(cd, 82.0, FONT_BODY);
            let mut cy = self.y;
            for l in &cd_lines {
                draw_text(&self.layer, &self.fonts.regular, FONT_BODY, MARGIN_L, cy, l);
                cy -= 4.8;
            }
            draw_text_centered(
                &self.layer,
                &self.fonts.italic,
                FONT_BODY,
                sig_right_cx,
                self.y,
                &date_line,
            );
            self.y -= 5.2;
            draw_text_centered(
                &self.layer,
                &self.fonts.bold,
                FONT_BODY,
                sig_right_cx,
                self.y,
                right_title,
            );
            self.y -= 4.8;
            draw_text_centered(
                &self.layer,
                &self.fonts.italic,
                FONT_BODY,
                sig_right_cx,
                self.y,
                right_sub,
            );
            self.y = (self.y - 16.0).min(cy - 6.0);
            return;
        }

        draw_text_centered(
            &self.layer,
            &self.fonts.italic,
            FONT_BODY,
            sig_right_cx,
            self.y,
            &date_line,
        );
        self.y -= 5.2;

        if let Some(lt) = left_title {
            draw_text_centered(
                &self.layer,
                &self.fonts.bold,
                FONT_BODY,
                sig_left_cx,
                self.y,
                lt,
            );
        }
        draw_text_centered(
            &self.layer,
            &self.fonts.bold,
            FONT_BODY,
            sig_right_cx,
            self.y,
            right_title,
        );
        self.y -= 4.8;

        if left_title.is_some() {
            draw_text_centered(
                &self.layer,
                &self.fonts.italic,
                FONT_BODY,
                sig_left_cx,
                self.y,
                left_sub,
            );
        }
        draw_text_centered(
            &self.layer,
            &self.fonts.italic,
            FONT_BODY,
            sig_right_cx,
            self.y,
            right_sub,
        );

        if !right_name.trim().is_empty() {
            let name_cx = if left_title.is_some() {
                sig_left_cx
            } else {
                sig_right_cx
            };
            draw_text_centered(
                &self.layer,
                &self.fonts.bold,
                FONT_BODY,
                name_cx,
                self.y - 14.0,
                right_name,
            );
        }
        self.y -= 20.0;
    }
}

/// Vẽ nhãn và giá trị kèm dòng kẻ điền tay, tự ngắt dòng khi giá trị dài.
pub fn draw_label_val_wrapped(
    layer: &PdfLayerReference,
    fonts: &PdfFonts,
    x: f32,
    mut y: f32,
    max_x: f32,
    label: &str,
    val: &str,
    size_pt: f32,
    bold_val: bool,
) -> f32 {
    draw_text(layer, &fonts.bold, size_pt, x, y, label);
    let lbl_w = approx_text_w_mm(label, size_pt);
    let vx = x + lbl_w + 1.8;
    let val_trim = val.trim();
    let f = if bold_val {
        &fonts.bold
    } else {
        &fonts.regular
    };

    if vx >= max_x - 16.0 {
        y -= LINE_H;
        let lines = wrap_lines(val_trim, max_x - x - 2.0, size_pt);
        if lines.is_empty() {
            draw_dotted_line(layer, x + 2.0, y - 0.9, max_x);
            return y - LINE_H;
        }
        for line in lines {
            draw_dotted_line(layer, x + 2.0, y - 0.9, max_x);
            draw_text(layer, f, size_pt, x + 2.5, y, &line);
            y -= LINE_H;
        }
        return y;
    }

    let first_avail = max_x - vx - 1.5;
    let lines = wrap_lines(val_trim, first_avail, size_pt);
    if lines.is_empty() {
        draw_dotted_line(layer, vx, y - 0.9, max_x);
        return y - LINE_H;
    }

    draw_dotted_line(layer, vx, y - 0.9, max_x);
    draw_text(layer, f, size_pt, vx + 0.8, y, &lines[0]);
    y -= LINE_H;

    if lines.len() > 1 {
        let rest_text = lines[1..].join(" ");
        for cont in wrap_lines(&rest_text, max_x - x - 3.0, size_pt) {
            draw_dotted_line(layer, x + 2.0, y - 0.9, max_x);
            draw_text(layer, f, size_pt, x + 2.5, y, &cont);
            y -= LINE_H;
        }
    }
    y
}

