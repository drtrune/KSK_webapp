use std::io::BufWriter;

use printpdf::{
    Color, IndirectFontRef, Line, Mm, PdfDocument, PdfDocumentReference, PdfLayerReference, Point,
    Rect, Rgb,
};
use serde_json::{Map, Value};

use crate::config::AppConfig;

const PAGE_W: f32 = 210.0;
const PAGE_H: f32 = 297.0;
const MARGIN_L: f32 = 14.0;
const MARGIN_R: f32 = 14.0;
const CONTENT_W: f32 = PAGE_W - MARGIN_L - MARGIN_R; // 182mm

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

/// Approximate text width in mm for Times New Roman (used for centering/right-align).
fn approx_text_w_mm(text: &str, size_pt: f32) -> f32 {
    // Average glyph width ~ 0.48 * font_size_pt; 1 pt = 0.352778 mm
    let pt_to_mm = 0.352778;
    let units: f32 = text
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() || matches!(c, 'Đ' | 'Â' | 'Ê' | 'Ô' | 'Ơ' | 'Ư' | 'Ă') {
                0.62
            } else if matches!(c, 'i' | 'l' | 'I' | '.' | ',' | ':' | ';' | ' ' | '!' | '/') {
                0.28
            } else if matches!(c, 'm' | 'w' | 'M' | 'W') {
                0.78
            } else {
                0.46
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
    let sz = 3.0;
    draw_rect_stroke(layer, x, y_baseline - 0.3, sz, sz, 0.55);
    if checked {
        draw_line(layer, x + 0.5, y_baseline + 0.2, x + sz - 0.5, y_baseline + sz - 0.8, 0.7);
        draw_line(layer, x + 0.5, y_baseline + sz - 0.8, x + sz - 0.5, y_baseline + 0.2, 0.7);
    }
}

fn wrap_lines(text: &str, max_w_mm: f32, size_pt: f32) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for para in text.split('\n') {
        let mut cur = String::new();
        for word in para.split_whitespace() {
            let candidate = if cur.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", cur, word)
            };
            if approx_text_w_mm(&candidate, size_pt) <= max_w_mm || cur.is_empty() {
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

fn draw_barcode_header(
    layer: &PdfLayerReference,
    fonts: &PdfFonts,
    ma_tn: &str,
) {
    let code = if ma_tn.trim().is_empty() { "000000" } else { ma_tn.trim() };
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
    let y_bar = PAGE_H - 16.5;
    let bar_h = 7.5;

    let mut cx = x0;
    for (idx, &w) in widths.iter().enumerate() {
        let w_mm = w as f32 * unit_w;
        if idx % 2 == 0 {
            draw_rect_fill(layer, cx, y_bar, w_mm, bar_h);
        }
        cx += w_mm;
    }

    let cap = format!("Mã tiếp nhận: {}", code);
    draw_text_centered(layer, &fonts.bold, 8.5, x0 + actual_w / 2.0, y_bar - 3.3, &cap);
}

fn draw_footer(layer: &PdfLayerReference, fonts: &PdfFonts, page_num: usize) {
    draw_text_centered(
        layer,
        &fonts.regular,
        10.0,
        PAGE_W / 2.0,
        9.0,
        &format!("- Trang {}/5 -", page_num),
    );
}

fn get_s<'a>(form: &'a Map<String, Value>, key: &str) -> &'a str {
    form.get(key).and_then(|v| v.as_str()).unwrap_or("").trim()
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

fn draw_label_val_line(
    layer: &PdfLayerReference,
    fonts: &PdfFonts,
    x: f32,
    y: f32,
    max_x: f32,
    label: &str,
    val: &str,
    size_pt: f32,
    bold_val: bool,
) {
    draw_text(layer, &fonts.bold, size_pt, x, y, label);
    let vx = x + approx_text_w_mm(label, size_pt) + 1.5;
    if vx < max_x {
        draw_dotted_line(layer, vx, y - 0.8, max_x);
        if !val.is_empty() {
            let f = if bold_val { &fonts.bold } else { &fonts.regular };
            draw_text(layer, f, size_pt, vx + 0.8, y, val);
        }
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
    let ma_tn = get_s(form, "MA_TIEP_NHAN");
    let v = |key: &str| -> &str {
        if is_reception_print { "" } else { get_s(form, key) }
    };
    let b = |key: &str| -> bool {
        if is_reception_print { false } else { get_b(form, key) }
    };

    // ==================== TRANG 1 / 5 ====================
    let l1 = if is_first_page_of_doc {
        first_layer.unwrap()
    } else {
        let (p, l) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
        doc.get_page(p).get_layer(l)
    };
    draw_barcode_header(&l1, fonts, ma_tn);
    draw_footer(&l1, fonts, 1);

    let left_cx = MARGIN_L + 38.0;
    let right_cx = MARGIN_L + 132.0;
    let mut y = PAGE_H - 24.0;
    draw_text_centered(&l1, &fonts.regular, 11.5, left_cx, y, "BỘ Y TẾ");
    draw_text_centered(
        &l1,
        &fonts.bold,
        11.5,
        right_cx,
        y,
        "CỘNG HÒA XÃ HỘI CHỦ NGHĨA VIỆT NAM",
    );
    y -= 4.8;
    let cskcb = if cfg.ten_cskcb.trim().is_empty() {
        "BỆNH VIỆN BÌNH ĐỊNH"
    } else {
        cfg.ten_cskcb.trim()
    };
    draw_text_centered(&l1, &fonts.bold, 11.5, left_cx, y, &cskcb.to_uppercase());
    draw_text_centered(
        &l1,
        &fonts.bold,
        11.5,
        right_cx,
        y,
        "Độc lập - Tự do - Hạnh phúc",
    );
    y -= 4.2;
    draw_text_centered(&l1, &fonts.regular, 11.0, left_cx, y, "—————————");
    draw_text_centered(&l1, &fonts.regular, 11.0, right_cx, y, "—————————");
    y -= 4.8;
    let ngay_vao = get_s(form, "NGAY_VAO");
    let yy = if ngay_vao.len() >= 2 {
        &ngay_vao[ngay_vao.len() - 2..]
    } else {
        "26"
    };
    let so_phieu = get_s(form, "SO_PHIEU");
    let so_line = format!("Số: {}/GKSK-{}/{}", if so_phieu.is_empty() { "......" } else { so_phieu }, cfg.ma_cskcb, yy);
    draw_text_centered(&l1, &fonts.regular, 11.0, left_cx, y, &so_line);

    y -= 8.0;
    draw_text_centered(
        &l1,
        &fonts.bold,
        13.0,
        PAGE_W / 2.0,
        y,
        "GIẤY KHÁM SỨC KHỎE VÀ KHÁM SỨC KHỎE ĐỊNH KỲ DÙNG CHO",
    );
    y -= 5.5;
    draw_text_centered(
        &l1,
        &fonts.bold,
        13.0,
        PAGE_W / 2.0,
        y,
        "NGƯỜI TỪ ĐỦ 18 TUỔI TRỞ LÊN",
    );

    // Thông tin hành chính 1..13 (không khung ảnh)
    y -= 7.0;
    let adm_x = MARGIN_L;
    let max_x = PAGE_W - MARGIN_R;
    let mut ay = y;

    let ho_ten = get_s(form, "HO_TEN").to_uppercase();
    draw_label_val_line(&l1, fonts, adm_x, ay, max_x, "1. Họ và tên (viết chữ in hoa):", &ho_ten, 12.0, true);
    ay -= 5.8;

    let gt = get_s(form, "GIOI_TINH");
    draw_text(&l1, &fonts.bold, 12.0, adm_x, ay, "2. Giới tính:");
    draw_checkbox(&l1, adm_x + 28.0, ay, gt == "1");
    draw_text(&l1, &fonts.regular, 12.0, adm_x + 33.0, ay, "Nam");
    draw_checkbox(&l1, adm_x + 48.0, ay, gt == "2");
    draw_text(&l1, &fonts.regular, 12.0, adm_x + 53.0, ay, "Nữ");
    ay -= 5.8;

    let ns = get_s(form, "NGAY_SINH");
    let tuoi = if ns.len() >= 4 {
        ns[ns.len() - 4..]
            .parse::<i32>()
            .ok()
            .map(|yr| (2026 - yr).to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };
    draw_label_val_line(&l1, fonts, adm_x, ay, adm_x + 95.0, "3. Ngày tháng năm sinh:", ns, 12.0, false);
    draw_label_val_line(&l1, fonts, adm_x + 98.0, ay, max_x, "Tuổi:", &tuoi, 12.0, true);
    ay -= 5.8;

    draw_label_val_line(&l1, fonts, adm_x, ay, max_x, "4. Số CCCD/Hộ chiếu/Mã định danh:", get_s(form, "SO_CCCD"), 12.0, false);
    ay -= 5.8;

    draw_label_val_line(&l1, fonts, adm_x, ay, adm_x + 74.0, "5. Cấp ngày:", get_s(form, "NGAYCAP_CCCD"), 12.0, false);
    draw_label_val_line(&l1, fonts, adm_x + 78.0, ay, max_x, "Nơi cấp:", get_s(form, "NOICAP_CCCD"), 12.0, false);
    ay -= 5.8;

    let dantoc = get_cat_name(cat_map, "DAN_TOC", get_s(form, "MA_DAN_TOC"));
    let doituong = get_cat_name(cat_map, "DOI_TUONG", get_s(form, "DOI_TUONG"));
    draw_label_val_line(&l1, fonts, adm_x, ay, adm_x + 74.0, "6. Dân tộc:", &dantoc, 12.0, false);
    draw_label_val_line(&l1, fonts, adm_x + 78.0, ay, max_x, "7. Đối tượng:", &doituong, 12.0, false);
    ay -= 5.8;

    let nguon = get_cat_name(cat_map, "NGUON_CHI_TRA", get_s(form, "NGUON_CHI_TRA"));
    draw_label_val_line(&l1, fonts, adm_x, ay, max_x, "8. Nguồn chi trả:", &nguon, 12.0, false);
    ay -= 5.8;

    draw_label_val_line(&l1, fonts, adm_x, ay, adm_x + 68.0, "9. Nhóm máu:", get_s(form, "NHOM_MAU"), 12.0, false);
    draw_label_val_line(&l1, fonts, adm_x + 72.0, ay, max_x, "Điện thoại:", get_s(form, "DIEN_THOAI"), 12.0, false);
    ay -= 5.8;

    let tinh = get_cat_name(cat_map, "TINH_TP", get_s(form, "MATINH_CU_TRU"));
    let xa = get_cat_name(cat_map, "XA_PHUONG", get_s(form, "MAXA_CU_TRU"));
    draw_label_val_line(&l1, fonts, MARGIN_L, ay, MARGIN_L + 100.0, "10. Nơi ở hiện tại: Tỉnh/TP:", &tinh, 12.0, false);
    draw_label_val_line(&l1, fonts, MARGIN_L + 103.0, ay, max_x, "Phường/xã:", &xa, 12.0, false);
    ay -= 5.8;

    draw_label_val_line(&l1, fonts, MARGIN_L + 6.0, ay, max_x, "Số nhà/thôn/xóm:", get_s(form, "DIA_CHI"), 12.0, false);
    ay -= 5.8;

    let nghe = get_cat_name(cat_map, "NGHE_NGHIEP", get_s(form, "MA_NGHE_NGHIEP"));
    draw_label_val_line(&l1, fonts, MARGIN_L, ay, max_x, "11. Nghề nghiệp:", &nghe, 12.0, false);
    ay -= 5.8;

    draw_label_val_line(&l1, fonts, MARGIN_L, ay, max_x, "12. Nơi làm việc, học tập:", get_s(form, "NOI_LAM_VIEC_HOC_TAP"), 12.0, false);
    ay -= 5.8;

    let ly_do = if get_s(form, "LY_DO_VV").is_empty() {
        "Khám sức khỏe"
    } else {
        get_s(form, "LY_DO_VV")
    };
    draw_label_val_line(&l1, fonts, MARGIN_L, ay, max_x, "13. Lý do khám sức khỏe:", ly_do, 12.0, false);

    // Tiền sử bệnh
    y = ay - 8.5;
    draw_text_centered(&l1, &fonts.bold, 12.5, PAGE_W / 2.0, y, "TIỀN SỬ BỆNH CỦA ĐỐI TƯỢNG KHÁM SỨC KHỎE");
    y -= 5.5;
    draw_text(&l1, &fonts.bold, 11.5, MARGIN_L, y, "1. Tiền sử gia đình:");
    draw_text(&l1, &fonts.regular, 11.0, MARGIN_L + 38.0, y, "Có ai trong gia đình mắc bệnh: truyền nhiễm, tim mạch, ĐTĐ, lao, hen, ung thư...");
    y -= 5.2;
    let ts_gd_co = !v("TS_GIA_DINH_CHI_TIET").is_empty() || b("TS_GIA_DINH");
    draw_text(&l1, &fonts.regular, 11.0, MARGIN_L + 4.0, y, "a)");
    draw_checkbox(&l1, MARGIN_L + 10.0, y, !is_reception_print && !ts_gd_co);
    draw_text(&l1, &fonts.regular, 11.0, MARGIN_L + 14.5, y, "Không");
    draw_text(&l1, &fonts.regular, 11.0, MARGIN_L + 30.0, y, "b)");
    draw_checkbox(&l1, MARGIN_L + 35.0, y, ts_gd_co);
    draw_label_val_line(&l1, fonts, MARGIN_L + 39.5, y, max_x, "Có; Ghi cụ thể:", v("TS_GIA_DINH_CHI_TIET"), 11.0, false);

    y -= 6.5;
    draw_text(&l1, &fonts.bold, 11.5, MARGIN_L, y, "2. Tiền sử bản thân:");
    draw_text(&l1, &fonts.regular, 11.0, MARGIN_L + 38.0, y, "Ông (bà) đã/đang mắc bệnh, tình trạng nào sau đây không:");
    y -= 3.5;

    let ts_items_p1: &[(&str, &str, &str)] = &[
        ("1", "Có bệnh hay bị thương trong 5 năm qua", "TSBT_BENH_TRONG_5_NAM_QUA"),
        ("2", "Có bệnh thần kinh hay bị thương ở đầu", "TSBT_BENH_THAN_KINH"),
        ("3", "Bệnh mắt hoặc giảm thị lực (trừ trường hợp đeo kính thuốc)", "TSBT_BENH_MAT"),
        ("4", "Bệnh ở tai, giảm sức nghe hoặc thăng bằng", "TSBT_BENH_TAI"),
        ("5", "Bệnh ở tim, hoặc nhồi máu cơ tim, bệnh tim mạch khác", "TSBT_BENH_TIM"),
        ("6", "Phẫu thuật can thiệp tim - mạch (thay van, bắc cầu nối, tạo hình mạch, đặt stent...)", "TSBT_PHAU_THUAT_TIM"),
    ];
    draw_ts_table(&l1, fonts, MARGIN_L, y, CONTENT_W, ts_items_p1, &b, is_reception_print);

    // ==================== TRANG 2 / 5 ====================
    let (p2, l2_idx) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
    let l2 = doc.get_page(p2).get_layer(l2_idx);
    draw_barcode_header(&l2, fonts, ma_tn);
    draw_footer(&l2, fonts, 2);

    let mut y2 = PAGE_H - 25.0;
    draw_text(&l2, &fonts.bold, 10.5, MARGIN_L, y2, "2. Tiền sử bản thân (tiếp theo):");
    y2 -= 3.5;
    let ts_items_p2: &[(&str, &str, &str)] = &[
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
        ("17", "Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ ban ngày, ngáy to", "TSBT_ROI_LOAN_GIAC_NGU"),
        ("18", "Tai biến mạch máu não hoặc liệt", "TSBT_TAI_BIEN"),
        ("19", "Bệnh hoặc tổn thương cột sống", "TSBT_BENH_COT_SONG"),
        ("20", "Sử dụng rượu thường xuyên, liên tục", "TSBT_RUOU_THUONG_XUYEN"),
        ("21", "Sử dụng ma túy và chất gây nghiện", "TSBT_MA_TUY"),
        ("22", "Bệnh khác (ghi rõ):", "TSBT_BENH_KHAC"),
    ];
    let table_end_y2 = draw_ts_table(&l2, fonts, MARGIN_L, y2, CONTENT_W, ts_items_p2, &b, is_reception_print);

    y2 = table_end_y2 - 7.0;
    draw_text(&l2, &fonts.bold, 10.5, MARGIN_L, y2, "3. Câu hỏi khác (nếu có):");
    y2 -= 5.0;
    draw_text(&l2, &fonts.regular, 10.0, MARGIN_L + 3.0, y2, "a) Ông (bà) có đang điều trị bệnh gì không?");
    y2 -= 4.8;
    let q3a = v("TS_DANG_DIEU_TRI");
    draw_checkbox(&l2, MARGIN_L + 8.0, y2, !is_reception_print && q3a.is_empty());
    draw_text(&l2, &fonts.regular, 10.0, MARGIN_L + 12.5, y2, "Không");
    draw_checkbox(&l2, MARGIN_L + 30.0, y2, !q3a.is_empty());
    draw_label_val_line(&l2, fonts, MARGIN_L + 34.5, y2, max_x, "Có (liệt kê tên bệnh, thuốc):", q3a, 10.0, false);
    y2 -= 5.5;
    draw_dotted_line(&l2, MARGIN_L + 8.0, y2, max_x);

    y2 -= 6.5;
    draw_text(&l2, &fonts.regular, 10.0, MARGIN_L + 3.0, y2, "b) Tiền sử thai sản (Đối với phụ nữ):");
    y2 -= 4.8;
    let q3b = v("TS_THAI_SAN");
    draw_checkbox(&l2, MARGIN_L + 8.0, y2, !is_reception_print && q3b.is_empty());
    draw_text(&l2, &fonts.regular, 10.0, MARGIN_L + 12.5, y2, "Không");
    draw_checkbox(&l2, MARGIN_L + 30.0, y2, !q3b.is_empty());
    draw_label_val_line(&l2, fonts, MARGIN_L + 34.5, y2, max_x, "Có (liệt kê):", q3b, 10.0, false);
    y2 -= 5.5;
    draw_dotted_line(&l2, MARGIN_L + 8.0, y2, max_x);

    y2 -= 10.0;
    draw_text(
        &l2,
        &fonts.italic,
        10.0,
        MARGIN_L,
        y2,
        "Tôi xin cam đoan những điều khai trên đây hoàn toàn đúng với sự thật theo sự hiểu biết của tôi.",
    );
    let sig_cx2 = MARGIN_L + 135.0;
    let dia_danh = if cfg.dia_danh.trim().is_empty() { "Gia Lai" } else { cfg.dia_danh.trim() };
    draw_text_centered(&l2, &fonts.italic, 10.0, sig_cx2, y2, &format!("{}, ngày ..... tháng ..... năm 20....", dia_danh));
    draw_text_centered(&l2, &fonts.bold, 10.5, sig_cx2, y2 - 5.0, "NGƯỜI ĐỀ NGHỊ KHÁM SỨC KHỎE");
    draw_text_centered(&l2, &fonts.italic, 9.5, sig_cx2, y2 - 9.5, "(Ký và ghi rõ họ, tên)");

    // ==================== TRANG 3 / 5 ====================
    let (p3, l3_idx) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
    let l3 = doc.get_page(p3).get_layer(l3_idx);
    draw_barcode_header(&l3, fonts, ma_tn);
    draw_footer(&l3, fonts, 3);

    let mut y3 = PAGE_H - 25.0;
    draw_text(&l3, &fonts.bold, 11.0, MARGIN_L, y3, "I. KHÁM THỂ LỰC");
    y3 -= 5.5;
    draw_label_val_line(&l3, fonts, MARGIN_L, y3, MARGIN_L + 55.0, "• Chiều cao (cm):", v("CHIEU_CAO"), 10.0, false);
    draw_label_val_line(&l3, fonts, MARGIN_L + 58.0, y3, MARGIN_L + 115.0, "• Cân nặng (kg):", v("CAN_NANG"), 10.0, false);
    draw_label_val_line(&l3, fonts, MARGIN_L + 118.0, y3, max_x, "• BMI:", v("BMI"), 10.0, false);
    y3 -= 5.5;
    draw_label_val_line(&l3, fonts, MARGIN_L, y3, MARGIN_L + 60.0, "• Mạch (lần/phút):", v("MACH"), 10.0, false);
    let ha = if !v("HUYET_AP_TAM_THU").is_empty() || !v("HUYET_AP_TAM_TRUONG").is_empty() {
        format!("{}/{}", v("HUYET_AP_TAM_THU"), v("HUYET_AP_TAM_TRUONG"))
    } else {
        v("HUYET_AP").to_string()
    };
    draw_label_val_line(&l3, fonts, MARGIN_L + 64.0, y3, max_x, "• Huyết áp (mmHg):", &ha, 10.0, false);
    y3 -= 5.5;
    draw_label_val_line(&l3, fonts, MARGIN_L, y3, max_x, "• Phân loại thể lực:", v("PHAN_LOAI_THE_LUC"), 10.0, false);
    y3 -= 6.5;
    draw_text_centered(&l3, &fonts.bold, 10.0, MARGIN_L + 140.0, y3, "Người đo thể lực");
    draw_text_centered(&l3, &fonts.italic, 9.5, MARGIN_L + 140.0, y3 - 4.2, "(Ký, ghi rõ họ tên)");
    if !is_reception_print && !v("NGUOI_DO_THE_LUC").is_empty() {
        draw_text_centered(&l3, &fonts.bold, 10.0, MARGIN_L + 140.0, y3 - 13.0, v("NGUOI_DO_THE_LUC"));
    }

    y3 -= 19.0;
    draw_text(&l3, &fonts.bold, 11.0, MARGIN_L, y3, "II. KHÁM LÂM SÀNG");
    y3 -= 3.5;
    let col1_w = CONTENT_W * 0.74;
    let col2_w = CONTENT_W - col1_w;
    // Header row
    draw_rect_stroke(&l3, MARGIN_L, y3 - 7.0, col1_w, 7.0, 0.55);
    draw_rect_stroke(&l3, MARGIN_L + col1_w, y3 - 7.0, col2_w, 7.0, 0.55);
    draw_text_centered(&l3, &fonts.bold, 10.0, MARGIN_L + col1_w / 2.0, y3 - 4.8, "Nội dung khám");
    draw_text_centered(&l3, &fonts.bold, 10.0, MARGIN_L + col1_w + col2_w / 2.0, y3 - 4.8, "Họ tên và chữ ký Bác sỹ");
    y3 -= 7.0;

    let noi_items: &[(&str, &str, &str)] = &[
        ("a) Tuần hoàn:", "KHAM_NOI_TUAN_HOAN", "PL_NOI_TUAN_HOAN"),
        ("b) Hô hấp:", "KHAM_NOI_HO_HAP", "PL_NOI_HO_HAP"),
        ("c) Tiêu hóa:", "KHAM_NOI_TIEU_HOA", "PL_NOI_TIEU_HOA"),
        ("d) Thận - Tiết niệu - Sinh dục:", "KHAM_NOI_THAN_TN", "PL_NOI_THAN_TN"),
        ("đ) Nội tiết:", "KHAM_NOI_TIET", "PL_NOI_TIET"),
        ("e) Cơ – xương – khớp:", "KHAM_NOI_CO_XUONG_KHOP", "PL_NOI_CO_XUONG_KHOP"),
        ("g) Thần kinh:", "KHAM_NOI_THAN_KINH", "PL_NOI_THAN_KINH"),
        ("h) Tâm thần:", "KHAM_NOI_TAM_THAN", "PL_NOI_TAM_THAN"),
    ];
    let noi_top = y3;
    // 1. Nội khoa title row
    draw_rect_stroke(&l3, MARGIN_L, y3 - 6.0, col1_w, 6.0, 0.5);
    draw_text(&l3, &fonts.bold, 10.0, MARGIN_L + 2.0, y3 - 4.2, "1. Nội khoa");
    y3 -= 6.0;
    for &(lbl, val_k, pl_k) in noi_items {
        let rh = 11.5;
        draw_rect_stroke(&l3, MARGIN_L, y3 - rh, col1_w, rh, 0.5);
        draw_label_val_line(&l3, fonts, MARGIN_L + 2.0, y3 - 4.2, MARGIN_L + col1_w - 2.0, lbl, v(val_k), 9.8, false);
        draw_label_val_line(&l3, fonts, MARGIN_L + 2.0, y3 - 9.2, MARGIN_L + 65.0, "Phân loại:", v(pl_k), 9.5, false);
        y3 -= rh;
    }
    draw_rect_stroke(&l3, MARGIN_L + col1_w, y3, col2_w, noi_top - y3, 0.5);
    if !is_reception_print && !v("BS_NOI_KHOA").is_empty() {
        draw_text_centered(&l3, &fonts.bold, 9.5, MARGIN_L + col1_w + col2_w / 2.0, y3 + 12.0, v("BS_NOI_KHOA"));
    }

    for (title, val_k, pl_k, bs_k) in [
        ("2. Ngoại khoa:", "KHAM_NGOAI_KHOA", "PL_NGOAI_KHOA", "BS_NGOAI_KHOA"),
        ("3. Da liễu:", "KHAM_DA_LIEU", "PL_DA_LIEU", "BS_DA_LIEU"),
        ("4. Sản phụ khoa (đối với nữ):", "KHAM_SAN_PHU_KHOA", "PL_SAN_PHU_KHOA", "BS_SAN_PHU_KHOA"),
    ] {
        let rh = 17.5;
        draw_rect_stroke(&l3, MARGIN_L, y3 - rh, col1_w, rh, 0.5);
        draw_rect_stroke(&l3, MARGIN_L + col1_w, y3 - rh, col2_w, rh, 0.5);
        draw_text(&l3, &fonts.bold, 10.0, MARGIN_L + 2.0, y3 - 4.2, title);
        draw_dotted_line(&l3, MARGIN_L + 2.0, y3 - 9.5, MARGIN_L + col1_w - 2.0);
        if !v(val_k).is_empty() {
            draw_text(&l3, &fonts.regular, 9.8, MARGIN_L + 2.5, y3 - 8.8, v(val_k));
        }
        draw_label_val_line(&l3, fonts, MARGIN_L + 2.0, y3 - 14.8, MARGIN_L + 65.0, "Phân loại:", v(pl_k), 9.5, false);
        if !is_reception_print && !v(bs_k).is_empty() {
            draw_text_centered(&l3, &fonts.bold, 9.5, MARGIN_L + col1_w + col2_w / 2.0, y3 - 12.0, v(bs_k));
        }
        y3 -= rh;
    }

    // ==================== TRANG 4 / 5 ====================
    let (p4, l4_idx) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
    let l4 = doc.get_page(p4).get_layer(l4_idx);
    draw_barcode_header(&l4, fonts, ma_tn);
    draw_footer(&l4, fonts, 4);

    let mut y4 = PAGE_H - 25.0;
    draw_text(&l4, &fonts.bold, 11.0, MARGIN_L, y4, "II. KHÁM LÂM SÀNG (TIẾP THEO)");
    y4 -= 3.5;
    draw_rect_stroke(&l4, MARGIN_L, y4 - 7.0, col1_w, 7.0, 0.55);
    draw_rect_stroke(&l4, MARGIN_L + col1_w, y4 - 7.0, col2_w, 7.0, 0.55);
    draw_text_centered(&l4, &fonts.bold, 10.0, MARGIN_L + col1_w / 2.0, y4 - 4.8, "Nội dung khám");
    draw_text_centered(&l4, &fonts.bold, 10.0, MARGIN_L + col1_w + col2_w / 2.0, y4 - 4.8, "Họ tên và chữ ký Bác sỹ");
    y4 -= 7.0;

    // 5. Mắt
    let h_mat = 30.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_mat, col1_w, h_mat, 0.5);
    draw_rect_stroke(&l4, MARGIN_L + col1_w, y4 - h_mat, col2_w, h_mat, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "5. Mắt: Kết quả khám thị lực:");
    draw_label_val_line(&l4, fonts, MARGIN_L + 4.0, y4 - 9.5, MARGIN_L + 60.0, "• Không kính: Mắt phải:", v("MAT_KHONG_KINH_PHAI"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 64.0, y4 - 9.5, MARGIN_L + 115.0, "Mắt trái:", v("MAT_KHONG_KINH_TRAI"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 4.0, y4 - 15.0, MARGIN_L + 60.0, "• Có kính:     Mắt phải:", v("MAT_CO_KINH_PHAI"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 64.0, y4 - 15.0, MARGIN_L + 115.0, "Mắt trái:", v("MAT_CO_KINH_TRAI"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 2.0, y4 - 20.5, MARGIN_L + col1_w - 2.0, "Các bệnh về mắt (nếu có):", v("KHAM_MAT"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 2.0, y4 - 26.5, MARGIN_L + 65.0, "Phân loại:", v("PL_MAT"), 9.5, false);
    if !v("BS_MAT").is_empty() {
        draw_text_centered(&l4, &fonts.bold, 9.5, MARGIN_L + col1_w + col2_w / 2.0, y4 - 20.0, v("BS_MAT"));
    }
    y4 -= h_mat;

    // 6. Tai - Mũi - Họng
    let h_tmh = 30.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_tmh, col1_w, h_tmh, 0.5);
    draw_rect_stroke(&l4, MARGIN_L + col1_w, y4 - h_tmh, col2_w, h_tmh, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "6. Tai – Mũi – Họng: Kết quả khám thính lực:");
    draw_label_val_line(&l4, fonts, MARGIN_L + 4.0, y4 - 9.5, MARGIN_L + 62.0, "• Tai trái:  Nói thường (m):", v("TMH_TAI_TRAI_THUONG"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 66.0, y4 - 9.5, MARGIN_L + 120.0, "Nói thầm (m):", v("TMH_TAI_TRAI_THAM"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 4.0, y4 - 15.0, MARGIN_L + 62.0, "• Tai phải: Nói thường (m):", v("TMH_TAI_PHAI_THUONG"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 66.0, y4 - 15.0, MARGIN_L + 120.0, "Nói thầm (m):", v("TMH_TAI_PHAI_THAM"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 2.0, y4 - 20.5, MARGIN_L + col1_w - 2.0, "Các bệnh về tai mũi họng:", v("KHAM_TMH"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 2.0, y4 - 26.5, MARGIN_L + 65.0, "Phân loại:", v("PL_TMH"), 9.5, false);
    if !v("BS_TMH").is_empty() {
        draw_text_centered(&l4, &fonts.bold, 9.5, MARGIN_L + col1_w + col2_w / 2.0, y4 - 20.0, v("BS_TMH"));
    }
    y4 -= h_tmh;

    // 7. Răng - Hàm - Mặt
    let h_rhm = 28.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_rhm, col1_w, h_rhm, 0.5);
    draw_rect_stroke(&l4, MARGIN_L + col1_w, y4 - h_rhm, col2_w, h_rhm, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "7. Răng – Hàm – Mặt:");
    draw_label_val_line(&l4, fonts, MARGIN_L + 4.0, y4 - 9.5, MARGIN_L + col1_w - 2.0, "• Hàm trên:", v("RHM_HAM_TREN"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 4.0, y4 - 14.5, MARGIN_L + col1_w - 2.0, "• Hàm dưới:", v("RHM_HAM_DUOI"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 2.0, y4 - 19.5, MARGIN_L + col1_w - 2.0, "Các bệnh về răng hàm mặt:", v("KHAM_RHM"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 2.0, y4 - 24.8, MARGIN_L + 65.0, "Phân loại:", v("PL_RHM"), 9.5, false);
    if !v("BS_RHM").is_empty() {
        draw_text_centered(&l4, &fonts.bold, 9.5, MARGIN_L + col1_w + col2_w / 2.0, y4 - 18.0, v("BS_RHM"));
    }
    y4 -= h_rhm;

    // III. KHÁM CẬN LÂM SÀNG
    y4 -= 7.0;
    draw_text(&l4, &fonts.bold, 11.0, MARGIN_L, y4, "III. KHÁM CẬN LÂM SÀNG");
    y4 -= 4.5;
    draw_text(&l4, &fonts.italic, 9.5, MARGIN_L, y4, "Khám phân loại sức khỏe để đi học, đi làm thực hiện khám cận lâm sàng sau:");
    y4 -= 3.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - 7.0, col1_w, 7.0, 0.55);
    draw_rect_stroke(&l4, MARGIN_L + col1_w, y4 - 7.0, col2_w, 7.0, 0.55);
    draw_text_centered(&l4, &fonts.bold, 10.0, MARGIN_L + col1_w / 2.0, y4 - 4.8, "Nội dung khám");
    draw_text_centered(&l4, &fonts.bold, 10.0, MARGIN_L + col1_w + col2_w / 2.0, y4 - 4.8, "Họ tên và chữ ký Bác sỹ");
    y4 -= 7.0;

    let cls_top = y4;
    let h_mau = 23.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_mau, col1_w, h_mau, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "a) Xét nghiệm máu:");
    draw_label_val_line(&l4, fonts, MARGIN_L + 3.0, y4 - 9.5, MARGIN_L + col1_w - 2.0, "• Huyết học (TPTTB máu ngoại vi):", v("XN_HUYET_HOC"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 3.0, y4 - 14.8, MARGIN_L + 50.0, "• Đường máu:", v("XN_DUONG_MAU"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 52.0, y4 - 14.8, MARGIN_L + 92.0, "Urê:", v("XN_URE"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 94.0, y4 - 14.8, MARGIN_L + col1_w - 2.0, "Creatinin:", v("XN_CREATININ"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 5.0, y4 - 20.0, MARGIN_L + 66.0, "ASAT (GOT):", v("XN_GOT"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 68.0, y4 - 20.0, MARGIN_L + col1_w - 2.0, "ALAT (GPT):", v("XN_GPT"), 9.5, false);
    y4 -= h_mau;

    let h_nt = 16.5;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_nt, col1_w, h_nt, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "b) Xét nghiệm nước tiểu:");
    draw_label_val_line(&l4, fonts, MARGIN_L + 3.0, y4 - 9.5, MARGIN_L + col1_w - 2.0, "• Tổng phân tích nước tiểu:", v("XN_NUOC_TIEU"), 9.5, false);
    draw_label_val_line(&l4, fonts, MARGIN_L + 3.0, y4 - 14.5, MARGIN_L + col1_w - 2.0, "• Khác (nếu có):", v("XN_NUOC_TIEU_KHAC"), 9.5, false);
    y4 -= h_nt;

    let h_xq = 22.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_xq, col1_w, h_xq, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "c) Chẩn đoán hình ảnh (XQ tim phổi thẳng):");
    for line_i in 0..3 {
        draw_dotted_line(&l4, MARGIN_L + 3.0, y4 - 9.5 - (line_i as f32) * 5.2, MARGIN_L + col1_w - 3.0);
    }
    for (idx, line) in wrap_lines(v("CDHA_XQ"), col1_w - 8.0, 9.5).into_iter().take(3).enumerate() {
        draw_text(&l4, &fonts.regular, 9.5, MARGIN_L + 3.5, y4 - 8.8 - (idx as f32) * 5.2, &line);
    }
    y4 -= h_xq;

    let h_clsk = 27.0;
    draw_rect_stroke(&l4, MARGIN_L, y4 - h_clsk, col1_w, h_clsk, 0.5);
    draw_text(&l4, &fonts.bold, 10.0, MARGIN_L + 2.0, y4 - 4.2, "d) Kết quả khám Cận lâm sàng khác:");
    let clsk = v("CLS_KHAC");
    draw_checkbox(&l4, MARGIN_L + 4.0, y4 - 9.2, !is_reception_print && clsk.is_empty());
    draw_text(&l4, &fonts.regular, 9.5, MARGIN_L + 8.5, y4 - 9.2, "Không");
    draw_checkbox(&l4, MARGIN_L + 24.0, y4 - 9.2, !clsk.is_empty());
    draw_text(&l4, &fonts.regular, 9.5, MARGIN_L + 28.5, y4 - 9.2, "Có (nếu có, liệt kê chi tiết):");
    for line_i in 0..3 {
        draw_dotted_line(&l4, MARGIN_L + 3.0, y4 - 14.5 - (line_i as f32) * 5.2, MARGIN_L + col1_w - 3.0);
    }
    for (idx, line) in wrap_lines(clsk, col1_w - 8.0, 9.5).into_iter().take(3).enumerate() {
        draw_text(&l4, &fonts.regular, 9.5, MARGIN_L + 3.5, y4 - 13.8 - (idx as f32) * 5.2, &line);
    }
    y4 -= h_clsk;
    draw_rect_stroke(&l4, MARGIN_L + col1_w, y4, col2_w, cls_top - y4, 0.5);
    if !v("BS_CLS").is_empty() {
        draw_text_centered(&l4, &fonts.bold, 9.5, MARGIN_L + col1_w + col2_w / 2.0, y4 + 18.0, v("BS_CLS"));
    }

    // ==================== TRANG 5 / 5 ====================
    let (p5, l5_idx) = doc.add_page(Mm(PAGE_W), Mm(PAGE_H), "Page");
    let l5 = doc.get_page(p5).get_layer(l5_idx);
    draw_barcode_header(&l5, fonts, ma_tn);
    draw_footer(&l5, fonts, 5);

    let mut y5 = PAGE_H - 25.0;
    draw_text(&l5, &fonts.bold, 11.0, MARGIN_L, y5, "III. KHÁM CẬN LÂM SÀNG (TIẾP THEO)");
    y5 -= 4.5;
    draw_text(&l5, &fonts.italic, 9.5, MARGIN_L, y5, "Khám sức khỏe định kỳ (cận lâm sàng thực hiện theo chỉ định của bác sỹ)");
    y5 -= 3.5;
    draw_rect_stroke(&l5, MARGIN_L, y5 - 7.0, col1_w, 7.0, 0.55);
    draw_rect_stroke(&l5, MARGIN_L + col1_w, y5 - 7.0, col2_w, 7.0, 0.55);
    draw_text_centered(&l5, &fonts.bold, 10.0, MARGIN_L + col1_w / 2.0, y5 - 4.8, "Nội dung khám");
    draw_text_centered(&l5, &fonts.bold, 10.0, MARGIN_L + col1_w + col2_w / 2.0, y5 - 4.8, "Họ tên và chữ ký Bác sỹ");
    y5 -= 7.0;

    let h_cls_dk = 62.0;
    draw_rect_stroke(&l5, MARGIN_L, y5 - h_cls_dk, col1_w, h_cls_dk, 0.5);
    draw_rect_stroke(&l5, MARGIN_L + col1_w, y5 - h_cls_dk, col2_w, h_cls_dk, 0.5);
    draw_text(&l5, &fonts.bold, 10.0, MARGIN_L + 3.0, y5 - 5.0, "Cận lâm sàng thực hiện theo chỉ định của bác sỹ:");
    draw_text(&l5, &fonts.regular, 10.0, MARGIN_L + 3.0, y5 - 10.5, "Kết quả:");
    for line_i in 0..8 {
        draw_dotted_line(&l5, MARGIN_L + 3.0, y5 - 16.5 - (line_i as f32) * 5.8, MARGIN_L + col1_w - 3.0);
    }
    for (idx, line) in wrap_lines(v("CLS_THEO_CHI_DINH"), col1_w - 8.0, 9.8).into_iter().take(8).enumerate() {
        draw_text(&l5, &fonts.regular, 9.8, MARGIN_L + 3.5, y5 - 15.8 - (idx as f32) * 5.8, &line);
    }
    y5 -= h_cls_dk;

    // IV. KẾT LUẬN
    y5 -= 10.0;
    draw_text(&l5, &fonts.bold, 12.0, MARGIN_L, y5, "IV. KẾT LUẬN");
    y5 -= 6.0;
    draw_text(&l5, &fonts.bold, 10.5, MARGIN_L, y5, "1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có):");
    for line_i in 0..5 {
        draw_dotted_line(&l5, MARGIN_L + 4.0, y5 - 6.0 - (line_i as f32) * 6.0, max_x);
    }
    let kl_benh = v("KET_LUAN_BENH");
    for (idx, line) in wrap_lines(kl_benh, CONTENT_W - 8.0, 10.0).into_iter().take(5).enumerate() {
        draw_text(&l5, &fonts.regular, 10.0, MARGIN_L + 4.5, y5 - 5.2 - (idx as f32) * 6.0, &line);
    }
    y5 -= 38.0;

    draw_label_val_line(
        &l5,
        fonts,
        MARGIN_L,
        y5,
        max_x,
        "2. Phân loại sức khỏe (theo quy định của Bộ Y tế):",
        v("PHAN_LOAI_SUC_KHOE"),
        10.5,
        true,
    );

    y5 -= 14.0;
    let sig_cx5 = MARGIN_L + 135.0;
    draw_text_centered(&l5, &fonts.italic, 10.5, sig_cx5, y5, &format!("{}, ngày ..... tháng ..... năm 20....", dia_danh));
    draw_text_centered(&l5, &fonts.bold, 11.5, sig_cx5, y5 - 5.5, "NGƯỜI KẾT LUẬN");
    draw_text_centered(&l5, &fonts.italic, 10.0, sig_cx5, y5 - 10.5, "(Ký, ghi rõ họ tên và đóng dấu)");
    if !is_reception_print && !v("NGUOI_KET_LUAN").is_empty() {
        draw_text_centered(&l5, &fonts.bold, 10.5, sig_cx5, y5 - 28.0, v("NGUOI_KET_LUAN"));
    }
}

fn draw_ts_table<F>(
    layer: &PdfLayerReference,
    fonts: &PdfFonts,
    x: f32,
    mut y: f32,
    w: f32,
    items: &[(&str, &str, &str)],
    get_bool: &F,
    is_reception: bool,
) -> f32
where
    F: Fn(&str) -> bool,
{
    let w_tt = w * 0.06;
    let w_name = w * 0.74;
    let w_co = w * 0.10;
    let w_khong = w - w_tt - w_name - w_co;

    let hh = 6.5;
    draw_rect_stroke(layer, x, y - hh, w_tt, hh, 0.55);
    draw_rect_stroke(layer, x + w_tt, y - hh, w_name, hh, 0.55);
    draw_rect_stroke(layer, x + w_tt + w_name, y - hh, w_co, hh, 0.55);
    draw_rect_stroke(layer, x + w_tt + w_name + w_co, y - hh, w_khong, hh, 0.55);

    draw_text_centered(layer, &fonts.bold, 10.8, x + w_tt / 2.0, y - 4.5, "TT");
    draw_text_centered(layer, &fonts.bold, 10.8, x + w_tt + w_name / 2.0, y - 4.5, "Tên bệnh, tật");
    draw_text_centered(layer, &fonts.bold, 10.8, x + w_tt + w_name + w_co / 2.0, y - 4.5, "Có");
    draw_text_centered(layer, &fonts.bold, 10.8, x + w_tt + w_name + w_co + w_khong / 2.0, y - 4.5, "Không");
    y -= hh;

    for &(tt, label, key) in items {
        let lines = wrap_lines(label, w_name - 4.0, 10.5);
        let rh = if lines.len() > 1 { 4.4 + (lines.len() as f32) * 4.4 } else { 7.6 };
        draw_rect_stroke(layer, x, y - rh, w_tt, rh, 0.5);
        draw_rect_stroke(layer, x + w_tt, y - rh, w_name, rh, 0.5);
        draw_rect_stroke(layer, x + w_tt + w_name, y - rh, w_co, rh, 0.5);
        draw_rect_stroke(layer, x + w_tt + w_name + w_co, y - rh, w_khong, rh, 0.5);

        let mid_y = y - rh / 2.0 - 1.2;
        draw_text_centered(layer, &fonts.regular, 10.5, x + w_tt / 2.0, mid_y, tt);
        for (li, line) in lines.iter().enumerate() {
            draw_text(layer, &fonts.regular, 10.5, x + w_tt + 2.0, y - 4.6 - (li as f32) * 4.4, line);
        }

        let checked_co = !is_reception && get_bool(key);
        let checked_khong = false;
        draw_checkbox(layer, x + w_tt + w_name + w_co / 2.0 - 1.5, mid_y, checked_co);
        draw_checkbox(layer, x + w_tt + w_name + w_co + w_khong / 2.0 - 1.5, mid_y, checked_khong);
        y -= rh;
    }
    y
}

/// Generates a multi-page A4 PDF (5 pages per patient) directly in Rust.
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
        let bytes = generate_records_pdf(&[r1, r2], &cfg, &cats, true, "Test Group PDF")
            .expect("pdf generation should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 5000);
    }
}
