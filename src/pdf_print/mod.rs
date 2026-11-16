use std::io::BufWriter;

use printpdf::{Mm, PdfDocument, PdfDocumentReference, PdfLayerReference};
use serde_json::{Map, Value};

use crate::config::AppConfig;

mod cursor;
mod forms;
mod primitives;
mod values;
mod widgets;

pub use primitives::encode_code128b;
use primitives::{load_fonts, PdfFonts, PAGE_H, PAGE_W};

/// Chọn hàm vẽ tương ứng với mã mẫu KSK của hồ sơ.
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
    let explicit_tpl = form
        .get("_PRINT_TEMPLATE")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let tpl_code = if !explicit_tpl.is_empty() {
        explicit_tpl.to_string()
    } else {
        let raw_list = form
            .get("EXAM_TEMPLATES")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        crate::templates::parse_template_list(raw_list)
            .into_iter()
            .next()
            .unwrap_or_else(|| "TREN_18".to_string())
    };

    match tpl_code.as_str() {
        "TREN_18" => forms::tren18::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        "LAI_XE" => forms::laixe::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        "DUOI_6" => forms::duoi6::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        "MAU_GIAO" => forms::maugiao::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        "TU_6_DUOI_18" => forms::u18::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        "HS_PHO_THONG" => forms::hocsinh::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
        _ => forms::tren18::render(
            doc,
            fonts,
            form,
            cat_map,
            cfg,
            is_reception_print,
            is_first_page_of_doc,
            first_layer,
        ),
    }
}

/// Tạo file PDF khổ A4 nhiều trang trực tiếp bằng Rust cho danh sách hồ sơ KSK.
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
            if idx == 0 {
                Some(first_layer.clone())
            } else {
                None
            },
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
    use values::{get_cat_name, FormValues};

    #[test]
    fn test_form_values_blank_and_year() {
        let mut form = Map::new();
        form.insert("HO_TEN".into(), Value::String("NGUYỄN VĂN A".into()));
        form.insert("NGAY_VAO".into(), Value::String("16/11/2026".into()));
        form.insert("NOI_KHOA_TUAN_HOAN".into(), Value::String("Bình thường".into()));
        form.insert("LX_TSGD_MAC_BENH".into(), Value::String("Không".into()));

        let cats = Map::new();
        let blank_fv = FormValues::new(&form, &cats, true, "LAI_XE");
        assert_eq!(blank_fv.admin("HO_TEN"), "NGUYỄN VĂN A");
        assert_eq!(blank_fv.text(&["NOI_KHOA_TUAN_HOAN"]), "");
        assert_eq!(blank_fv.ns_yes_no("TSGD_MAC_BENH"), None);
        assert_eq!(blank_fv.exam_year_2d(), "26");

        let filled_fv = FormValues::new(&form, &cats, false, "LAI_XE");
        assert_eq!(filled_fv.text(&["NOI_KHOA_TUAN_HOAN"]), "Bình thường");
        assert_eq!(filled_fv.ns_yes_no("TSGD_MAC_BENH"), Some(false));

        let mut addr_form = Map::new();
        addr_form.insert("DIA_CHI".into(), Value::String("Tổ 11, Phường Quy Nhơn Đông, Tỉnh Gia Lai".into()));
        addr_form.insert("MAXA_CU_TRU".into(), Value::String("01".into()));
        addr_form.insert("MATINH_CU_TRU".into(), Value::String("52".into()));
        let mut addr_cats = Map::new();
        let mut xa = Map::new();
        xa.insert("01".into(), Value::String("Phường Quy Nhơn Đông".into()));
        let mut tinh = Map::new();
        tinh.insert("52".into(), Value::String("Tỉnh Gia Lai".into()));
        addr_cats.insert("XA_PHUONG".into(), Value::Object(xa));
        addr_cats.insert("TINH_TP".into(), Value::Object(tinh));
        let addr_fv = FormValues::new(&addr_form, &addr_cats, true, "DUOI_6");
        assert_eq!(addr_fv.full_address(), "Tổ 11, Phường Quy Nhơn Đông, Tỉnh Gia Lai");
    }

    #[test]
    fn test_code128b_and_generate_all_templates_pdf() {
        let bars = encode_code128b("HD2611-0001");
        assert!(bars.len() > 20);

        let cfg = AppConfig::default();
        let mut cats = Map::new();
        let mut dt = Map::new();
        dt.insert("01".into(), Value::String("Kinh".into()));
        cats.insert("DAN_TOC".into(), Value::Object(dt));
        assert_eq!(get_cat_name(&cats, "DAN_TOC", "01"), "Kinh");
        assert_eq!(get_cat_name(&cats, "DAN_TOC", "1"), "Kinh");

        for tpl in [
            "TREN_18",
            "LAI_XE",
            "DUOI_6",
            "TU_6_DUOI_18",
            "MAU_GIAO",
            "HS_PHO_THONG",
        ] {
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
            rec.insert(
                "DIA_CHI".into(),
                Value::String("123 Trần Hưng Đạo, Phường Hải Cảng, TP Quy Nhơn".into()),
            );
            rec.insert("DIEN_THOAI".into(), Value::String("0912345678".into()));
            rec.insert("NGAY_VAO".into(), Value::String("16/11/2026".into()));
            rec.insert("HANG_BANG_LAI".into(), Value::String("B2".into()));
            rec.insert(
                "TEN_CSGD".into(),
                Value::String("Trường Mầm Non / THPT Lê Quý Đôn".into()),
            );
            rec.insert("HO_TEN_NGUOI_GIAM_HO".into(), Value::String("NGUYỄN VĂN HÙNG".into()));
            rec.insert("MOI_QUAN_HE_VOI_TRE".into(), Value::String("Cha".into()));
            rec.insert("DIEN_THOAI_GIAM_HO".into(), Value::String("0905111222".into()));
            rec.insert("CHI_SO_BMI".into(), Value::String("22.5".into()));
            rec.insert("CHIEU_CAO".into(), Value::String("170".into()));
            rec.insert("CAN_NANG".into(), Value::String("65".into()));
            rec.insert("MACH".into(), Value::String("76".into()));
            rec.insert("HUYET_AP".into(), Value::String("120/80".into()));
            rec.insert(
                "NOI_KHOA_TUAN_HOAN".into(),
                Value::String("Nhịp tim đều, T1 T2 rõ".into()),
            );
            rec.insert("NOI_KHOA_TUAN_HOAN_PL".into(), Value::String("1".into()));
            rec.insert("BS_NOI".into(), Value::String("BS.CKI Trần Văn Bình".into()));
            rec.insert("PHAN_LOAI_SK".into(), Value::String("1".into()));
            rec.insert("BS_KET_LUAN".into(), Value::String("BS.CKII Lê Hoàng Nam".into()));
            rec.insert("_PRINT_TEMPLATE".into(), Value::String(tpl.into()));

            for is_blank in [true, false] {
                let pdf = generate_records_pdf(&[rec.clone()], &cfg, &cats, is_blank, tpl)
                    .expect("pdf generation should succeed");
                assert!(pdf.starts_with(b"%PDF-"));
                assert!(pdf.len() > 3000);
                if !is_blank {
                    if let Ok(out_dir) = std::env::var("KSK_DUMP_PDF_PREVIEW") {
                        let _ = std::fs::create_dir_all(&out_dir);
                        let p = std::path::Path::new(&out_dir).join(format!("{}.pdf", tpl));
                        std::fs::write(p, &pdf).unwrap();
                    }
                }
            }
        }
    }
}

