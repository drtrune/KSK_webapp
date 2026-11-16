use printpdf::{
    Color, IndirectFontRef, Line, Mm, PdfDocumentReference, PdfLayerReference, Point, Rect, Rgb,
};

pub const PAGE_W: f32 = 210.0;
pub const PAGE_H: f32 = 297.0;
pub const MARGIN_L: f32 = 16.0;
pub const MARGIN_R: f32 = 14.0;
pub const CONTENT_W: f32 = PAGE_W - MARGIN_L - MARGIN_R; // 180mm
pub const TOP_Y: f32 = PAGE_H - 25.0; // 272mm (dưới barcode header)
pub const BOTTOM_Y: f32 = 18.0; // trên footer (- Trang X -)

pub const FONT_BODY: f32 = 11.5;
pub const FONT_SMALL: f32 = 10.5;
pub const FONT_SUBTITLE: f32 = 12.5;
pub const FONT_TITLE: f32 = 13.5;
pub const LINE_H: f32 = 5.0;

/// Bộ font Times/Serif gồm thường, đậm, nghiêng dùng xuyên suốt tài liệu PDF.
pub struct PdfFonts {
    pub regular: IndirectFontRef,
    pub bold: IndirectFontRef,
    pub italic: IndirectFontRef,
}

/// Đọc file font đầu tiên tồn tại trong danh sách đường dẫn ứng viên.
fn load_font_bytes(candidates: &[&str]) -> Result<Vec<u8>, String> {
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            return Ok(bytes);
        }
    }
    Err(format!("Không tìm thấy font hệ thống trong: {:?}", candidates))
}

/// Nạp bộ font hệ thống (Times New Roman / Liberation Serif) vào PDF document.
pub fn load_fonts(doc: &PdfDocumentReference) -> Result<PdfFonts, String> {
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

/// Bảng mẫu vạch Code 128 (giá trị 0..=106).
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

/// Mã hóa chuỗi ASCII sang dãy độ rộng vạch/khoảng trắng Code128B.
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
            widths.push(2); // Vạch kết thúc
        }
    }
    widths
}

/// Trả về màu đen chuẩn.
pub fn black() -> Color {
    Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None))
}

/// Trả về màu xám cho dòng kẻ chấm điền tay.
pub fn gray_line() -> Color {
    Color::Rgb(Rgb::new(0.45, 0.45, 0.45, None))
}

/// Vẽ một đoạn thẳng màu đen từ (x1, y1) tới (x2, y2).
pub fn draw_line(
    layer: &PdfLayerReference,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    thickness_pt: f32,
) {
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

/// Vẽ dòng kẻ mảnh màu xám cho người khám điền tay.
pub fn draw_dotted_line(layer: &PdfLayerReference, x1: f32, y: f32, x2: f32) {
    if x2 <= x1 + 1.0 {
        return;
    }
    layer.set_outline_color(gray_line());
    layer.set_outline_thickness(0.35);
    let line = Line {
        points: vec![
            (Point::new(Mm(x1), Mm(y)), false),
            (Point::new(Mm(x2), Mm(y)), false),
        ],
        is_closed: false,
    };
    layer.add_line(line);
}

/// Vẽ đường gạch đứt nằm ngang từ x1 đến x2 để phân tách nhẹ các phần trong ô.
pub fn draw_dashed_line(
    layer: &PdfLayerReference,
    x1: f32,
    y: f32,
    x2: f32,
    dash_mm: f32,
    gap_mm: f32,
    thickness_pt: f32,
) {
    if x2 <= x1 + 1.0 || dash_mm <= 0.1 {
        return;
    }
    layer.set_outline_color(gray_line());
    layer.set_outline_thickness(thickness_pt);
    let mut cx = x1;
    let step = dash_mm + gap_mm.max(0.3);
    while cx < x2 {
        let end = (cx + dash_mm).min(x2);
        let line = Line {
            points: vec![
                (Point::new(Mm(cx), Mm(y)), false),
                (Point::new(Mm(end), Mm(y)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
        cx += step;
    }
}

/// Vẽ khung viền hình chữ nhật (tọa độ đáy trái).
pub fn draw_rect_stroke(
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

/// Tô đặc hình chữ nhật màu đen (dùng vẽ vạch mã vạch).
pub fn draw_rect_fill(layer: &PdfLayerReference, x: f32, y_bottom: f32, w: f32, h: f32) {
    layer.set_fill_color(black());
    let rect = Rect::new(Mm(x), Mm(y_bottom), Mm(x + w), Mm(y_bottom + h))
        .with_mode(printpdf::path::PaintMode::Fill);
    layer.add_rect(rect);
}

/// Vẽ chuỗi văn bản tại tọa độ (x, y).
pub fn draw_text(
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

/// Ước lượng bề rộng chuỗi theo mm cho font Times New Roman để ngắt dòng an toàn.
pub fn approx_text_w_mm(text: &str, size_pt: f32) -> f32 {
    let pt_to_mm = 0.352778;
    let units: f32 = text
        .chars()
        .map(|c| {
            if matches!(c, 'm' | 'w' | 'M' | 'W' | 'Ô' | 'Ơ' | 'Ư' | 'Đ') {
                0.76
            } else if c.is_uppercase() {
                0.66
            } else if matches!(
                c,
                'i' | 'l' | 'I' | '.' | ',' | ':' | ';' | ' ' | '!' | '/' | '(' | ')' | '-'
            ) {
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

/// Vẽ chuỗi văn bản căn giữa theo hoành độ center_x.
pub fn draw_text_centered(
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

/// Vẽ ô vuông checkbox (kích thước 3.3mm) và đánh dấu X nếu checked = true.
pub fn draw_checkbox(layer: &PdfLayerReference, x: f32, y_baseline: f32, checked: bool) {
    let sz = 3.3;
    draw_rect_stroke(layer, x, y_baseline - 0.3, sz, sz, 0.55);
    if checked {
        draw_line(
            layer,
            x + 0.5,
            y_baseline + 0.3,
            x + sz - 0.5,
            y_baseline + sz - 0.8,
            0.75,
        );
        draw_line(
            layer,
            x + 0.5,
            y_baseline + sz - 0.8,
            x + sz - 0.5,
            y_baseline + 0.3,
            0.75,
        );
    }
}

/// Ngắt đoạn văn bản thành các dòng vừa khít bề rộng max_w_mm.
pub fn wrap_lines(text: &str, max_w_mm: f32, size_pt: f32) -> Vec<String> {
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

/// Vẽ mã vạch Code128B ở góc trên bên phải mỗi trang.
pub fn draw_barcode_header(layer: &PdfLayerReference, fonts: &PdfFonts, ma_tn: &str) {
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
        FONT_SMALL,
        x0 + actual_w / 2.0,
        y_bar - 4.2,
        &cap,
    );
}

/// Vẽ số trang ở chân trang.
pub fn draw_footer(layer: &PdfLayerReference, fonts: &PdfFonts, page_num: usize) {
    draw_text_centered(
        layer,
        &fonts.regular,
        FONT_SMALL,
        PAGE_W / 2.0,
        9.0,
        &format!("- Trang {} -", page_num),
    );
}

