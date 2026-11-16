use std::io::BufWriter;

use printpdf::{
    Color, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference, Point,
    Rect, Rgb,
};
use serde_json::{Map, Value};

use crate::config::AppConfig;

const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN_L: f32 = 18.0;
const MARGIN_R: f32 = 15.0;
const CONTENT_W: f32 = PAGE_W - MARGIN_L - MARGIN_R; // 177mm
const TOP_Y: f32 = PAGE_H - 25.0; // 272mm (dưới barcode header)
const BOTTOM_Y: f32 = 18.0; // trên footer (- Trang X -)

const FONT_BODY: f32 = 13.0;
const FONT_SUBTITLE: f32 = 13.5;
const FONT_TITLE: f32 = 14.0;
const LINE_H: f32 = 5.2;

struct PdfFonts {
    regular: IndirectFontRef,
    bold: IndirectFontRef,
    italic: IndirectFontRef,
}

fn load_font_bytes(candidates: &[&str]) -> Result<Vec<u8>, String> {
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            return Ok(bytes);
        }
    }
    Err(format!("Không tìm thấy font hệ thống trong: {:?}", candidates))
}

fn load_fonts(doc: &PdfDocumentReference) -> Result<PdfFonts, String> {
    let reg_bytes = load_font_bytes(&[
        r"C:\Windows\Fonts\times.ttf",
        r"C:\Windows\Fonts\arial.ttf",
        r"C:\Windows\Fonts\tahoma.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf",
    ])?;
    let bold_bytes = load_font_bytes(&[
        r"C:\Windows\Fonts\timesbd.ttf",
        r"C:\Windows\Fonts\arialbd.ttf",
        r"C:\Windows\Fonts\tahomabd.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSerif-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSerif-Bold.ttf",
    ])
    .unwrap_or_else(|_| reg_bytes.clone());
    let italic_bytes = load_font_bytes(&[
        r"C:\Windows\Fonts\timesi.ttf",
        r"C:\Windows\Fonts\ariali.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSerif-Italic.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSerif-Italic.ttf",
    ])
    .unwrap_or_else(|_| reg_bytes.clone());

    let regular = doc
        .add_external_font(&reg_bytes[..])
        .map_err(|e| format!("Lỗi nạp font regular: {}", e))?;
    let bold = doc
        .add_external_font(&bold_bytes[..])
        .map_err(|e| format!("Lỗi nạp font bold: {}", e))?;
    let italic = doc
        .add_external_font(&italic_bytes[..])
        .map_err(|e| format!("Lỗi nạp font italic: {}", e))?;

    Ok(PdfFonts {
        regular,
        bold,
        italic,
    })
}

/// Code 128 patterns (values 0..=106). Each entry is 6 bar/space widths (stop has 7th bar=2).
const CODE128_PATTERNS: [&str; 107] = [
    "212222", "222122", "222221", "121223", "121322", "131222", "122213", "122312", "132212",
    "221213", "221312", "231212", "112232", "122132", "122231", "113222", "123122", "123221",
    "223211", "221132", "221231", "213212", "223112", "312131", "311222", "321122", "321221",
    "312212", "322112", "322211", "212123", "212321", "232121", "111323", "131123", "131321",
    "112313", "132113", "132311", "211313", "231113", "231311", "112133", "112331", "132131",
    "113123", "113321", "133121", "313121", "211331", "231131", "213113", "213311", "213131",
    "311123", "311321", "331121", "312113", "312311", "332111", "314111", "221411", "431111",
    "111224", "111422", "121124", "121421", "141122", "141221", "112214", "112412", "122114",
    "122411", "142112", "142211", "241211", "221114", "413111", "241112", "134111", "111242",
    "121142", "121241", "114212", "124112", "124211", "411212", "421112", "421211", "212141",
    "214121", "412121", "111143", "111341", "131141", "114113", "114311", "411113", "411311",
    "113141", "114131", "311141", "411131", "211412", "211214", "211232", "233111",
];

/// Encodes ASCII string into Code128B module widths (alternating bar, space, ... ending with bar).
pub fn encode_code128b(text: &str) -> Vec<u8> {
    let mut codes: Vec<usize> = vec![104]; // Start B
    for ch in text.chars() {
        let c = ch as u32;
        if (32..=126).contains(&c) {
            codes.push((c - 32) as usize);
        }
    }
    let mut checksum: usize = codes[0];
    for (idx, &code) in codes.iter().enumerate().skip(1) {
        checksum = (checksum + idx * code) % 103;
    }
    codes.push(checksum);
    codes.push(106); // Stop

    let mut widths = Vec::with_capacity(codes.len() * 6 + 2);
    for (i, &code) in codes.iter().enumerate() {
        for b in CODE128_PATTERNS[code].bytes() {
            widths.push(b - b'0');
        }
        if i == codes.len() - 1 {
            widths.push(2); // Stop final bar
        }
    }
    widths
}

fn black() -> Color {
    Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))
}

fn gray_line() -> Color {
    Color::Rgb(Rgb::new(0.35, 0.35, 0.35, None))
}

fn draw_line(layer: &PdfLayerReference, x1: f32, y1: f32, x2: f32, y2: f32, thickness_pt: f32) {
    layer.set_outline_color(black());
    layer.set_outline_thickness(thickness_pt);
    let line = Line {
        points: vec![
            (Point::new(Mm(x1), Mm(y1)), false),
            (Point::new(Mm(x2), Mm(y2)), false),
        ],
        is_closed: false,
    };
    layer.add_line(line);
}

fn draw_dotted_line(layer: &PdfLayerReference, x1: f32, y: f32, x2: f32) {
    if x2 <= x1 + 1.0 {
        return;
    }
    layer.set_outline_color(gray_line());
    layer.set_outline_thickness(0.4);
    let line = Line {
        points: vec![
            (Point::new(Mm(x1), Mm(y)), false),
            (Point::new(Mm(x2), Mm(y)), false),
        ],
        is_closed: false,
    };
    layer.add_line(line);
}

fn draw_rect_stroke(
    layer: &PdfLayerReference,
    x: f32,
    y_bottom: f32,
    w: f32,
    h: f32,
    thickness_pt: f32,
) {
    layer.set_outline_color(black());
    layer.set_outline_thickness(thickness_pt);
    let rect = Rect::new(Mm(x), Mm(y_bottom), Mm(x + w), Mm(y_bottom + h))
        .with_mode(printpdf::path::PaintMode::Stroke);
    layer.add_rect(rect);
}

fn draw_rect_fill(layer: &PdfLayerReference, x: f32, y_bottom: f32, w: f32, h: f32) {
    layer.set_fill_color(black());
    let rect = Rect::new(Mm(x), Mm(y_bottom), Mm(x + w), Mm(y_bottom + h))
        .with_mode(printpdf::path::PaintMode::Fill);
    layer.add_rect(rect);
}

fn draw_text(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    size_pt: f32,
    x: f32,
    y: f32,
    text: &str,
) {
    if text.is_empty() {
        return;
    }
    layer.set_fill_color(black());
    layer.use_text(text, size_pt, Mm(x), Mm(y), font);
}

/// Conservative text width estimate in mm for Times New Roman so wrapped lines never overflow.
fn approx_text_w_mm(text: &str, size_pt: f32) -> f32 {
    let pt_to_mm = 0.352778;
    let units: f32 = text
        .chars()
        .map(|c| {
            if matches!(c, 'm' | 'w' | 'M' | 'W' | 'Ô' | 'Ơ' | 'Ư' | 'Đ') {
                0.76
            } else if c.is_uppercase() {
                0.66
            } else if matches!(c, 'i' | 'l' | 'I' | '.' | ',' | ':' | ';' | ' ' | '!' | '/' | '(' | ')' | '-') {
                0.30
            } else if c.is_ascii_digit() {
                0.52
            } else {
                0.49
            }
        })
        .sum();
    units * size_pt * pt_to_mm
}

fn draw_text_centered(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    size_pt: f32,
    center_x: f32,
    y: f32,
    text: &str,
) {
    let w = approx_text_w_mm(text, size_pt);
    draw_text(layer, font, size_pt, center_x - w / 2.0, y, text);
}

fn draw_checkbox(layer: &PdfLayerReference, x: f32, y_baseline: f32, checked: bool) {
    let sz = 3.4;
    draw_rect_stroke(layer, x, y_baseline - 0.3, sz, sz, 0.6);
    if checked {
        draw_line(layer, x + 0.5, y_baseline + 0.3, x + sz - 0.5, y_baseline + sz - 0.8, 0.8);
        draw_line(layer, x + 0.5, y_baseline + sz - 0.8, x + sz - 0.5, y_baseline + 0.3, 0.8);
    }
}

fn wrap_lines(text: &str, max_w_mm: f32, size_pt: f32) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let safe_w = max_w_mm.max(15.0);
    let mut out = Vec::new();
    for para in text.split('\n') {
        let mut cur = String::new();
        for word in para.split_whitespace() {
            let candidate = if cur.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", cur, word)
            };
            if approx_text_w_mm(&candidate, size_pt) <= safe_w || cur.is_empty() {
                cur = candidate;
            } else {
                out.push(cur);
                cur = word.to_string();
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
    }
    out
}

fn draw_barcode_header(layer: &PdfLayerReference, fonts: &PdfFonts, ma_tn: &str) {
    let code = if ma_tn.trim().is_empty() {
        "000000"
    } else {
        ma_tn.trim()
    };
    let widths = encode_code128b(code);
    let total_units: u32 = widths.iter().map(|&w| w as u32).sum();
    let target_w = 42.0_f32;
    let unit_w = if total_units > 0 {
        (target_w / total_units as f32).min(0.42)
    } else {
        0.25
    };
    let actual_w = total_units as f32 * unit_w;
    let x0 = PAGE_W - MARGIN_R - actual_w;
    let y_bar = PAGE_H - 14.0;
    let bar_h = 6.5;

    let mut cx = x0;
    for (idx, &w) in widths.iter().enumerate() {
        let w_mm = w as f32 * unit_w;
        if idx % 2 == 0 {
            draw_rect_fill(layer, cx, y_bar, w_mm, bar_h);
        }
        cx += w_mm;
    }

    let cap = format!("Mã TN: {}", code);
    draw_text_centered(
        layer,
        &fonts.bold,
        FONT_BODY,
        x0 + actual_w / 2.0,
        y_bar - 4.4,
        &cap,
    );
}

fn draw_footer(layer: &PdfLayerReference, fonts: &PdfFonts, page_num: usize) {
    draw_text_centered(
        layer,
        &fonts.regular,
        FONT_BODY,
        PAGE_W / 2.0,
        9.0,
        &format!("- Trang {} -", page_num),
    );
}

fn get_s<'a>(form: &'a Map<String, Value>, key: &str) -> &'a str {
    form.get(key).and_then(|v| v.as_str()).unwrap_or("").trim()
}

fn get_any<'a>(form: &'a Map<String, Value>, keys: &[&str]) -> &'a str {
    for k in keys {
        let v = get_s(form, k);
        if !v.is_empty() {
            return v;
        }
    }
    ""
}

fn get_b(form: &Map<String, Value>, key: &str) -> bool {
    match form.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => {
            let t = s.trim().to_ascii_lowercase();
            t == "1" || t == "true" || t == "co" || t == "có"
        }
        Some(Value::Number(n)) => n.as_i64() == Some(1),
        _ => false,
    }
}

fn get_cat_name(cat_map: &Map<String, Value>, cat_key: &str, code: &str) -> String {
    let c_trim = code.trim();
    if c_trim.is_empty() {
        return String::new();
    }
    match cat_map.get(cat_key) {
        Some(Value::Object(obj)) => {
            if let Some(v) = obj.get(c_trim).and_then(|v| v.as_str()) {
                return v.to_string();
            }
            if let Ok(num) = c_trim.parse::<u64>() {
                let padded = format!("{:02}", num);
                if let Some(v) = obj.get(&padded).and_then(|v| v.as_str()) {
                    return v.to_string();
                }
                let unpadded = num.to_string();
                if let Some(v) = obj.get(&unpadded).and_then(|v| v.as_str()) {
                    return v.to_string();
                }
            }
        }
        Some(Value::Array(arr)) => {
            for item in arr {
                let c = item.get("code").and_then(|v| v.as_str()).unwrap_or("");
                if c == c_trim {
                    return item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(c_trim)
                        .to_string();
                }
            }
        }
        _ => {}
    }
    c_trim.to_string()
}

/// Draw a label + value with dotted line, automatically wrapping long values across multiple lines.
/// Returns the new y baseline below the drawn lines.
fn draw_label_val_wrapped(
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
    let vx = x + lbl_w + 2.0;
    let val_trim = val.trim();
    let f = if bold_val { &fonts.bold } else { &fonts.regular };

    if vx >= max_x - 18.0 {
        // Label takes almost full width -> put value on next line
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

    // First line beside label
    draw_dotted_line(layer, vx, y - 0.9, max_x);
    draw_text(layer, f, size_pt, vx + 0.8, y, &lines[0]);
    y -= LINE_H;

    // Continuation lines if wrapped
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

struct PdfPageCursor<'a> {
    doc: &'a PdfDocumentReference,
    fonts: &'a PdfFonts,
    layer: PdfLayerReference,
    page_no: usize,
    y: f32,
    ma_tn: &'a str,
}

impl<'a> PdfPageCursor<'a> {
    fn new(
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

    fn next_page(&mut self) {
        self.page_no += 1;
        let (p, l) = self.doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
        self.layer = self.doc.get_page(p).get_layer(l);
        draw_barcode_header(&self.layer, self.fonts, self.ma_tn);
        draw_footer(&self.layer, self.fonts, self.page_no);
        self.y = TOP_Y;
    }

    fn ensure_space(&mut self, needed_mm: f32) {
        if self.y - needed_mm < BOTTOM_Y {
            self.next_page();
        }
    }

    fn draw_national_header(
        &mut self,
        cfg: &AppConfig,
        so_line: &str,
        title1: &str,
        title2: &str,
    ) {
        let left_cx = MARGIN_L + 35.0;
        let right_cx = MARGIN_L + 128.0;
        draw_text_centered(&self.layer, &self.fonts.regular, FONT_BODY, left_cx, self.y, "BỘ Y TẾ");
        draw_text_centered(
            &self.layer,
            &self.fonts.bold,
            FONT_BODY,
            right_cx,
            self.y,
            "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM",
        );
        self.y -= 5.6;

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
        self.y -= 4.8;

        draw_text_centered(&self.layer, &self.fonts.regular, FONT_BODY, left_cx, self.y, "—————————");
        draw_text_centered(&self.layer, &self.fonts.regular, FONT_BODY, right_cx, self.y, "—————————");
        self.y -= 5.4;

        draw_text_centered(&self.layer, &self.fonts.regular, FONT_BODY, left_cx, self.y, so_line);
        self.y -= 8.0;

        for line in wrap_lines(title1, CONTENT_W, FONT_TITLE) {
            draw_text_centered(&self.layer, &self.fonts.bold, FONT_TITLE, PAGE_W / 2.0, self.y, &line);
            self.y -= 6.0;
        }
        if !title2.trim().is_empty() {
            for line in wrap_lines(title2, CONTENT_W, FONT_TITLE) {
                draw_text_centered(&self.layer, &self.fonts.bold, FONT_TITLE, PAGE_W / 2.0, self.y, &line);
                self.y -= 6.0;
            }
        }
        self.y -= 2.5;
    }

    fn draw_section_heading(&mut self, heading: &str) {
        self.ensure_space(28.0);
        if self.y < TOP_Y - 1.0 {
            self.y -= 6.0;
        }
        for line in wrap_lines(heading, CONTENT_W, FONT_SUBTITLE) {
            draw_text(&self.layer, &self.fonts.bold, FONT_SUBTITLE, MARGIN_L, self.y, &line);
            self.y -= 5.8;
        }
    }

    fn draw_field(&mut self, label: &str, val: &str, bold_val: bool) {
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

    fn draw_two_fields(&mut self, l_lbl: &str, l_val: &str, r_lbl: &str, r_val: &str) {
        let max_x = PAGE_W - MARGIN_R;
        let split_x = MARGIN_L + CONTENT_W * 0.50;
        let l_need = approx_text_w_mm(l_lbl, FONT_BODY) + approx_text_w_mm(l_val, FONT_BODY) + 18.0;
        let r_need = approx_text_w_mm(r_lbl, FONT_BODY) + approx_text_w_mm(r_val, FONT_BODY) + 18.0;
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

    fn draw_gender_dob_age(&mut self, gt: &str, ns: &str, tuoi: &str) {
        self.ensure_space(LINE_H * 2.0);
        draw_text(&self.layer, &self.fonts.bold, FONT_BODY, MARGIN_L, self.y, "Giới tính:");
        draw_checkbox(&self.layer, MARGIN_L + 22.0, self.y, gt == "1");
        draw_text(&self.layer, &self.fonts.regular, FONT_BODY, MARGIN_L + 27.0, self.y, "Nam");
        draw_checkbox(&self.layer, MARGIN_L + 42.0, self.y, gt == "2");
        draw_text(&self.layer, &self.fonts.regular, FONT_BODY, MARGIN_L + 47.0, self.y, "Nữ");

        let max_x = PAGE_W - MARGIN_R;
        let y_dob = draw_label_val_wrapped(
            &self.layer,
            self.fonts,
            MARGIN_L + 60.0,
            self.y,
            MARGIN_L + 136.0,
            "Ngày sinh:",
            ns,
            FONT_BODY,
            false,
        );
        let y_age = draw_label_val_wrapped(
            &self.layer,
            self.fonts,
            MARGIN_L + 139.0,
            self.y,
            max_x,
            "Tuổi:",
            tuoi,
            FONT_BODY,
            true,
        );
        self.y = y_dob.min(y_age);
    }

    fn draw_signature_block(
        &mut self,
        cfg: &AppConfig,
        ngay_vao: &str,
        left_title: Option<&str>,
        right_title: &str,
        right_name: &str,
    ) {
        let need_h = if right_name.trim().is_empty() { 20.0 } else { 30.0 };
        self.ensure_space(need_h);
        self.y -= 2.5;
        let parts: Vec<&str> = ngay_vao.split('/').collect();
        let (d_s, m_s, y_s) = if parts.len() == 3 {
            (parts[0], parts[1], parts[2])
        } else {
            ("....", "....", "2026")
        };
        let dia_danh = if cfg.dia_danh.trim().is_empty() {
            "Gia Lai"
        } else {
            cfg.dia_danh.trim()
        };
        let date_line = format!("{}, ngày {} tháng {} năm {}", dia_danh, d_s, m_s, y_s);
        let sig_right_cx = MARGIN_L + 130.0;

        if let Some(lt) = left_title {
            let sig_left_cx = MARGIN_L + 42.0;
            draw_text_centered(&self.layer, &self.fonts.italic, FONT_BODY, sig_right_cx, self.y, &date_line);
            self.y -= 5.8;
            draw_text_centered(&self.layer, &self.fonts.bold, FONT_BODY, sig_left_cx, self.y, lt);
            draw_text_centered(&self.layer, &self.fonts.bold, FONT_BODY, sig_right_cx, self.y, right_title);
            self.y -= 5.5;
            draw_text_centered(
                &self.layer,
                &self.fonts.italic,
                FONT_BODY,
                sig_left_cx,
                self.y,
                "(Ký, ghi rõ họ tên)",
            );
            draw_text_centered(
                &self.layer,
                &self.fonts.italic,
                FONT_BODY,
                sig_right_cx,
                self.y,
                "(Ký, ghi rõ họ tên và đóng dấu)",
            );
            if !right_name.trim().is_empty() {
                draw_text_centered(
                    &self.layer,
                    &self.fonts.bold,
                    FONT_BODY,
                    sig_left_cx,
                    self.y - 16.0,
                    right_name,
                );
            }
            self.y -= 22.0;
        } else {
            draw_text_centered(&self.layer, &self.fonts.italic, FONT_BODY, sig_right_cx, self.y, &date_line);
            self.y -= 5.8;
            draw_text_centered(&self.layer, &self.fonts.bold, FONT_BODY, sig_right_cx, self.y, right_title);
            self.y -= 5.5;
            draw_text_centered(
                &self.layer,
                &self.fonts.italic,
                FONT_BODY,
                sig_right_cx,
                self.y,
                "(Ký, ghi rõ họ tên và đóng dấu)",
            );
            if !right_name.trim().is_empty() {
                draw_text_centered(
                    &self.layer,
                    &self.fonts.bold,
                    FONT_BODY,
                    sig_right_cx,
                    self.y - 16.0,
                    right_name,
                );
            }
            self.y -= 22.0;
        }
    }
}

fn calc_age(ns: &str) -> String {
    if ns.len() >= 4 {
        ns[ns.len() - 4..]
            .parse::<i32>()
            .ok()
            .map(|yr| (2026 - yr).max(0).to_string())
            .unwrap_or_default()
    } else {
        String::new()
    }
}

fn render_patient_pages(
    doc: &PdfDocumentReference,
    fonts: &PdfFonts,
    form: &Map<String, Value>,
    cat_map: &Map<String, Value>,
    cfg: &AppConfig,
    is_reception_print: bool,
    is_first_page_of_doc: bool,
    first_layer: Option<PdfLayerReference>,
) {
    let explicit_tpl = get_s(form, "_PRINT_TEMPLATE");
    let tpl_code = if !explicit_tpl.is_empty() {
        explicit_tpl.to_string()
    } else {
        crate::templates::parse_template_list(get_s(form, "EXAM_TEMPLATES"))
            .into_iter()
            .next()
            .unwrap_or_else(|| "TREN_18".to_string())
    };
    match tpl_code.as_str() {
        "TREN_18" => render_tren18_pages(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        "LAI_XE" => render_laixe_pages(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        _ => render_custom_template_pages(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
            &tpl_code,
        ),
    }
}

fn render_tren18_pages(
    doc: &PdfDocumentReference,
    fonts: &PdfFonts,
    form: &Map<String, Value>,
    cat_map: &Map<String, Value>,
    cfg: &AppConfig,
    is_reception_print: bool,
    is_first_page_of_doc: bool,
    first_layer: Option<PdfLayerReference>,
) {
    let ma_tn = get_s(form, "MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);
    let v = |keys: &[&str]| -> &str {
        if is_reception_print {
            ""
        } else {
            get_any(form, keys)
        }
    };
    let b = |key: &str| -> bool {
        if is_reception_print {
            false
        } else {
            get_b(form, key)
        }
    };

    let ngay_vao = get_s(form, "NGAY_VAO");
    let yy = if ngay_vao.len() >= 2 {
        &ngay_vao[ngay_vao.len() - 2..]
    } else {
        "26"
    };
    let so_phieu = get_s(form, "SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSK-{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        cfg.ma_cskcb,
        yy
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG CHO",
        "NGƯỜI TỪ ĐỦ 18 TUỔI TRỞ LÊN",
    );

    let ho_ten = get_s(form, "HO_TEN").to_uppercase();
    cur.draw_field("1. Họ và tên (viết chữ in hoa):", &ho_ten, true);

    let gt = get_s(form, "GIOI_TINH");
    let ns = get_s(form, "NGAY_SINH");
    let tuoi = calc_age(ns);
    cur.draw_gender_dob_age(gt, ns, &tuoi);

    cur.draw_field("4. Số CCCD/Hộ chiếu/Mã định danh:", get_s(form, "SO_CCCD"), false);
    cur.draw_two_fields(
        "5. Cấp ngày:",
        get_s(form, "NGAYCAP_CCCD"),
        "Nơi cấp:",
        get_s(form, "NOICAP_CCCD"),
    );

    let dantoc = get_cat_name(cat_map, "DAN_TOC", get_s(form, "MA_DAN_TOC"));
    let doituong = get_cat_name(cat_map, "DOI_TUONG", get_s(form, "DOI_TUONG"));
    cur.draw_two_fields("6. Dân tộc:", &dantoc, "7. Đối tượng:", &doituong);

    let nguon = get_cat_name(cat_map, "NGUON_CHI_TRA", get_s(form, "NGUON_CHI_TRA"));
    cur.draw_field("8. Nguồn chi trả:", &nguon, false);
    cur.draw_two_fields("9. Nhóm máu:", get_s(form, "NHOM_MAU"), "Điện thoại:", get_s(form, "DIEN_THOAI"));

    let tinh = get_cat_name(cat_map, "TINH_TP", get_s(form, "MATINH_CU_TRU"));
    let xa = get_cat_name(cat_map, "XA_PHUONG", get_s(form, "MAXA_CU_TRU"));
    cur.draw_two_fields("10. Nơi ở hiện tại - Tỉnh/TP:", &tinh, "Phường/xã:", &xa);
    cur.draw_field("    Số nhà/thôn/xóm:", get_s(form, "DIA_CHI"), false);

    let nghe = get_cat_name(cat_map, "NGHE_NGHIEP", get_s(form, "MA_NGHE_NGHIEP"));
    cur.draw_field("11. Nghề nghiệp:", &nghe, false);
    cur.draw_field("12. Nơi làm việc, học tập:", get_s(form, "NOI_LAM_VIEC_HOC_TAP"), false);
    let ly_do = if get_s(form, "LY_DO_VV").is_empty() {
        "Khám sức khỏe"
    } else {
        get_s(form, "LY_DO_VV")
    };
    cur.draw_field("13. Lý do khám sức khỏe:", ly_do, false);

    // Tiền sử bệnh
    cur.draw_section_heading("TIỀN SỬ BỆNH CỦA ĐỐI TƯỢNG KHÁM SỨC KHỎE");
    cur.ensure_space(16.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "1. Tiền sử gia đình:");
    cur.y -= LINE_H;
    let ts_gd_co = b("TSGD_MAC_BENH") || b("TS_GIA_DINH") || !v(&["TSGD_MA_BENH", "TS_GIA_DINH_CHI_TIET"]).is_empty();
    let ts_gd_detail = v(&["TSGD_MA_BENH", "TS_GIA_DINH_CHI_TIET"]);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 2.0, cur.y, "a)");
    draw_checkbox(&cur.layer, MARGIN_L + 9.0, cur.y, !is_reception_print && !ts_gd_co);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 14.0, cur.y, "Không");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 32.0, cur.y, "b)");
    draw_checkbox(&cur.layer, MARGIN_L + 38.0, cur.y, ts_gd_co);
    cur.y = draw_label_val_wrapped(
        &cur.layer,
        cur.fonts,
        MARGIN_L + 43.0,
        cur.y,
        PAGE_W - MARGIN_R,
        "Có (nếu có ghi rõ):",
        ts_gd_detail,
        FONT_BODY,
        false,
    );

    cur.ensure_space(14.0);
    draw_text(
        &cur.layer,
        &cur.fonts.bold,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "2. Tiền sử bản thân (Ông/bà đã/đang mắc bệnh nào sau đây không):",
    );
    cur.y -= 5.5;

    let ts_items: &[(&str, &str, &str)] = &[
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
    draw_ts_table_paginated(&mut cur, ts_items, &b, is_reception_print);
    cur.y -= 6.0;

    // Câu hỏi khác
    cur.ensure_space(28.0);
    cur.y -= 3.0;
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "3. Câu hỏi khác (nếu có):");
    cur.y -= LINE_H;
    let q3a = v(&["TSBT_TEN_THUOC_LIEU_LUONG", "TS_DANG_DIEU_TRI"]);
    let q3a_co = b("TSBT_DANG_DIEU_TRI_BENH") || !q3a.is_empty();
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 2.0, cur.y, "a) Đang điều trị bệnh gì không?");
    draw_checkbox(&cur.layer, MARGIN_L + 75.0, cur.y, !is_reception_print && !q3a_co);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 80.0, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 98.0, cur.y, q3a_co);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 103.0, cur.y, "Có");
    cur.y -= LINE_H;
    if q3a_co {
        cur.draw_field("   Chi tiết bệnh & thuốc:", q3a, false);
    }

    let q3b = v(&["TSBT_TEN_THUOC_THAI_SAN", "TS_THAI_SAN"]);
    let q3b_co = gt == "2" && (b("TSBT_THAI_SAN") || !q3b.is_empty());
    cur.ensure_space(14.0);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 2.0, cur.y, "b) Tiền sử thai sản (nữ):");
    draw_checkbox(&cur.layer, MARGIN_L + 75.0, cur.y, !is_reception_print && !q3b_co);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 80.0, cur.y, "Không");
    draw_checkbox(&cur.layer, MARGIN_L + 98.0, cur.y, q3b_co);
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + 103.0, cur.y, "Có");
    cur.y -= LINE_H;
    if q3b_co {
        cur.draw_field("   Chi tiết thai sản:", q3b, false);
    }

    cur.ensure_space(34.0);
    draw_text(
        &cur.layer,
        &cur.fonts.italic,
        FONT_BODY,
        MARGIN_L,
        cur.y,
        "Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật.",
    );
    cur.y -= 4.0;
    cur.draw_signature_block(cfg, ngay_vao, None, "NGƯỜI ĐỀ NGHỊ KHÁM SỨC KHỎE", "");

    // Sang trang mới cho Phần I. Khám thể lực & II. Khám lâm sàng
    cur.next_page();
    cur.draw_section_heading("I. KHÁM THỂ LỰC");
    let chieu_cao = v(&["CHIEU_CAO"]);
    let can_nang = v(&["CAN_NANG"]);
    let bmi = v(&["CHI_SO_BMI", "BMI"]);
    cur.draw_two_fields("• Chiều cao (cm):", chieu_cao, "• Cân nặng (kg):", can_nang);
    cur.draw_two_fields("• Chỉ số BMI:", bmi, "• Mạch (lần/phút):", v(&["MACH"]));
    let ha = if !v(&["HUYET_AP"]).is_empty() {
        v(&["HUYET_AP"]).to_string()
    } else if !v(&["HUYET_AP_TAM_THU"]).is_empty() {
        format!("{}/{}", v(&["HUYET_AP_TAM_THU"]), v(&["HUYET_AP_TAM_TRUONG"]))
    } else {
        String::new()
    };
    let pl_tl = v(&["KHAM_THE_LUC_PL", "PHAN_LOAI_THE_LUC"]);
    cur.draw_two_fields("• Huyết áp (mmHg):", &ha, "• Phân loại thể lực:", pl_tl);
    cur.draw_field("• Người đo thể lực:", v(&["BS_THE_LUC", "NGUOI_DO_THE_LUC"]), false);

    // II. KHÁM LÂM SÀNG
    cur.draw_section_heading("II. KHÁM LÂM SÀNG");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_two_col_table_header(&mut cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký Bác sỹ");

    let noi_items: &[(&str, &[&str], &[&str])] = &[
        ("1a) Nội - Tuần hoàn:", &["NOI_KHOA_TUAN_HOAN", "KHAM_NOI_TUAN_HOAN"], &["NOI_KHOA_TUAN_HOAN_PL", "PL_NOI_TUAN_HOAN"]),
        ("1b) Nội - Hô hấp:", &["NOI_KHOA_HO_HAP", "KHAM_NOI_HO_HAP"], &["NOI_KHOA_HO_HAP_PL", "PL_NOI_HO_HAP"]),
        ("1c) Nội - Tiêu hóa:", &["NOI_KHOA_TIEU_HOA", "KHAM_NOI_TIEU_HOA"], &["NOI_KHOA_TIEU_HOA_PL", "PL_NOI_TIEU_HOA"]),
        ("1d) Thận - Tiết niệu - SD:", &["NOI_KHOA_THAN_TN_SD", "KHAM_NOI_THAN_TN"], &["NOI_KHOA_THAN_TIETNIEU_PL", "PL_NOI_THAN_TN"]),
        ("1đ) Nội tiết:", &["NOI_KHOA_NOI_TIET", "KHAM_NOI_TIET"], &["NOI_KHOA_NOI_TIET_PL", "PL_NOI_TIET"]),
        ("1e) Cơ – xương – khớp:", &["NOI_KHOA_CO_XUONG_KHOP", "KHAM_NOI_CO_XUONG_KHOP"], &["NOI_KHOA_CO_XUONG_KHOP_PL", "PL_NOI_CO_XUONG_KHOP"]),
        ("1g) Thần kinh:", &["NOI_KHOA_THAN_KINH", "KHAM_NOI_THAN_KINH"], &["NOI_KHOA_THAN_KINH_PL", "PL_NOI_THAN_KINH"]),
        ("1h) Tâm thần:", &["NOI_KHOA_TAM_THAN", "KHAM_NOI_TAM_THAN"], &["NOI_KHOA_TAM_THAN_PL", "PL_NOI_TAM_THAN"]),
    ];
    let bs_noi = v(&["BS_NOI", "BS_NOI_KHOA"]);
    for (lbl, vk, pk) in noi_items {
        let val = v(vk);
        let pl = v(pk);
        let body = format!(
            "{} {}",
            val,
            if pl.is_empty() {
                String::new()
            } else {
                format!("(Phân loại: {})", pl)
            }
        )
        .trim()
        .to_string();
        draw_two_col_row(&mut cur, col1_w, col2_w, lbl, &body, bs_noi);
    }

    let other_specs: &[(&str, &[&str], &[&str], &[&str])] = &[
        ("2. Ngoại khoa:", &["KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA"], &["KHAM_NGOAI_KHOA_PL", "PL_NGOAI_KHOA"], &["BS_NGOAI", "BS_NGOAI_KHOA"]),
        ("3. Da liễu:", &["KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU"], &["KHAM_DA_LIEU_PL", "PL_DA_LIEU"], &["BS_DA_LIEU"]),
        ("4. Sản phụ khoa (nữ):", &["KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA"], &["KHAM_SAN_PHU_KHOA_PL", "PL_SAN_PHU_KHOA"], &["BS_SAN", "BS_SAN_PHU_KHOA"]),
    ];
    for (lbl, vk, pk, bk) in other_specs {
        let val = v(vk);
        let pl = v(pk);
        let body = format!(
            "{} {}",
            val,
            if pl.is_empty() {
                String::new()
            } else {
                format!("(Phân loại: {})", pl)
            }
        )
        .trim()
        .to_string();
        draw_two_col_row(&mut cur, col1_w, col2_w, lbl, &body, v(bk));
    }

    // 5. Mắt
    let mat_text = format!(
        "Không kính MP: {}, MT: {} | Có kính MP: {}, MT: {}. Bệnh mắt: {} (PL: {})",
        v(&["KHONG_KINH_MAT_PHAI", "MAT_KHONG_KINH_PHAI"]),
        v(&["KHONG_KINH_MAT_TRAI", "MAT_KHONG_KINH_TRAI"]),
        v(&["CO_KINH_MAT_PHAI", "MAT_CO_KINH_PHAI"]),
        v(&["CO_KINH_MAT_TRAI", "MAT_CO_KINH_TRAI"]),
        v(&["BENH_KHAC_MAT", "KHAM_MAT"]),
        v(&["KHAM_MAT_PL", "PL_MAT"])
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "5. Mắt:", &mat_text, v(&["BS_MAT"]));

    // 6. Tai - Mũi - Họng
    let tmh_text = format!(
        "Tai trái: thường {}m, thầm {}m | Tai phải: thường {}m, thầm {}m. Bệnh TMH: {} (PL: {})",
        v(&["TAI_TRAI_NOI_THUONG", "TMH_TAI_TRAI_THUONG"]),
        v(&["TAI_TRAI_NOI_THAM", "TMH_TAI_TRAI_THAM"]),
        v(&["TAI_PHAI_NOI_THUONG", "TMH_TAI_PHAI_THUONG"]),
        v(&["TAI_PHAI_NOI_THAM", "TMH_TAI_PHAI_THAM"]),
        v(&["BENH_KHAC_TAI_MUI_HONG", "KHAM_TMH"]),
        v(&["KHAM_TAI_MUI_HONG_PL", "PL_TMH"])
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "6. Tai - Mũi - Họng:", &tmh_text, v(&["BS_TMH"]));

    // 7. Răng - Hàm - Mặt
    let rhm_text = format!(
        "Hàm trên: {} | Hàm dưới: {}. Bệnh RHM: {} (PL: {})",
        v(&["HAM_TREN", "RHM_HAM_TREN"]),
        v(&["HAM_DUOI", "RHM_HAM_DUOI"]),
        v(&["BENH_KHAC_RANG_HAM_MAT", "KHAM_RHM"]),
        v(&["KHAM_RANG_HAM_MAT_PL", "PL_RHM"])
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "7. Răng - Hàm - Mặt:", &rhm_text, v(&["BS_RHM"]));

    // III. KHÁM CẬN LÂM SÀNG
    cur.draw_section_heading("III. KHÁM CẬN LÂM SÀNG");
    draw_two_col_table_header(&mut cur, col1_w, col2_w, "Nội dung xét nghiệm / CĐHA", "Họ tên, chữ ký Bác sỹ");
    let bs_cls = v(&["BS_CLS"]);
    let cls_mau = format!(
        "Huyết học: {} | Đường máu: {} | Urê: {} | Creatinin: {} | GOT: {} | GPT: {}",
        v(&["CLS_HUYET_HOC", "XN_HUYET_HOC"]),
        v(&["CLS_DUONG_MAU", "XN_DUONG_MAU"]),
        v(&["CLS_URE", "XN_URE"]),
        v(&["CLS_CREATININ", "XN_CREATININ"]),
        v(&["CLS_ASAT_GOT", "XN_GOT"]),
        v(&["CLS_ALAT_GPT", "XN_GPT"]),
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "a) Xét nghiệm máu:", &cls_mau, bs_cls);
    draw_two_col_row(
        &mut cur,
        col1_w,
        col2_w,
        "b) Xét nghiệm nước tiểu:",
        v(&["CLS_NUOC_TIEU", "XN_NUOC_TIEU"]),
        bs_cls,
    );
    draw_two_col_row(
        &mut cur,
        col1_w,
        col2_w,
        "c) X-Quang tim phổi:",
        v(&["CLS_XQ_TIM_PHOI", "CDHA_XQ"]),
        bs_cls,
    );
    draw_two_col_row(
        &mut cur,
        col1_w,
        col2_w,
        "d) CLS khác & Kết luận CLS:",
        &format!(
            "{} {}",
            v(&["CLS_KHAC_MO_TA", "CLS_KHAC"]),
            v(&["CLS_DINH_KY_KET_QUA", "CLS_THEO_CHI_DINH"])
        )
        .trim(),
        bs_cls,
    );

    // IV. KẾT LUẬN
    cur.draw_section_heading("IV. KẾT LUẬN");
    let kl_benh = format!(
        "{} {}",
        v(&["KET_LUAN_BENH_TAT", "KET_LUAN_BENH"]),
        v(&["CAC_BENH_TAT_NEU_CO"])
    )
    .trim()
    .to_string();
    cur.draw_field("1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có):", &kl_benh, false);
    let pl_sk = v(&["PHAN_LOAI_SK", "PHAN_LOAI_SUC_KHOE"]);
    let pl_disp = if pl_sk.is_empty() {
        String::new()
    } else {
        format!("Loại {}", pl_sk)
    };
    cur.draw_field("2. Phân loại sức khỏe (theo quy định của Bộ Y tế):", &pl_disp, true);
    cur.draw_signature_block(
        cfg,
        ngay_vao,
        None,
        "NGƯỜI KẾT LUẬN",
        v(&["BS_KET_LUAN", "NGUOI_KET_LUAN"]),
    );
}

fn render_laixe_pages(
    doc: &PdfDocumentReference,
    fonts: &PdfFonts,
    form: &Map<String, Value>,
    cat_map: &Map<String, Value>,
    cfg: &AppConfig,
    is_reception_print: bool,
    is_first_page_of_doc: bool,
    first_layer: Option<PdfLayerReference>,
) {
    let ma_tn = get_s(form, "MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);
    let v_ns = |raw_k: &str| -> &str {
        if is_reception_print {
            ""
        } else {
            let k = crate::templates::namespaced_key("LAI_XE", raw_k);
            form.get(&k).and_then(|val| val.as_str()).unwrap_or("").trim()
        }
    };
    let b_ns = |raw_k: &str| -> bool {
        if is_reception_print {
            false
        } else {
            let k = crate::templates::namespaced_key("LAI_XE", raw_k);
            let s = form.get(&k).and_then(|val| val.as_str()).unwrap_or("").trim();
            s == "Có" || s == "1" || s.eq_ignore_ascii_case("true")
        }
    };

    let ngay_vao = get_s(form, "NGAY_VAO");
    let yy = if ngay_vao.len() >= 2 {
        &ngay_vao[ngay_vao.len() - 2..]
    } else {
        "26"
    };
    let so_phieu = get_s(form, "SO_PHIEU");
    let so_line = format!(
        "Số: {}/GKSKLX/{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        cfg.ma_cskcb,
        yy
    );

    cur.draw_national_header(
        cfg,
        &so_line,
        "MẪU GIẤY KHÁM SỨC KHỎE",
        "CỦA NGƯỜI LÁI XE, NGƯỜI ĐIỀU KHIỂN XE MÁY CHUYÊN DÙNG",
    );

    // Khung ảnh 4x6 bên trái + Thông tin hành chính 1..7 bên phải
    cur.ensure_space(54.0);
    let photo_w = 34.0;
    let photo_h = 44.0;
    let photo_top = cur.y;
    draw_rect_stroke(&cur.layer, MARGIN_L, photo_top - photo_h, photo_w, photo_h, 0.6);
    draw_text_centered(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L + photo_w / 2.0, photo_top - 16.0, "Ảnh");
    draw_text_centered(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + photo_w / 2.0, photo_top - 22.5, "(4 x 6 cm)");

    let rx = MARGIN_L + photo_w + 5.0;
    let max_x = PAGE_W - MARGIN_R;
    let mut ry = photo_top;
    ry = draw_label_val_wrapped(&cur.layer, cur.fonts, rx, ry, max_x, "1. Họ và tên:", &get_s(form, "HO_TEN").to_uppercase(), FONT_BODY, true);
    let gt = get_s(form, "GIOI_TINH");
    let ns = get_s(form, "NGAY_SINH");
    let tuoi = calc_age(ns);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, rx, ry, "2. Giới:");
    draw_checkbox(&cur.layer, rx + 17.0, ry, gt == "1");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, rx + 22.0, ry, "Nam");
    draw_checkbox(&cur.layer, rx + 35.0, ry, gt == "2");
    draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, rx + 40.0, ry, "Nữ");
    ry = draw_label_val_wrapped(&cur.layer, cur.fonts, rx + 52.0, ry, max_x, "3. Sinh:", &format!("{} (Tuổi: {})", ns, tuoi), FONT_BODY, false);
    ry = draw_label_val_wrapped(&cur.layer, cur.fonts, rx, ry, max_x, "4. Số CCCD/Định danh:", get_s(form, "SO_CCCD"), FONT_BODY, false);
    ry = draw_label_val_wrapped(&cur.layer, cur.fonts, rx, ry, max_x, "5. Cấp ngày / Tại:", &format!("{} - {}", get_s(form, "NGAYCAP_CCCD"), get_s(form, "NOICAP_CCCD")), FONT_BODY, false);
    let tinh = get_cat_name(cat_map, "TINH_TP", get_s(form, "MATINH_CU_TRU"));
    let xa = get_cat_name(cat_map, "XA_PHUONG", get_s(form, "MAXA_CU_TRU"));
    let dia_chi = [get_s(form, "DIA_CHI"), xa.as_str(), tinh.as_str()]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    ry = draw_label_val_wrapped(&cur.layer, cur.fonts, rx, ry, max_x, "6. Chỗ ở hiện tại:", &dia_chi, FONT_BODY, false);
    let hang_lx = get_any(form, &["HANG_BANG_LAI", "LX_HANG_BANG_LAI_KL"]);
    ry = draw_label_val_wrapped(&cur.layer, cur.fonts, rx, ry, max_x, "7. Đề nghị khám lái xe hạng:", hang_lx, FONT_BODY, true);

    cur.y = ry.min(photo_top - photo_h - 4.0);

    // I. TIỀN SỬ BỆNH
    cur.draw_section_heading("I. TIỀN SỬ BỆNH CỦA ĐỐI TƯỢNG KHÁM SỨC KHỎE");
    let tsgd_co = b_ns("TSGD_MAC_BENH");
    cur.draw_field(
        "1. Tiền sử gia đình (Có/Không & tên bệnh):",
        &format!(
            "{} {}",
            if tsgd_co { "Có" } else { "Không" },
            v_ns("TSGD_TEN_BENH")
        )
        .trim(),
        false,
    );

    cur.ensure_space(12.0);
    draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L, cur.y, "2. Tiền sử, bệnh sử bản thân (đánh dấu X vào ô tương ứng):");
    cur.y -= 5.5;

    let lx_items: &[(&str, &str, &str)] = &[
        ("1", "Có bệnh hay bị thương trong 5 năm qua", "TSBT_1"),
        ("2", "Có bệnh thần kinh hay bị thương ở đầu", "TSBT_2"),
        ("3", "Bệnh mắt hoặc giảm thị lực (trừ đeo kính thuốc)", "TSBT_3"),
        ("4", "Bệnh ở tai, giảm sức nghe hoặc thăng bằng", "TSBT_4"),
        ("5", "Bệnh ở tim, nhồi máu cơ tim, bệnh tim mạch khác", "TSBT_5"),
        ("6", "Phẫu thuật can thiệp tim - mạch (van, cầu nối, stent...)", "TSBT_6"),
        ("7", "Tăng huyết áp", "TSBT_7"),
        ("8", "Khó thở", "TSBT_8"),
        ("9", "Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn", "TSBT_9"),
        ("10", "Bệnh thận, lọc máu", "TSBT_10"),
        ("11", "Đái tháo đường hoặc kiểm soát tăng đường huyết", "TSBT_11"),
        ("12", "Bệnh tâm thần", "TSBT_12"),
        ("13", "Mất ý thức, rối loạn ý thức", "TSBT_13"),
        ("14", "Ngất, chóng mặt, mệt mỏi, buồn nôn hoặc ngất xỉu", "TSBT_14"),
        ("15", "Bệnh tiêu hóa", "TSBT_15"),
        ("16", "Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ, ngáy to", "TSBT_16"),
        ("17", "Tai biến mạch máu não hoặc liệt", "TSBT_17"),
        ("18", "Bệnh hoặc tổn thương cột sống", "TSBT_18"),
        ("19", "Sử dụng rượu thường xuyên, liên tục", "TSBT_19"),
        ("20", "Sử dụng ma túy và chất gây nghiện", "TSBT_20"),
    ];
    let b_lx = |k: &str| -> bool { b_ns(k) };
    draw_ts_table_paginated(&mut cur, lx_items, &b_lx, is_reception_print);
    cur.y -= 6.0;

    cur.draw_field("Nếu “có”, ghi cụ thể tên bệnh:", v_ns("TSBT_TEN_BENH"), false);
    cur.draw_field("3. Đang điều trị bệnh gì / thuốc & liều lượng:", v_ns("TSBT_DANG_DIEU_TRI"), false);
    cur.ensure_space(34.0);
    draw_text(&cur.layer, &cur.fonts.italic, FONT_BODY, MARGIN_L, cur.y, "Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật.");
    cur.y -= 4.0;
    cur.draw_signature_block(cfg, ngay_vao, None, "NGƯỜI ĐỀ NGHỊ KHÁM SỨC KHỎE", "");

    // II. KHÁM LÂM SÀNG
    cur.next_page();
    cur.draw_section_heading("II. KHÁM LÂM SÀNG (LÁI XE)");
    let col1_w = CONTENT_W * 0.72;
    let col2_w = CONTENT_W - col1_w;
    draw_two_col_table_header(&mut cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký Bác sỹ");

    draw_two_col_row(&mut cur, col1_w, col2_w, "1. Tâm thần:", &format!("{} (KL: {})", v_ns("KLS_TAM_THAN"), v_ns("KL_TAM_THAN")), v_ns("BS_TAM_THAN"));
    draw_two_col_row(&mut cur, col1_w, col2_w, "2. Thần kinh:", &format!("{} (KL: {})", v_ns("KLS_THAN_KINH"), v_ns("KL_THAN_KINH")), v_ns("BS_THAN_KINH"));
    let mat_lx = format!(
        "Không kính MP: {}, MT: {} | Có kính MP: {}, MT: {} | Hai mắt KK: {}, CK: {} | Thị trường ngang: {}, đứng: {} | Sắc giác: {} | Bệnh mắt: {} | KL: {}",
        v_ns("MAT_KK_MP"), v_ns("MAT_KK_MT"), v_ns("MAT_CK_MP"), v_ns("MAT_CK_MT"),
        v_ns("MAT_2M_KK"), v_ns("MAT_2M_CK"), v_ns("MAT_TT_NGANG"), v_ns("MAT_TT_DUNG"),
        v_ns("MAT_SAC_GIAC"), v_ns("MAT_BENH"), v_ns("KL_MAT")
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "3. Mắt:", &mat_lx, v_ns("BS_MAT"));
    let tmh_lx = format!(
        "Tai trái: thường {}m, thầm {}m | Tai phải: thường {}m, thầm {}m | Bệnh TMH: {} | KL: {}",
        v_ns("TMH_TT_THUONG"), v_ns("TMH_TT_THAM"), v_ns("TMH_TP_THUONG"), v_ns("TMH_TP_THAM"),
        v_ns("TMH_BENH"), v_ns("KL_TMH")
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "4. Tai - Mũi - Họng:", &tmh_lx, v_ns("BS_TMH"));
    let tm_lx = format!(
        "Mạch: {} l/ph; HA: {} mmHg | {} | KL: {}",
        v_ns("MACH"), v_ns("HUYET_AP"), v_ns("KLS_TIM_MACH"), v_ns("KL_TIM_MACH")
    );
    draw_two_col_row(&mut cur, col1_w, col2_w, "5. Tim mạch:", &tm_lx, v_ns("BS_TIM_MACH"));
    draw_two_col_row(&mut cur, col1_w, col2_w, "6. Hô hấp:", &format!("{} (KL: {})", v_ns("KLS_HO_HAP"), v_ns("KL_HO_HAP")), v_ns("BS_HO_HAP"));
    draw_two_col_row(&mut cur, col1_w, col2_w, "7. Cơ Xương Khớp:", &format!("{} (KL: {})", v_ns("KLS_CXK"), v_ns("KL_CXK")), v_ns("BS_CXK"));
    draw_two_col_row(&mut cur, col1_w, col2_w, "8. Nội tiết:", &format!("{} (KL: {})", v_ns("KLS_NOI_TIET"), v_ns("KL_NOI_TIET")), v_ns("BS_NOI_TIET"));

    // III. KHÁM CẬN LÂM SÀNG
    cur.draw_section_heading("III. KHÁM CẬN LÂM SÀNG");
    draw_two_col_table_header(&mut cur, col1_w, col2_w, "Nội dung khám", "Họ tên, chữ ký BS");
    draw_two_col_row(
        &mut cur,
        col1_w,
        col2_w,
        "1. Xét nghiệm ma túy (Amphetamin, Marijuana, Morphin, Codein, Heroin):",
        v_ns("XN_MA_TUY"),
        v_ns("BS_XN_MA_TUY"),
    );
    draw_two_col_row(
        &mut cur,
        col1_w,
        col2_w,
        "2. XN nồng độ cồn / huyết học / sinh hóa / X-quang:",
        &format!("KQ: {} | KL: {}", v_ns("XN_KHAC_KQ"), v_ns("XN_KHAC_KL")),
        v_ns("BS_XN_KHAC"),
    );

    // IV. KẾT LUẬN
    cur.draw_section_heading("IV. KẾT LUẬN");
    let kl_lx = format!("{} {}", v_ns("KET_LUAN_SK"), hang_lx).trim().to_string();
    cur.draw_field("Kết luận điều kiện sức khỏe:", &kl_lx, true);
    cur.draw_field("Tình trạng bệnh tật (nếu có):", v_ns("KET_LUAN_BENH"), false);
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
    cur.draw_signature_block(cfg, ngay_vao, None, "NGƯỜI KẾT LUẬN", v_ns("BS_KET_LUAN"));
}

fn draw_two_col_table_header(
    cur: &mut PdfPageCursor,
    col1_w: f32,
    col2_w: f32,
    h1: &str,
    h2: &str,
) {
    let hh = 7.5;
    cur.ensure_space(hh + 10.0);
    draw_rect_stroke(&cur.layer, MARGIN_L, cur.y - hh, col1_w, hh, 0.55);
    draw_rect_stroke(&cur.layer, MARGIN_L + col1_w, cur.y - hh, col2_w, hh, 0.55);
    draw_text_centered(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L + col1_w / 2.0, cur.y - 5.2, h1);
    draw_text_centered(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L + col1_w + col2_w / 2.0, cur.y - 5.2, h2);
    cur.y -= hh;
}

fn draw_two_col_row(
    cur: &mut PdfPageCursor,
    col1_w: f32,
    col2_w: f32,
    title: &str,
    body: &str,
    doctor: &str,
) {
    let combined = if body.trim().is_empty() {
        format!("{} ..............................", title)
    } else {
        format!("{} {}", title, body.trim())
    };
    let lines = wrap_lines(&combined, col1_w - 4.0, FONT_BODY);
    let doc_lines = wrap_lines(doctor, col2_w - 4.0, FONT_BODY);
    let max_lines = lines.len().max(doc_lines.len()).max(1);
    let rh = (max_lines as f32) * LINE_H + 3.0;

    if cur.y - rh < BOTTOM_Y {
        cur.next_page();
        draw_two_col_table_header(cur, col1_w, col2_w, "Nội dung khám (tiếp theo)", "Họ tên, chữ ký Bác sỹ");
    }

    draw_rect_stroke(&cur.layer, MARGIN_L, cur.y - rh, col1_w, rh, 0.5);
    draw_rect_stroke(&cur.layer, MARGIN_L + col1_w, cur.y - rh, col2_w, rh, 0.5);

    for (i, line) in lines.iter().enumerate() {
        let f = if i == 0 && body.trim().is_empty() {
            &cur.fonts.bold
        } else {
            &cur.fonts.regular
        };
        draw_text(&cur.layer, f, FONT_BODY, MARGIN_L + 2.0, cur.y - 5.0 - (i as f32) * LINE_H, line);
    }
    for (i, line) in doc_lines.iter().enumerate() {
        draw_text_centered(
            &cur.layer,
            &cur.fonts.bold,
            FONT_BODY,
            MARGIN_L + col1_w + col2_w / 2.0,
            cur.y - 5.0 - (i as f32) * LINE_H,
            line,
        );
    }
    cur.y -= rh;
}

fn draw_ts_table_paginated<F>(
    cur: &mut PdfPageCursor,
    items: &[(&str, &str, &str)],
    get_bool: &F,
    is_reception: bool,
) where
    F: Fn(&str) -> bool,
{
    let w_tt = CONTENT_W * 0.07;
    let w_name = CONTENT_W * 0.69;
    let w_co = CONTENT_W * 0.11;
    let w_khong = CONTENT_W - w_tt - w_name - w_co;

    let draw_hdr = |c: &mut PdfPageCursor| {
        let hh = 7.5;
        draw_rect_stroke(&c.layer, MARGIN_L, c.y - hh, w_tt, hh, 0.55);
        draw_rect_stroke(&c.layer, MARGIN_L + w_tt, c.y - hh, w_name, hh, 0.55);
        draw_rect_stroke(&c.layer, MARGIN_L + w_tt + w_name, c.y - hh, w_co, hh, 0.55);
        draw_rect_stroke(&c.layer, MARGIN_L + w_tt + w_name + w_co, c.y - hh, w_khong, hh, 0.55);
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, MARGIN_L + w_tt / 2.0, c.y - 5.2, "TT");
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, MARGIN_L + w_tt + w_name / 2.0, c.y - 5.2, "Tên bệnh, tật");
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, MARGIN_L + w_tt + w_name + w_co / 2.0, c.y - 5.2, "Có");
        draw_text_centered(&c.layer, &c.fonts.bold, FONT_BODY, MARGIN_L + w_tt + w_name + w_co + w_khong / 2.0, c.y - 5.2, "Không");
        c.y -= hh;
    };

    cur.ensure_space(16.0);
    draw_hdr(cur);

    for &(tt, label, key) in items {
        let lines = wrap_lines(label, w_name - 4.0, FONT_BODY);
        let rh = (lines.len().max(1) as f32) * LINE_H + 2.4;
        if cur.y - rh < BOTTOM_Y {
            cur.next_page();
            draw_hdr(cur);
        }

        draw_rect_stroke(&cur.layer, MARGIN_L, cur.y - rh, w_tt, rh, 0.5);
        draw_rect_stroke(&cur.layer, MARGIN_L + w_tt, cur.y - rh, w_name, rh, 0.5);
        draw_rect_stroke(&cur.layer, MARGIN_L + w_tt + w_name, cur.y - rh, w_co, rh, 0.5);
        draw_rect_stroke(&cur.layer, MARGIN_L + w_tt + w_name + w_co, cur.y - rh, w_khong, rh, 0.5);

        let mid_y = cur.y - rh / 2.0 - 1.5;
        draw_text_centered(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + w_tt / 2.0, mid_y, tt);
        for (li, line) in lines.iter().enumerate() {
            draw_text(&cur.layer, &cur.fonts.regular, FONT_BODY, MARGIN_L + w_tt + 2.0, cur.y - 5.0 - (li as f32) * LINE_H, line);
        }

        let checked_co = !is_reception && get_bool(key);
        let checked_khong = false;
        draw_checkbox(&cur.layer, MARGIN_L + w_tt + w_name + w_co / 2.0 - 1.7, mid_y, checked_co);
        draw_checkbox(&cur.layer, MARGIN_L + w_tt + w_name + w_co + w_khong / 2.0 - 1.7, mid_y, checked_khong);
        cur.y -= rh;
    }
}

fn render_custom_template_pages(
    doc: &PdfDocumentReference,
    fonts: &PdfFonts,
    form: &Map<String, Value>,
    cat_map: &Map<String, Value>,
    cfg: &AppConfig,
    is_reception_print: bool,
    is_first_page_of_doc: bool,
    first_layer: Option<PdfLayerReference>,
    tpl_code: &str,
) {
    let ma_tn = get_s(form, "MA_TIEP_NHAN");
    let mut cur = PdfPageCursor::new(doc, fonts, is_first_page_of_doc, first_layer, ma_tn);

    let ngay_vao = get_s(form, "NGAY_VAO");
    let yy = if ngay_vao.len() >= 2 {
        &ngay_vao[ngay_vao.len() - 2..]
    } else {
        "26"
    };
    let so_phieu = get_s(form, "SO_PHIEU");
    let code_tag = match tpl_code {
        "MAU_GIAO" => "GKSK-MN",
        "HS_PHO_THONG" => "GKSK-HS",
        _ => "GKSK",
    };
    let so_line = format!(
        "Số: {}/{}-{}/{}",
        if so_phieu.is_empty() { "......" } else { so_phieu },
        code_tag,
        cfg.ma_cskcb,
        yy
    );

    let (title1, title2) = match tpl_code {
        "DUOI_6" => (
            "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG CHO",
            "TRẺ EM DƯỚI 06 TUỔI",
        ),
        "TU_6_DUOI_18" => (
            "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG CHO",
            "NGƯỜI TỪ ĐỦ 06 TUỔI ĐẾN DƯỚI 18 TUỔI",
        ),
        "MAU_GIAO" => (
            "KIỂM TRA SỨC KHỎE ĐẦU NĂM HỌC CHO TRẺ EM",
            "TRONG CÁC CƠ SỞ GIÁO DỤC MẦM NON",
        ),
        "HS_PHO_THONG" => (
            "KIỂM TRA SỨC KHỎE ĐẦU NĂM HỌC CHO HỌC SINH",
            "TRONG CÁC CƠ SỞ GIÁO DỤC PHỔ THÔNG",
        ),
        _ => (
            "GIẤY KHÁM SỨC KHỎE",
            crate::templates::template_full_label(tpl_code),
        ),
    };

    cur.draw_national_header(cfg, &so_line, title1, title2);

    cur.draw_section_heading("I. THÔNG TIN HÀNH CHÍNH");
    cur.draw_field("1. Họ và tên (viết chữ in hoa):", &get_s(form, "HO_TEN").to_uppercase(), true);

    let gt = get_s(form, "GIOI_TINH");
    let ns = get_s(form, "NGAY_SINH");
    let tuoi = calc_age(ns);
    cur.draw_gender_dob_age(gt, ns, &tuoi);

    cur.draw_two_fields(
        "Số CCCD/Định danh:",
        get_s(form, "SO_CCCD"),
        "Mã BN / Số TN:",
        &format!("{} / {}", get_s(form, "MA_Y_TE"), ma_tn),
    );

    if matches!(tpl_code, "TU_6_DUOI_18" | "HS_PHO_THONG") {
        cur.draw_two_fields(
            "Cấp ngày:",
            get_s(form, "NGAYCAP_CCCD"),
            "Nơi cấp:",
            get_s(form, "NOICAP_CCCD"),
        );
    }

    if matches!(tpl_code, "DUOI_6" | "MAU_GIAO") {
        let sn = get_s(form, "SINH_NON");
        cur.draw_two_fields(
            "Sinh non:",
            if sn.is_empty() { "Không" } else { sn },
            "Tuần thai khi sinh:",
            get_s(form, "TUAN_THAI_KHI_SINH"),
        );
    }

    let dantoc = get_cat_name(cat_map, "DAN_TOC", get_s(form, "MA_DAN_TOC"));
    let nguon = get_cat_name(cat_map, "NGUON_CHI_TRA", get_s(form, "NGUON_CHI_TRA"));
    cur.draw_two_fields("Dân tộc:", &dantoc, "Nguồn chi trả:", &nguon);

    let tinh = get_cat_name(cat_map, "TINH_TP", get_s(form, "MATINH_CU_TRU"));
    let xa = get_cat_name(cat_map, "XA_PHUONG", get_s(form, "MAXA_CU_TRU"));
    let dia_chi_full = [get_s(form, "DIA_CHI"), xa.as_str(), tinh.as_str()]
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    cur.draw_field("Nơi ở hiện tại:", &dia_chi_full, false);

    if matches!(tpl_code, "MAU_GIAO" | "HS_PHO_THONG") {
        cur.draw_field("Cơ sở giáo dục đang theo học:", get_s(form, "TEN_CSGD"), false);
    }

    if matches!(tpl_code, "DUOI_6" | "MAU_GIAO" | "TU_6_DUOI_18") {
        cur.draw_two_fields(
            "Người giám hộ:",
            get_s(form, "HO_TEN_NGUOI_GIAM_HO"),
            "Quan hệ / ĐT:",
            &format!(
                "{} {}",
                get_s(form, "MOI_QUAN_HE_VOI_TRE"),
                get_s(form, "DIEN_THOAI_GIAM_HO")
            )
            .trim(),
        );
    }

    let tabs_order = [
        ("history", "II. TIỀN SỬ BỆNH TẬT & TIÊM CHỦNG"),
        ("physical", "III. DẤU HIỆU SINH TỒN & THỂ LỰC / DINH DƯỠNG"),
        ("clinical", "IV. KHÁM LÂM SÀNG & CẬN LÂM SÀNG"),
        ("conclusion", "V. KẾT LUẬN & TƯ VẤN SỨC KHỎE"),
    ];

    for (tab_key, sec_heading) in tabs_order {
        let groups = crate::templates::get_template_groups(tpl_code, tab_key);
        if groups.is_empty() {
            continue;
        }
        if tab_key == "conclusion" {
            cur.ensure_space(68.0);
        }
        cur.draw_section_heading(sec_heading);

        let hide_val = is_reception_print && tab_key != "history";
        for grp in groups {
            cur.ensure_space(14.0);
            for line in wrap_lines(grp.title, CONTENT_W, FONT_BODY) {
                draw_text(&cur.layer, &cur.fonts.bold, FONT_BODY, MARGIN_L + 1.0, cur.y, &line);
                cur.y -= LINE_H;
            }

            let mut i = 0usize;
            while i < grp.fields.len() {
                let f1 = &grp.fields[i];
                let k1 = crate::templates::namespaced_key(tpl_code, f1.key);
                let v1 = if hide_val { "" } else { get_s(form, &k1) };
                let l1 = format!("{}:", f1.label);

                if i + 1 < grp.fields.len() {
                    let f2 = &grp.fields[i + 1];
                    let k2 = crate::templates::namespaced_key(tpl_code, f2.key);
                    let v2 = if hide_val { "" } else { get_s(form, &k2) };
                    let l2 = format!("{}:", f2.label);
                    let need1 = approx_text_w_mm(&l1, FONT_BODY) + approx_text_w_mm(v1, FONT_BODY) + 18.0;
                    let need2 = approx_text_w_mm(&l2, FONT_BODY) + approx_text_w_mm(v2, FONT_BODY) + 18.0;
                    let half_w = CONTENT_W * 0.50 - 3.0;
                    if need1 <= half_w && need2 <= half_w {
                        cur.draw_two_fields(&l1, v1, &l2, v2);
                        i += 2;
                        continue;
                    }
                }
                cur.draw_field(&format!("- {}", l1), v1, false);
                i += 1;
            }
            cur.y -= 1.0;
        }
    }

    let has_two_sigs = matches!(tpl_code, "MAU_GIAO" | "HS_PHO_THONG");
    cur.draw_signature_block(
        cfg,
        ngay_vao,
        if has_two_sigs { Some("NGƯỜI KẾT LUẬN") } else { None },
        if has_two_sigs { "GIÁM ĐỐC CHUYÊN MÔN" } else { "NGƯỜI KẾT LUẬN" },
        "",
    );
}

/// Generates a multi-page A4 PDF directly in Rust.
pub fn generate_records_pdf(
    records: &[Map<String, Value>],
    cfg: &AppConfig,
    cat_map: &Map<String, Value>,
    is_reception_print: bool,
    title: &str,
) -> Result<Vec<u8>, String> {
    if records.is_empty() {
        return Err("Không có hồ sơ nào để xuất PDF!".into());
    }
    let (doc, page1, layer1) = PdfDocument::new(title, Mm(PAGE_W), Mm(PAGE_H), "Page");
    let fonts = load_fonts(&doc)?;
    let first_layer = doc.get_page(page1).get_layer(layer1);

    for (idx, form) in records.iter().enumerate() {
        render_patient_pages(
            &doc,
            &fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            idx == 0,
            if idx == 0 { Some(first_layer.clone()) } else { None },
        );
    }

    let mut buf = Vec::new();
    doc.save(&mut BufWriter::new(&mut buf))
        .map_err(|e| format!("Lỗi ghi PDF: {}", e))?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code128b_and_generate_pdf() {
        let bars = encode_code128b("HD2611-0001");
        assert!(bars.len() > 20);
        let mut r1 = Map::new();
        r1.insert("MA_TIEP_NHAN".into(), Value::String("HD2611-0001".into()));
        r1.insert("HO_TEN".into(), Value::String("NGUYỄN VĂN AN".into()));
        r1.insert("GIOI_TINH".into(), Value::String("1".into()));
        r1.insert("NGAY_SINH".into(), Value::String("12/03/1988".into()));
        r1.insert("CHI_SO_BMI".into(), Value::String("22.5".into()));
        r1.insert("NOI_KHOA_TUAN_HOAN".into(), Value::String("Nhịp tim đều, T1 T2 rõ".into()));
        r1.insert("PHAN_LOAI_SK".into(), Value::String("1".into()));
        let mut r2 = Map::new();
        r2.insert("MA_TIEP_NHAN".into(), Value::String("HD2611-0002".into()));
        r2.insert("HO_TEN".into(), Value::String("TRẦN THỊ BÍCH NGỌC".into()));
        r2.insert("GIOI_TINH".into(), Value::String("2".into()));
        let cfg = AppConfig::default();
        let mut cats = Map::new();
        let mut dt = Map::new();
        dt.insert("01".into(), Value::String("Kinh".into()));
        cats.insert("DAN_TOC".into(), Value::Object(dt));
        assert_eq!(get_cat_name(&cats, "DAN_TOC", "01"), "Kinh");
        assert_eq!(get_cat_name(&cats, "DAN_TOC", "1"), "Kinh");
        let bytes = generate_records_pdf(&[r1, r2], &cfg, &cats, false, "Test Group PDF")
            .expect("pdf generation should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 5000);

        for tpl in ["TREN_18", "LAI_XE", "DUOI_6", "TU_6_DUOI_18", "MAU_GIAO", "HS_PHO_THONG"] {
            let mut rec = Map::new();
            rec.insert("MA_TIEP_NHAN".into(), Value::String("TN2026-0099".into()));
            rec.insert("MA_Y_TE".into(), Value::String("BN00099".into()));
            rec.insert("HO_TEN".into(), Value::String("NGUYỄN TRẦN MINH KHÔI".into()));
            rec.insert("GIOI_TINH".into(), Value::String("1".into()));
            rec.insert("NGAY_SINH".into(), Value::String("15/05/1995".into()));
            rec.insert("SO_CCCD".into(), Value::String("052095001234".into()));
            rec.insert("NGAYCAP_CCCD".into(), Value::String("10/08/2021".into()));
            rec.insert("NOICAP_CCCD".into(), Value::String("Cục CSQLHC về TTXH".into()));
            rec.insert("MA_DAN_TOC".into(), Value::String("01".into()));
            rec.insert("DIA_CHI".into(), Value::String("123 Trần Hưng Đạo, Phường Hải Cảng, TP Quy Nhơn".into()));
            rec.insert("DIEN_THOAI".into(), Value::String("0912345678".into()));
            rec.insert("NGAY_VAO".into(), Value::String("16/11/2026".into()));
            rec.insert("HANG_BANG_LAI".into(), Value::String("B2".into()));
            rec.insert("TEN_CSGD".into(), Value::String("Trường Mầm Non / THPT Lê Quý Đôn".into()));
            rec.insert("HO_TEN_NGUOI_GIAM_HO".into(), Value::String("NGUYỄN VĂN HÙNG".into()));
            rec.insert("MOI_QUAN_HE_VOI_TRE".into(), Value::String("Cha".into()));
            rec.insert("DIEN_THOAI_GIAM_HO".into(), Value::String("0905111222".into()));
            rec.insert("CHI_SO_BMI".into(), Value::String("22.5".into()));
            rec.insert("CHIEU_CAO".into(), Value::String("170".into()));
            rec.insert("CAN_NANG".into(), Value::String("65".into()));
            rec.insert("MACH".into(), Value::String("76".into()));
            rec.insert("HUYET_AP".into(), Value::String("120/80".into()));
            rec.insert("NOI_KHOA_TUAN_HOAN".into(), Value::String("Nhịp tim đều, T1 T2 rõ, không tiếng thổi bệnh lý".into()));
            rec.insert("NOI_KHOA_TUAN_HOAN_PL".into(), Value::String("1".into()));
            rec.insert("BS_NOI".into(), Value::String("BS.CKI Trần Văn Bình".into()));
            rec.insert("PHAN_LOAI_SK".into(), Value::String("1".into()));
            rec.insert("BS_KET_LUAN".into(), Value::String("BS.CKII Lê Hoàng Nam".into()));
            rec.insert("LX_KET_LUAN_SK".into(), Value::String("Đủ điều kiện sức khỏe lái xe hạng".into()));
            rec.insert("LX_XN_MA_TUY".into(), Value::String("Âm tính".into()));
            rec.insert("LX_XN_KHAC_KQ".into(), Value::String("Nồng độ cồn: Âm tính (0 mg/100ml)".into()));
            rec.insert("LX_BS_KET_LUAN".into(), Value::String("BS.CKII Lê Hoàng Nam".into()));
            rec.insert("_PRINT_TEMPLATE".into(), Value::String(tpl.into()));
            let pdf = generate_records_pdf(&[rec], &cfg, &cats, false, tpl)
                .expect("custom template pdf should succeed");
            assert!(pdf.starts_with(b"%PDF-"));
            if let Ok(out_dir) = std::env::var("KSK_DUMP_PDF_PREVIEW") {
                let _ = std::fs::create_dir_all(&out_dir);
                let p = std::path::Path::new(&out_dir).join(format!("{}.pdf", tpl));
                std::fs::write(p, &pdf).unwrap();
            }
        }
    }
}
