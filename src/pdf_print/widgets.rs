use super::cursor::PdfPageCursor;
use super::primitives::{
    approx_text_w_mm, draw_checkbox, draw_dashed_line, draw_dotted_line, draw_line,
    draw_rect_stroke, draw_text, draw_text_centered, wrap_lines, BOTTOM_Y, CONTENT_W, FONT_BODY,
    FONT_SMALL, LINE_H, MARGIN_L, MARGIN_R, PAGE_W,
};

/// Vẽ nhãn và n dòng kẻ chấm để điền tay; nếu có giá trị thì tự động ngắt dòng in lên trên.
pub fn write_lines(
    cur: &mut PdfPageCursor,
    x: f32,
    max_x: f32,
    label: &str,
    val: &str,
    min_lines: usize,
) {
    let min_l = min_lines.max(1);
    let has_lbl = !label.trim().is_empty();
    let lbl_w = if has_lbl {
        approx_text_w_mm(label, FONT_BODY)
    } else {
        0.0
    };
    let vx = if has_lbl { x + lbl_w + 2.0 } else { x };
    let val_trim = val.trim();

    let first_avail = (max_x - vx - 1.5).max(20.0);
    let mut lines = if val_trim.is_empty() {
        Vec::new()
    } else if has_lbl && vx < max_x - 22.0 {
        let mut all = wrap_lines(val_trim, first_avail, FONT_BODY);
        if all.len() > 1 {
            let rest = all[1..].join(" ");
            let mut cont = wrap_lines(&rest, max_x - x - 2.0, FONT_BODY);
            all.truncate(1);
            all.append(&mut cont);
        }
        all
    } else {
        wrap_lines(val_trim, max_x - x - 2.0, FONT_BODY)
    };

    let total_lines = min_l.max(lines.len());
    cur.ensure_space((total_lines as f32) * LINE_H + 2.0);

    if has_lbl {
        draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, x, cur.y, label);
    }

    if has_lbl && vx < max_x - 22.0 {
        draw_dotted_line(&cur.layer, vx, cur.y - 0.9, max_x);
        if !lines.is_empty() {
            draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, vx + 0.8, cur.y, &lines[0]);
            lines.remove(0);
        }
        cur.y -= LINE_H;
        let rem = (total_lines - 1).max(lines.len());
        for i in 0..rem {
            draw_dotted_line(&cur.layer, x, cur.y - 0.9, max_x);
            if let Some(txt) = lines.get(i) {
                draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, x + 1.0, cur.y, txt);
            }
            cur.y -= LINE_H;
        }
    } else {
        if has_lbl {
            cur.y -= LINE_H;
        }
        for i in 0..total_lines {
            draw_dotted_line(&cur.layer, x, cur.y - 0.9, max_x);
            if let Some(txt) = lines.get(i) {
                draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, x + 1.0, cur.y, txt);
            }
            cur.y -= LINE_H;
        }
    }
}

/// Vẽ một hàng gồm nhãn và các ô checkbox lựa chọn; tự xuống hàng khi nhiều lựa chọn.
pub fn option_row(
    cur: &mut PdfPageCursor,
    x: f32,
    max_x: f32,
    label: &str,
    options: &[&str],
    selected: &str,
    cols_per_row: usize,
) {
    let lbl_w = approx_text_w_mm(label, FONT_BODY);
    let cols = cols_per_row.max(1);
    let put_on_next_line = lbl_w > (max_x - x) * 0.48 || options.len() > 3;

    let rows_needed = if put_on_next_line {
        1 + (options.len() + cols - 1) / cols
    } else {
        1
    };
    cur.ensure_space((rows_needed as f32) * LINE_H + 1.5);

    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, x, cur.y, label);

    let opt_start_x = if put_on_next_line {
        cur.y -= LINE_H;
        x + 4.0
    } else {
        (x + lbl_w + 4.0).max(x + 56.0)
    };
    let avail_w = (max_x - opt_start_x).max(40.0);
    let col_w = avail_w / (cols as f32);

    for (idx, opt) in options.iter().enumerate() {
        let col_idx = idx % cols;
        if idx > 0 && col_idx == 0 {
            cur.y -= LINE_H;
        }
        let cx = opt_start_x + (col_idx as f32) * col_w;
        let checked = !selected.trim().is_empty() && selected.trim().eq_ignore_ascii_case(opt.trim());
        draw_checkbox(&cur.layer, cx, cur.y, checked);
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, cx + 4.6, cur.y, opt);
    }
    cur.y -= LINE_H;
}

/// Trạng thái một khung viền bao quanh mục (tự cắt khung sạch sẽ khi sang trang mới).
pub struct BoxedSection {
    pub x: f32,
    pub w: f32,
    pub top_y: f32,
}

impl BoxedSection {
    /// Bắt đầu khung viền tại vị trí con trỏ hiện tại và vẽ tiêu đề khung (nếu có).
    pub fn begin(cur: &mut PdfPageCursor, title: Option<&str>) -> Self {
        cur.ensure_space(16.0);
        let top_y = cur.y + 3.5;
        let bx = MARGIN_L;
        let bw = CONTENT_W;

        if let Some(t) = title {
            draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, bx + 2.5, cur.y - 1.2, t);
            cur.y -= 6.2;
            draw_line(&cur.layer, bx, cur.y + 2.2, bx + bw, cur.y + 2.2, 0.45);
            cur.y -= 2.2;
        } else {
            cur.y -= 1.0;
        }

        Self {
            x: bx,
            w: bw,
            top_y,
        }
    }

    /// Vẽ đường phân cách ngang bên trong khung trước khi sang nhóm con tiếp theo.
    pub fn divider(&mut self, cur: &mut PdfPageCursor, subtitle: Option<&str>) {
        self.divider_with_min_space(cur, subtitle, 14.0);
    }

    /// Vẽ đường phân cách ngang kèm ngưỡng chỗ trống tối thiểu để tránh tiêu đề đứng mồ côi cuối trang.
    pub fn divider_with_min_space(
        &mut self,
        cur: &mut PdfPageCursor,
        subtitle: Option<&str>,
        min_space: f32,
    ) {
        self.ensure_space(cur, min_space);
        draw_line(
            &cur.layer,
            self.x,
            cur.y + 2.4,
            self.x + self.w,
            cur.y + 2.4,
            0.45,
        );
        cur.y -= 1.8;
        if let Some(sub) = subtitle {
            draw_text(
                &cur.layer,
                &cur.fonts.bold,
                FONT_BODY,
                self.x + 2.5,
                cur.y,
                sub,
            );
            cur.y -= LINE_H + 0.3;
        }
    }

    /// Kiểm tra chỗ trống bên trong khung; nếu hết trang thì đóng khung cũ và mở khung mới.
    pub fn ensure_space(&mut self, cur: &mut PdfPageCursor, needed_mm: f32) {
        if cur.y - needed_mm < BOTTOM_Y + 2.0 {
            let bottom = cur.y + 2.0;
            if self.top_y > bottom + 1.0 {
                draw_rect_stroke(&cur.layer, self.x, bottom, self.w, self.top_y - bottom, 0.55);
            }
            cur.next_page();
            self.top_y = cur.y + 3.5;
            cur.y -= 1.0;
        }
    }

    /// Đóng khung viền khi kết thúc nội dung bên trong.
    pub fn end(self, cur: &mut PdfPageCursor) {
        let bottom = cur.y + 1.5;
        if self.top_y > bottom + 1.0 {
            draw_rect_stroke(&cur.layer, self.x, bottom, self.w, self.top_y - bottom, 0.55);
        }
        cur.y = bottom - 2.0;
    }
}

/// Vẽ bảng kiểm Có / Không (hoặc Có / Không / Không nhớ rõ) có kẻ ô và tự phân trang.
pub fn draw_checklist_table(
    cur: &mut PdfPageCursor,
    col_title: &str,
    with_stt: bool,
    with_unknown: bool,
    rows: &[(&str, &str, Option<u8>)], // (stt, nội dung, Some(0=Có, 1=Không, 2=Không rõ))
) {
    let w_stt = if with_stt { 11.0 } else { 0.0 };
    let w_opt = if with_unknown { 22.0 } else { 18.0 };
    let opt_count = if with_unknown { 3.0 } else { 2.0 };
    let w_name = CONTENT_W - w_stt - w_opt * opt_count;

    let draw_hdr = |c: &mut PdfPageCursor| {
        let hh = 6.2;
        let mut cx = MARGIN_L;
        if with_stt {
            draw_rect_stroke(&c.layer, cx, c.y - hh, w_stt, hh, 0.55);
            draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, cx + w_stt / 2.0, c.y - 4.4, "STT");
            cx += w_stt;
        }
        draw_rect_stroke(&c.layer, cx, c.y - hh, w_name, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, cx + w_name / 2.0, c.y - 4.4, col_title);
        cx += w_name;

        draw_rect_stroke(&c.layer, cx, c.y - hh, w_opt, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, cx + w_opt / 2.0, c.y - 4.4, "Có");
        cx += w_opt;

        draw_rect_stroke(&c.layer, cx, c.y - hh, w_opt, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, cx + w_opt / 2.0, c.y - 4.4, "Không");
        if with_unknown {
            cx += w_opt;
            draw_rect_stroke(&c.layer, cx, c.y - hh, w_opt, hh, 0.55);
            draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, cx + w_opt / 2.0, c.y - 4.4, "Không rõ");
        }
        c.y -= hh;
    };

    cur.ensure_space(14.0);
    draw_hdr(cur);

    for &(stt, label, state) in rows {
        let lines = wrap_lines(label, w_name - 4.0, FONT_BODY);
        let rh = (lines.len().max(1) as f32) * 4.4 + 1.6;
        if cur.y - rh < BOTTOM_Y {
            cur.next_page();
            draw_hdr(cur);
        }

        let mut cx = MARGIN_L;
        let mid_y = cur.y - rh / 2.0 - 1.2;
        if with_stt {
            draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_stt, rh, 0.45);
            draw_text_centered(&cur.layer, &cur.fonts.regular, FONT_BODY, cx + w_stt / 2.0, mid_y, stt);
            cx += w_stt;
        }
        draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_name, rh, 0.45);
        for (li, line) in lines.iter().enumerate() {
            draw_text(
                &cur.layer,
                &cur.fonts.regular,
                FONT_BODY,
                cx + 2.0,
                cur.y - 4.2 - (li as f32) * 4.4,
                line,
            );
        }
        cx += w_name;

        draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_opt, rh, 0.45);
        draw_checkbox(&cur.layer, cx + w_opt / 2.0 - 1.65, mid_y, state == Some(0));
        cx += w_opt;

        draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_opt, rh, 0.45);
        draw_checkbox(&cur.layer, cx + w_opt / 2.0 - 1.65, mid_y, state == Some(1));
        if with_unknown {
            cx += w_opt;
            draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_opt, rh, 0.45);
            draw_checkbox(&cur.layer, cx + w_opt / 2.0 - 1.65, mid_y, state == Some(2));
        }
        cur.y -= rh;
    }
}

/// Vẽ bảng tiền sử bệnh 2 nửa song song (mẫu Trên 18: 8 cột; mẫu Lái xe: 6 cột).
pub fn draw_ts_table_double(
    cur: &mut PdfPageCursor,
    with_tt: bool,
    left_items: &[(&str, &str, Option<bool>)],
    right_items: &[(&str, &str, Option<bool>)],
) {
    let half_w = CONTENT_W / 2.0;
    let w_tt = if with_tt { 8.5 } else { 0.0 };
    let w_co = 10.5;
    let w_kh = 12.5;
    let w_name = half_w - w_tt - w_co - w_kh;

    let draw_half_hdr = |c: &mut PdfPageCursor, x0: f32, tt_lbl: &str, hh: f32| {
        let mut cx = x0;
        if with_tt {
            draw_rect_stroke(&c.layer, cx, c.y - hh, w_tt, hh, 0.55);
            draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, cx + w_tt / 2.0, c.y - 4.8, tt_lbl);
            cx += w_tt;
        }
        draw_rect_stroke(&c.layer, cx, c.y - hh, w_name, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, cx + w_name / 2.0, c.y - 4.8, "Tên bệnh, tật");
        cx += w_name;
        draw_rect_stroke(&c.layer, cx, c.y - hh, w_co, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, cx + w_co / 2.0, c.y - 4.8, "Có");
        cx += w_co;
        draw_rect_stroke(&c.layer, cx, c.y - hh, w_kh, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_SMALL, cx + w_kh / 2.0, c.y - 4.8, "Không");
    };

    let draw_hdr = |c: &mut PdfPageCursor| {
        let hh = 6.2;
        draw_half_hdr(c, MARGIN_L, "TT", hh);
        draw_half_hdr(c, MARGIN_L + half_w, "STT", hh);
        c.y -= hh;
    };

    cur.ensure_space(14.0);
    draw_hdr(cur);

    let rows_count = left_items.len().max(right_items.len());
    for i in 0..rows_count {
        let l_opt = left_items.get(i).copied();
        let r_opt = right_items.get(i).copied();

        let l_lines = l_opt
            .map(|(_, lbl, _)| wrap_lines(lbl, w_name - 2.5, FONT_SMALL))
            .unwrap_or_default();
        let r_lines = r_opt
            .map(|(_, lbl, _)| wrap_lines(lbl, w_name - 2.5, FONT_SMALL))
            .unwrap_or_default();
        let max_l = l_lines.len().max(r_lines.len()).max(1);
        let rh = (max_l as f32) * 3.9 + 1.8;

        if cur.y - rh < BOTTOM_Y {
            cur.next_page();
            draw_hdr(cur);
        }

        for (side_idx, (item, lines)) in [(l_opt, l_lines), (r_opt, r_lines)].into_iter().enumerate() {
            let x0 = MARGIN_L + (side_idx as f32) * half_w;
            let mut cx = x0;
            let mid_y = cur.y - rh / 2.0 - 1.2;

            if with_tt {
                draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_tt, rh, 0.45);
                if let Some((tt, _, _)) = item {
                    draw_text_centered(&cur.layer, &cur.fonts.regular, FONT_SMALL, cx + w_tt / 2.0, mid_y, tt);
                }
                cx += w_tt;
            }

            draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_name, rh, 0.45);
            for (li, line) in lines.iter().enumerate() {
                draw_text(
                    &cur.layer,
                    &cur.fonts.regular,
                    FONT_SMALL,
                    cx + 1.5,
                    cur.y - 4.0 - (li as f32) * 3.9,
                    line,
                );
            }
            cx += w_name;

            draw_rect_stroke(&cur.layer, cx, cur.y - rh, w_co, rh, 0.45);
            draw_rect_stroke(&cur.layer, cx + w_co, cur.y - rh, w_kh, rh, 0.45);
            if let Some((_, _, yn)) = item {
                draw_checkbox(&cur.layer, cx + w_co / 2.0 - 1.65, mid_y, yn == Some(true));
                draw_checkbox(&cur.layer, cx + w_co + w_kh / 2.0 - 1.65, mid_y, yn == Some(false));
            }
        }
        cur.y -= rh;
    }
}

/// Vẽ phần đầu bảng 2 cột (Nội dung khám | Họ tên, chữ ký Bác sỹ), hỗ trợ ngắt dòng bằng '\n'.
pub fn draw_exam_table_header(cur: &mut PdfPageCursor, col1_w: f32, col2_w: f32, h1: &str, h2: &str) {
    let h1_lines: Vec<&str> = h1.split('\n').collect();
    let h2_lines: Vec<&str> = h2.split('\n').collect();
    let max_l = h1_lines.len().max(h2_lines.len()).max(1);
    let hh = if max_l > 1 { 10.5 } else { 7.5 };
    cur.ensure_space(hh + 10.0);
    draw_rect_stroke(&cur.layer, MARGIN_L, cur.y - hh, col1_w, hh, 0.55);
    draw_rect_stroke(&cur.layer, MARGIN_L + col1_w, cur.y - hh, col2_w, hh, 0.55);

    let draw_hdr_lines = |lines: &[&str], cx: f32| {
        let start_y = if lines.len() > 1 {
            cur.y - 4.4
        } else {
            cur.y - 5.2
        };
        for (i, line) in lines.iter().enumerate() {
            draw_text_centered(
                &cur.layer,
                &cur.fonts.bold,
                FONT_BODY,
                cx,
                start_y - (i as f32) * 4.3,
                line,
            );
        }
    };
    draw_hdr_lines(&h1_lines, MARGIN_L + col1_w / 2.0);
    draw_hdr_lines(&h2_lines, MARGIN_L + col1_w + col2_w / 2.0);
    cur.y -= hh;
}

/// Vẽ hàng tiêu đề nhóm span toàn bộ chiều rộng bảng (ví dụ "1. Nội khoa").
pub fn draw_exam_group_banner(cur: &mut PdfPageCursor, col1_w: f32, col2_w: f32, title: &str) {
    let rh = 6.5;
    if cur.y - rh - 10.0 < BOTTOM_Y {
        cur.next_page();
        draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký của Bác sỹ");
    }
    draw_rect_stroke(&cur.layer, MARGIN_L, cur.y - rh, col1_w + col2_w, rh, 0.5);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L + 2.0, cur.y - 4.7, title);
    cur.y -= rh;
}

/// Vẽ một mục khám chuẩn gồm ô Kết quả, ô Phân loại (hoặc Kết luận) và ô chữ ký BS bên phải.
pub fn draw_exam_spec_row(
    cur: &mut PdfPageCursor,
    col1_w: f32,
    col2_w: f32,
    title: &str,
    result_val: &str,
    pl_label: &str,
    pl_val: &str,
    doctor: &str,
    result_lines_min: usize,
) {
    let res_lines = wrap_lines(result_val, col1_w - 6.0, FONT_BODY);
    let n_res = result_lines_min.max(res_lines.len());
    let h_top = 5.5 + (n_res as f32) * LINE_H + 1.5;
    let h_pl = 6.5;
    let total_h = h_top + h_pl;

    if cur.y - total_h < BOTTOM_Y {
        cur.next_page();
        draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám (tiếp theo)", "Họ tên, chữ ký của Bác sỹ");
    }

    let y_top = cur.y;
    // Khung ô trái bao trọn kết quả + phân loại, ngăn giữa bằng nét gạch đứt
    draw_rect_stroke(&cur.layer, MARGIN_L, y_top - total_h, col1_w, total_h, 0.45);
    draw_dashed_line(
        &cur.layer,
        MARGIN_L + 1.0,
        y_top - h_top,
        MARGIN_L + col1_w - 1.0,
        1.6,
        1.2,
        0.35,
    );
    // Khung ô phải (chữ ký bác sĩ)
    draw_rect_stroke(&cur.layer, MARGIN_L + col1_w, y_top - total_h, col2_w, total_h, 0.45);

    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L + 2.0, y_top - 4.6, title);
    let mut ry = y_top - 4.6 - LINE_H;
    for i in 0..n_res {
        draw_dotted_line(&cur.layer, MARGIN_L + 2.0, ry - 0.8, MARGIN_L + col1_w - 2.0);
        if let Some(txt) = res_lines.get(i) {
            draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 2.5, ry, txt);
        }
        ry -= LINE_H;
    }

    // Hàng phân loại / kết luận
    let py = y_top - h_top - 4.6;
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 2.0, py, pl_label);
    let pl_w = approx_text_w_mm(pl_label, FONT_BODY);
    let px = MARGIN_L + 2.0 + pl_w + 2.0;
    draw_dotted_line(&cur.layer, px, py - 0.8, MARGIN_L + col1_w - 2.0);
    if !pl_val.trim().is_empty() {
        draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, px + 0.8, py, pl_val.trim());
    }

    // Tên bác sĩ (nếu có) căn giữa ô phải
    if !doctor.trim().is_empty() {
        let doc_lines = wrap_lines(doctor, col2_w - 4.0, FONT_BODY);
        let doc_start_y = y_top - total_h / 2.0 + ((doc_lines.len() as f32) * LINE_H) / 2.0 - 2.5;
        for (i, dl) in doc_lines.iter().enumerate() {
            draw_text_centered(
                &cur.layer,
                &cur.fonts.bold,
                FONT_BODY,
                MARGIN_L + col1_w + col2_w / 2.0,
                doc_start_y - (i as f32) * LINE_H,
                dl,
            );
        }
    }

    cur.y -= total_h;
}

/// Vẽ một hàng 2 cột có nhiều dòng tự do bên trái (dùng cho Mắt, TMH, RHM, CLS).
pub fn draw_exam_custom_row<F>(
    cur: &mut PdfPageCursor,
    col1_w: f32,
    col2_w: f32,
    needed_h: f32,
    doctor: &str,
    draw_left: F,
) where
    F: FnOnce(&mut PdfPageCursor, f32, f32),
{
    if cur.y - needed_h < BOTTOM_Y {
        cur.next_page();
        draw_exam_table_header(cur, col1_w, col2_w, "Nội dung khám (tiếp theo)", "Họ tên, chữ ký của Bác sỹ");
    }
    let y_top = cur.y;
    let saved_y = cur.y;
    cur.y -= 4.6;
    draw_left(cur, MARGIN_L + 2.0, MARGIN_L + col1_w - 2.0);
    let actual_h = (saved_y - cur.y + 1.5).max(needed_h);
    cur.y = y_top - actual_h;

    draw_rect_stroke(&cur.layer, MARGIN_L, cur.y, col1_w, actual_h, 0.45);
    draw_rect_stroke(&cur.layer, MARGIN_L + col1_w, cur.y, col2_w, actual_h, 0.45);

    if !doctor.trim().is_empty() {
        let doc_lines = wrap_lines(doctor, col2_w - 4.0, FONT_BODY);
        let doc_y = y_top - actual_h / 2.0 + ((doc_lines.len() as f32) * LINE_H) / 2.0 - 2.5;
        for (i, dl) in doc_lines.iter().enumerate() {
            draw_text_centered(
                &cur.layer,
                &cur.fonts.bold,
                FONT_BODY,
                MARGIN_L + col1_w + col2_w / 2.0,
                doc_y - (i as f32) * LINE_H,
                dl,
            );
        }
    }
}

/// Vẽ nhãn + giá trị ngắn trên một vùng xác định (dùng trong ô bảng hoặc khung).
pub fn draw_inline_slot(
    cur: &PdfPageCursor,
    x: f32,
    max_x: f32,
    y: f32,
    label: &str,
    val: &str,
    bold_val: bool,
) {
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, x, y, label);
    let lw = approx_text_w_mm(label, FONT_BODY);
    let vx = x + lw + 1.5;
    draw_dotted_line(&cur.layer, vx, y - 0.8, max_x);
    if !val.trim().is_empty() {
        let f = if bold_val {
            &cur.fonts.bold
        } else {
            &cur.fonts.regular
        };
        draw_text(&cur.layer, f, FONT_BODY, vx + 0.8, y, val.trim());
    }
}

/// Vẽ hàng 2 ô inline cạnh nhau trong một khung (x..max_x).
pub fn draw_inline_two(
    cur: &mut PdfPageCursor,
    x: f32,
    max_x: f32,
    l_lbl: &str,
    l_val: &str,
    r_lbl: &str,
    r_val: &str,
) {
    let mid = x + (max_x - x) * 0.50;
    draw_inline_slot(cur, x, mid - 2.5, cur.y, l_lbl, l_val, false);
    draw_inline_slot(cur, mid, max_x, cur.y, r_lbl, r_val, false);
    cur.y -= LINE_H;
}

/// Vẽ trường nhãn + giá trị tự xuống dòng nếu dài bên trong khung (không tràn viền phải).
pub fn draw_boxed_wrapped_slot(
    bx: &mut BoxedSection,
    cur: &mut PdfPageCursor,
    x: f32,
    max_x: f32,
    label: &str,
    val: &str,
) {
    let lw = approx_text_w_mm(label, FONT_BODY);
    let vx = x + lw + 1.5;
    let first_avail = (max_x - vx - 1.5).max(20.0);
    let val_trim = val.trim();
    let lines = if val_trim.is_empty() {
        Vec::new()
    } else if vx < max_x - 22.0 {
        let mut all = wrap_lines(val_trim, first_avail, FONT_BODY);
        if all.len() > 1 {
            let rest = all[1..].join(" ");
            let mut cont = wrap_lines(&rest, max_x - x - 2.0, FONT_BODY);
            all.truncate(1);
            all.append(&mut cont);
        }
        all
    } else {
        wrap_lines(val_trim, max_x - x - 2.0, FONT_BODY)
    };
    let n_lines = lines.len().max(1);
    bx.ensure_space(cur, (n_lines as f32) * LINE_H + 1.5);

    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, x, cur.y, label);
    if lines.is_empty() {
        draw_dotted_line(&cur.layer, vx, cur.y - 0.8, max_x);
        cur.y -= LINE_H;
        return;
    }

    draw_dotted_line(&cur.layer, vx, cur.y - 0.8, max_x);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, vx + 0.8, cur.y, &lines[0]);
    cur.y -= LINE_H;
    for line in &lines[1..] {
        draw_dotted_line(&cur.layer, x, cur.y - 0.8, max_x);
        draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, x + 0.8, cur.y, line);
        cur.y -= LINE_H;
    }
}

/// Vẽ đường kẻ đơn giản cho phần không dùng BoxedSection.
pub fn max_content_x() -> f32 {
    PAGE_W - MARGIN_R
}

