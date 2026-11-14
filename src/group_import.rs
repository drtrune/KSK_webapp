use crate::models::{CatalogItem, GroupImportRow};
use calamine::{open_workbook_auto, Data, Reader};
use chrono::NaiveDate;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub const IMPORT_HEADERS: &[&str] = &[
    "MA_TIEP_NHAN",
    "MA_Y_TE",
    "HO_TEN",
    "NGAY_SINH",
    "GIOI_TINH",
    "SO_CCCD",
    "NGAYCAP_CCCD",
    "NOICAP_CCCD",
    "DIEN_THOAI",
    "MATINH_CU_TRU",
    "MAXA_CU_TRU",
    "DIA_CHI",
    "MA_DAN_TOC",
    "MA_NGHE_NGHIEP",
    "NOI_LAM_VIEC_HOC_TAP",
    "MA_LOAI_KCB",
    "NGAY_VAO",
    "NHOM_MAU",
    "LY_DO_VV",
];

pub fn sample_group_import_csv_bytes() -> Vec<u8> {
    let mut out = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    let header = IMPORT_HEADERS.join(",");
    let sample = [
        "TN20260001",
        "BN000001",
        "NGUYỄN VĂN A",
        "15/05/1990",
        "1",
        "052090001234",
        "01/05/2021",
        "Cục CSQLHC về TTXH",
        "0905123456",
        "52",
        "21550",
        "123 Trần Hưng Đạo",
        "01",
        "01",
        "Công ty TNHH ABC",
        "01",
        "15/11/2026",
        "O",
        "Khám sức khỏe định kỳ",
    ]
    .join(",");
    out.extend_from_slice(format!("{}\r\n{}\r\n", header, sample).as_bytes());
    out
}

fn strip_vi_accents(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ'
            | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' | 'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ'
            | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'a',
            'đ' | 'Đ' => 'd',
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' | 'È' | 'É'
            | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'e',
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' | 'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'i',
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ'
            | 'ớ' | 'ở' | 'ỡ' | 'ợ' | 'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố'
            | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'o',
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' | 'Ù' | 'Ú'
            | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'u',
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' | 'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'y',
            other => other,
        })
        .collect()
}

fn norm_header(raw: &str) -> String {
    strip_vi_accents(raw.trim().trim_start_matches('\u{feff}'))
        .to_ascii_lowercase()
        .replace([' ', '_', '-', '.', '/', '(', ')'], "")
}

fn map_header_to_key(raw: &str) -> Option<&'static str> {
    let h = norm_header(raw);
    match h.as_str() {
        "matiepnhan" | "sotiepnhan" | "tiepnhan" => Some("MA_TIEP_NHAN"),
        "mayte" | "mabenhnhan" | "mabn" | "myte" | "his" => Some("MA_Y_TE"),
        "hoten" | "hovaten" | "tenbenhnhan" | "tennhanvien" => Some("HO_TEN"),
        "ngaysinh" | "namsinh" | "dob" => Some("NGAY_SINH"),
        "gioitinh" | "phai" | "sex" | "gender" => Some("GIOI_TINH"),
        "socccd" | "cccd" | "cmnd" | "socmnd" | "cccdcmnd" => Some("SO_CCCD"),
        "ngaycapcccd" | "ngaycap" | "ngaycapcmnd" => Some("NGAYCAP_CCCD"),
        "noicapcccd" | "noicap" | "noicapcmnd" => Some("NOICAP_CCCD"),
        "dienthoai" | "sodienthoai" | "sdt" | "phone" => Some("DIEN_THOAI"),
        "matinhcutru" | "matinh" | "tinh" | "tinhthanh" | "tinhthanhpho" => Some("MATINH_CU_TRU"),
        "maxacutru" | "maxa" | "xa" | "phuongxa" | "xaphuong" => Some("MAXA_CU_TRU"),
        "diachi" | "diachichitiet" | "sonha" => Some("DIA_CHI"),
        "madantoc" | "dantoc" => Some("MA_DAN_TOC"),
        "manghenghiep" | "nghenghiep" | "nghe" => Some("MA_NGHE_NGHIEP"),
        "noilamviechoctap" | "noilamviec" | "congty" | "donvi" | "bophan" | "phongban" => {
            Some("NOI_LAM_VIEC_HOC_TAP")
        }
        "maloaikcb" | "loaikcb" | "loaihinhkham" => Some("MA_LOAI_KCB"),
        "ngayvao" | "ngaykham" | "ngayvaokham" => Some("NGAY_VAO"),
        "nhommau" | "blood" => Some("NHOM_MAU"),
        "lydovv" | "lydokham" | "lydo" => Some("LY_DO_VV"),
        _ => None,
    }
}

fn normalize_date_str(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
        return d.format("%d/%m/%Y").to_string();
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return d.format("%d/%m/%Y").to_string();
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d-%m-%Y") {
        return d.format("%d/%m/%Y").to_string();
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y/%m/%d") {
        return d.format("%d/%m/%Y").to_string();
    }
    // Excel serial number fallback (days since 1899-12-30)
    if let Ok(serial) = s.parse::<f64>() {
        if serial > 1000.0 && serial < 100000.0 {
            if let Some(base) = NaiveDate::from_ymd_opt(1899, 12, 30) {
                if let Some(d) = base.checked_add_days(chrono::Days::new(serial.floor() as u64)) {
                    return d.format("%d/%m/%Y").to_string();
                }
            }
        }
    }
    s.to_string()
}

fn normalize_gender(raw: &str) -> String {
    let s = strip_vi_accents(raw.trim()).to_ascii_lowercase();
    match s.as_str() {
        "1" | "nam" | "m" | "male" => "1".to_string(),
        "2" | "nu" | "f" | "female" => "2".to_string(),
        "3" | "khac" | "other" => "3".to_string(),
        _ => "1".to_string(),
    }
}

fn resolve_catalog_code(raw: &str, cat_type: &str, catalogs: &[CatalogItem]) -> String {
    let t = raw.trim();
    if t.is_empty() {
        return String::new();
    }
    let t_low = strip_vi_accents(t).to_ascii_lowercase();
    for c in catalogs.iter().filter(|c| c.cat_type == cat_type && c.is_active == 1) {
        if c.code.eq_ignore_ascii_case(t) {
            return c.code.clone();
        }
        if strip_vi_accents(&c.name).to_ascii_lowercase() == t_low {
            return c.code.clone();
        }
    }
    t.to_string()
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.trim().to_string(),
        Data::Int(i) => i.to_string(),
        Data::Float(f) => {
            if f.fract() == 0.0 {
                format!("{}", *f as i64)
            } else {
                f.to_string()
            }
        }
        Data::Bool(b) => if *b { "1" } else { "0" }.to_string(),
        Data::DateTime(dt) => {
            let days = dt.as_f64();
            normalize_date_str(&days.to_string())
        }
        Data::DateTimeIso(s) => normalize_date_str(s),
        Data::DurationIso(s) => s.trim().to_string(),
        Data::Error(_) => String::new(),
    }
}

fn read_raw_table(path: &Path) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if ext == "csv" {
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .from_path(path)
            .map_err(|e| format!("Không thể mở file CSV: {}", e))?;
        let headers: Vec<String> = rdr
            .headers()
            .map_err(|e| format!("Lỗi đọc header CSV: {}", e))?
            .iter()
            .map(|s| s.trim().trim_start_matches('\u{feff}').to_string())
            .collect();
        let mut rows = Vec::new();
        for rec in rdr.records() {
            let rec = rec.map_err(|e| format!("Lỗi đọc dòng CSV: {}", e))?;
            rows.push(rec.iter().map(|s| s.trim().to_string()).collect());
        }
        return Ok((headers, rows));
    }

    let mut workbook = open_workbook_auto(path)
        .map_err(|e| format!("Không thể mở file Excel ({}): {}", ext, e))?;
    let sheet_name = workbook
        .sheet_names()
        .first()
        .cloned()
        .ok_or_else(|| "File Excel không có sheet nào!".to_string())?;
    let range = workbook
        .worksheet_range(&sheet_name)
        .map_err(|e| format!("Lỗi đọc sheet '{}': {}", sheet_name, e))?;

    let mut iter = range.rows();
    let header_row = iter
        .next()
        .ok_or_else(|| "File Excel trống, không có dòng tiêu đề!".to_string())?;
    let headers: Vec<String> = header_row.iter().map(cell_to_string).collect();
    let mut rows = Vec::new();
    for r in iter {
        let vals: Vec<String> = r.iter().map(cell_to_string).collect();
        if vals.iter().all(|v| v.trim().is_empty()) {
            continue;
        }
        rows.push(vals);
    }
    Ok((headers, rows))
}

pub fn parse_group_import_file(
    path: &Path,
    company_name: &str,
    default_exam_date: &str,
    catalogs: &[CatalogItem],
    existing_ma_tn: &HashSet<String>,
    existing_contract_cccds: &HashSet<String>,
) -> Result<Vec<GroupImportRow>, String> {
    let (headers, raw_rows) = read_raw_table(path)?;
    let mut col_map: HashMap<&'static str, usize> = HashMap::new();
    for (idx, h) in headers.iter().enumerate() {
        if let Some(key) = map_header_to_key(h) {
            col_map.entry(key).or_insert(idx);
        }
    }

    if !col_map.contains_key("HO_TEN") {
        return Err("File thiếu cột Họ và tên (HO_TEN / Họ và tên)!".into());
    }
    if !col_map.contains_key("MA_TIEP_NHAN") {
        return Err("File thiếu cột Mã tiếp nhận (MA_TIEP_NHAN / Số tiếp nhận)!".into());
    }

    let get_col = |row: &[String], key: &str| -> String {
        col_map
            .get(key)
            .and_then(|&idx| row.get(idx))
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    };

    let today = chrono::Local::now().format("%d/%m/%Y").to_string();
    let fallback_exam_date = if default_exam_date.trim().is_empty() {
        today
    } else {
        normalize_date_str(default_exam_date)
    };

    let mut seen_ma_tn: HashSet<String> = HashSet::new();
    let mut seen_cccd: HashSet<String> = HashSet::new();
    let mut out = Vec::with_capacity(raw_rows.len());

    for (i, row) in raw_rows.iter().enumerate() {
        let row_index = i + 2; // 1-based + header
        let ma_tiep_nhan = get_col(row, "MA_TIEP_NHAN");
        let ma_y_te = get_col(row, "MA_Y_TE");
        let ho_ten = get_col(row, "HO_TEN");
        let ngay_sinh = normalize_date_str(&get_col(row, "NGAY_SINH"));
        let gioi_tinh = normalize_gender(&get_col(row, "GIOI_TINH"));
        let so_cccd = get_col(row, "SO_CCCD");
        let ngaycap_cccd = normalize_date_str(&get_col(row, "NGAYCAP_CCCD"));
        let noicap_cccd = get_col(row, "NOICAP_CCCD");
        let dien_thoai = get_col(row, "DIEN_THOAI");
        let matinh_cu_tru = resolve_catalog_code(&get_col(row, "MATINH_CU_TRU"), "TINH_TP", catalogs);
        let maxa_cu_tru = resolve_catalog_code(&get_col(row, "MAXA_CU_TRU"), "XA_PHUONG", catalogs);
        let dia_chi = get_col(row, "DIA_CHI");

        let raw_dt = get_col(row, "MA_DAN_TOC");
        let ma_dan_toc = if raw_dt.is_empty() {
            "01".to_string()
        } else {
            resolve_catalog_code(&raw_dt, "DAN_TOC", catalogs)
        };
        let ma_nghe_nghiep =
            resolve_catalog_code(&get_col(row, "MA_NGHE_NGHIEP"), "NGHE_NGHIEP", catalogs);

        let raw_wp = get_col(row, "NOI_LAM_VIEC_HOC_TAP");
        let noi_lam_viec_hoc_tap = if raw_wp.is_empty() {
            company_name.trim().to_string()
        } else {
            raw_wp
        };

        let raw_kcb = get_col(row, "MA_LOAI_KCB");
        let ma_loai_kcb = if raw_kcb.is_empty() {
            "01".to_string()
        } else {
            resolve_catalog_code(&raw_kcb, "LOAI_KCB", catalogs)
        };

        let raw_nv = get_col(row, "NGAY_VAO");
        let ngay_vao = if raw_nv.is_empty() {
            fallback_exam_date.clone()
        } else {
            normalize_date_str(&raw_nv)
        };

        let nhom_mau = get_col(row, "NHOM_MAU");
        let raw_lydo = get_col(row, "LY_DO_VV");
        let ly_do_vv = if raw_lydo.is_empty() {
            "Khám sức khỏe".to_string()
        } else {
            raw_lydo
        };

        let mut errs: Vec<String> = Vec::new();
        if ho_ten.is_empty() {
            errs.push("Thiếu Họ tên".into());
        }
        if ma_tiep_nhan.is_empty() {
            errs.push("Thiếu Mã tiếp nhận".into());
        } else {
            if existing_ma_tn.contains(&ma_tiep_nhan) {
                errs.push(format!("Mã tiếp nhận '{}' đã tồn tại trong DB", ma_tiep_nhan));
            }
            if !seen_ma_tn.insert(ma_tiep_nhan.clone()) {
                errs.push(format!("Trùng Mã tiếp nhận '{}' trong file", ma_tiep_nhan));
            }
        }
        if !so_cccd.is_empty() {
            if existing_contract_cccds.contains(&so_cccd) {
                errs.push(format!("CCCD '{}' đã có trong hợp đồng này", so_cccd));
            }
            if !seen_cccd.insert(so_cccd.clone()) {
                errs.push(format!("Trùng CCCD '{}' trong file", so_cccd));
            }
        }

        out.push(GroupImportRow {
            row_index,
            ma_tiep_nhan,
            ma_y_te,
            ho_ten,
            ngay_sinh,
            gioi_tinh,
            so_cccd,
            ngaycap_cccd,
            noicap_cccd,
            dien_thoai,
            matinh_cu_tru,
            maxa_cu_tru,
            dia_chi,
            ma_dan_toc,
            ma_nghe_nghiep,
            noi_lam_viec_hoc_tap,
            ma_loai_kcb,
            ngay_vao,
            nhom_mau,
            ly_do_vv,
            error: if errs.is_empty() {
                None
            } else {
                Some(errs.join("; "))
            },
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_sample_csv_parse_and_duplicate_detection() {
        let bytes = sample_group_import_csv_bytes();
        let tmp_dir = std::env::temp_dir();
        let tmp_path = tmp_dir.join("ksk_test_group_import.csv");
        let mut f = std::fs::File::create(&tmp_path).unwrap();
        f.write_all(&bytes).unwrap();
        drop(f);

        let existing_tn = HashSet::new();
        let existing_cccd = HashSet::new();
        let rows = parse_group_import_file(
            &tmp_path,
            "Công ty ABC",
            "15/11/2026",
            &[],
            &existing_tn,
            &existing_cccd,
        )
        .unwrap();
        let _ = std::fs::remove_file(&tmp_path);

        assert_eq!(rows.len(), 1);
        assert!(rows[0].error.is_none());
        assert_eq!(rows[0].ma_tiep_nhan, "TN20260001");
        assert_eq!(rows[0].ma_y_te, "BN000001");
        assert_eq!(rows[0].ho_ten, "NGUYỄN VĂN A");
        assert_eq!(rows[0].gioi_tinh, "1");
    }
}
