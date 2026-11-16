use serde_json::{Map, Value};

/// Bọc dữ liệu hồ sơ KSK, tự động ẩn kết quả khám khi in phôi (is_reception_print = true).
pub struct FormValues<'a> {
    form: &'a Map<String, Value>,
    cat_map: &'a Map<String, Value>,
    pub blank: bool,
    pub tpl_code: &'a str,
}

impl<'a> FormValues<'a> {
    /// Khởi tạo bộ đọc dữ liệu cho một hồ sơ và mã mẫu.
    pub fn new(
        form: &'a Map<String, Value>,
        cat_map: &'a Map<String, Value>,
        blank: bool,
        tpl_code: &'a str,
    ) -> Self {
        Self {
            form,
            cat_map,
            blank,
            tpl_code,
        }
    }

    /// Đọc chuỗi thô theo khóa (không phụ thuộc chế độ phôi).
    pub fn admin(&self, key: &str) -> &str {
        self.form
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
    }

    /// Đọc chuỗi thô đầu tiên có giá trị trong danh sách khóa (không ẩn khi in phôi).
    pub fn admin_any(&self, keys: &[&str]) -> &str {
        for k in keys {
            let v = self.admin(k);
            if !v.is_empty() {
                return v;
            }
        }
        ""
    }

    /// Đọc giá trị khám theo danh sách khóa; trả về rỗng nếu đang in phôi.
    pub fn text(&self, keys: &[&str]) -> &str {
        if self.blank {
            return "";
        }
        self.admin_any(keys)
    }

    /// Đọc giá trị khám đã gắn tiền tố mẫu (LX_, D6_, MG_...); trả về rỗng nếu in phôi.
    pub fn ns(&self, raw_key: &str) -> &str {
        if self.blank {
            return "";
        }
        let k = crate::templates::namespaced_key(self.tpl_code, raw_key);
        self.form
            .get(&k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
    }

    /// Đọc giá trị Có/Không (trả về None nếu đang in phôi hoặc chưa nhập).
    pub fn yes_no(&self, key: &str) -> Option<bool> {
        if self.blank {
            return None;
        }
        parse_yes_no_value(self.form.get(key))
    }

    /// Đọc giá trị Có/Không đã gắn tiền tố mẫu (trả về None nếu đang in phôi hoặc chưa nhập).
    pub fn ns_yes_no(&self, raw_key: &str) -> Option<bool> {
        if self.blank {
            return None;
        }
        let k = crate::templates::namespaced_key(self.tpl_code, raw_key);
        parse_yes_no_value(self.form.get(&k))
    }

    /// Kiểm tra trường namespaced có khớp lựa chọn cho trước hay không (luôn false khi in phôi).
    pub fn ns_matches(&self, raw_key: &str, option: &str) -> bool {
        if self.blank {
            return false;
        }
        let val = self.ns(raw_key);
        !val.is_empty() && val.eq_ignore_ascii_case(option.trim())
    }

    /// Tra tên hiển thị trong danh mục theo mã.
    pub fn cat(&self, cat_key: &str, code: &str) -> String {
        get_cat_name(self.cat_map, cat_key, code)
    }

    /// Ghép địa chỉ đầy đủ (số nhà + phường/xã + tỉnh/thành), tránh lặp nếu DIA_CHI đã chứa sẵn.
    pub fn full_address(&self) -> String {
        let base = self.admin("DIA_CHI");
        let xa = self.cat("XA_PHUONG", self.admin("MAXA_CU_TRU"));
        let tinh = self.cat("TINH_TP", self.admin("MATINH_CU_TRU"));
        let base_low = base.to_lowercase();

        let mut parts: Vec<String> = Vec::new();
        if !base.is_empty() {
            parts.push(base.to_string());
        }
        if !xa.trim().is_empty() && !base_low.contains(&xa.trim().to_lowercase()) {
            parts.push(xa.trim().to_string());
        }
        if !tinh.trim().is_empty() && !base_low.contains(&tinh.trim().to_lowercase()) {
            parts.push(tinh.trim().to_string());
        }
        parts.join(", ")
    }

    /// Lấy 2 chữ số cuối của năm khám (ưu tiên NGAY_KHAM, NGAY_VAO; mặc định năm hiện tại).
    pub fn exam_year_2d(&self) -> String {
        for key in ["NGAY_KHAM", "NGAY_VAO"] {
            let s = self.admin(key);
            if let Some(yy) = extract_year_2d(s) {
                return yy;
            }
        }
        chrono::Local::now().format("%y").to_string()
    }

    /// Lấy ngày/tháng/năm khám phục vụ dòng ký tên (dd, mm, yyyy).
    pub fn exam_date_parts(&self) -> (String, String, String) {
        let raw = self.admin_any(&["NGAY_KHAM", "NGAY_VAO"]);
        let parts: Vec<&str> = raw.split('/').collect();
        if parts.len() == 3 && parts[2].len() >= 4 {
            return (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2][..4].to_string(),
            );
        }
        let year = chrono::Local::now().format("%Y").to_string();
        ("......".into(), "......".into(), year)
    }

    /// Tính tuổi từ NGAY_SINH dựa trên năm khám.
    pub fn age(&self) -> String {
        let ns = self.admin("NGAY_SINH");
        if ns.len() >= 4 {
            let ref_year = chrono::Local::now().format("%Y").to_string();
            let base_yr = ref_year.parse::<i32>().unwrap_or(2026);
            if let Ok(yr) = ns[ns.len() - 4..].parse::<i32>() {
                return (base_yr - yr).max(0).to_string();
            }
        }
        String::new()
    }
}

/// Chuyển một JSON Value thành Option<bool> cho cặp Có/Không.
fn parse_yes_no_value(val: Option<&Value>) -> Option<bool> {
    match val {
        Some(Value::Bool(b)) => Some(*b),
        Some(Value::Number(n)) => match n.as_i64() {
            Some(1) => Some(true),
            Some(0) => Some(false),
            _ => None,
        },
        Some(Value::String(s)) => {
            let t = s.trim().to_ascii_lowercase();
            if t == "1" || t == "true" || t == "co" || t == "có" {
                Some(true)
            } else if t == "0" || t == "false" || t == "khong" || t == "không" {
                Some(false)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Trích 2 chữ số cuối của năm từ chuỗi ngày dạng dd/mm/yyyy hoặc yyyy-mm-dd.
fn extract_year_2d(date_str: &str) -> Option<String> {
    let s = date_str.trim();
    if s.is_empty() {
        return None;
    }
    // Dạng dd/mm/yyyy hoặc dd-mm-yyyy (phần cuối là 4 chữ số)
    let digits_tail: String = s
        .rsplit(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap_or("")
        .to_string();
    if digits_tail.len() == 4 {
        return Some(digits_tail[2..4].to_string());
    }
    // Dạng yyyy-mm-dd (phần đầu là 4 chữ số)
    let digits_head: String = s
        .split(|c: char| !c.is_ascii_digit())
        .next()
        .unwrap_or("")
        .to_string();
    if digits_head.len() == 4 {
        return Some(digits_head[2..4].to_string());
    }
    None
}

/// Tra tên danh mục từ bản đồ cat_map (hỗ trợ cả Object lẫn Array).
pub fn get_cat_name(cat_map: &Map<String, Value>, cat_key: &str, code: &str) -> String {
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

