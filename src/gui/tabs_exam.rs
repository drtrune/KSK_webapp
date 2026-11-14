use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder as _, px, rgb,
};
use gpui_component::{
    Disableable as _, Sizable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    input::Input,
};

use crate::models::phan_loai_config;

use super::{
    KskGpuiApp,
    widgets::{
        BG_CARD, BG_SUBTLE, BORDER_COL, DANGER_BG, DANGER_COL, PRIMARY_COL, PRIMARY_LIGHT,
        SUCCESS_BG, SUCCESS_COL, TEXT_MAIN, TEXT_MUTED, WARN_BG, WARN_COL, badge, card_box,
    },
};

const TIEN_SU_21_ITEMS: &[(&str, &str)] = &[
    ("TSBT_BENH_TRONG_5_NAM_QUA", "1. Bệnh trong 5 năm qua"),
    ("TSBT_BENH_THAN_KINH", "2. Bệnh thần kinh"),
    ("TSBT_BENH_MAT", "3. Bệnh mắt"),
    ("TSBT_BENH_TAI", "4. Bệnh tai"),
    ("TSBT_BENH_TIM", "5. Bệnh tim mạch"),
    ("TSBT_PHAU_THUAT_TIM", "6. Phẫu thuật tim"),
    ("TSBT_TANG_HUYET_AP", "7. Tăng huyết áp"),
    ("TSBT_KHO_THO", "8. Khó thở"),
    ("TSBT_BENH_PHOI", "9. Bệnh phổi / hen"),
    ("TSBT_BENH_THAN", "10. Bệnh thận"),
    ("TSBT_NGHIEN_RUOU", "11. Nghiện rượu"),
    ("TSBT_DAI_THAO_DUONG", "12. Đái tháo đường"),
    ("TSBT_BENH_TAM_THAN", "13. Bệnh tâm thần"),
    ("TSBT_MAT_Y_THUC", "14. Mất ý thức / động kinh"),
    ("TSBT_NGAT", "15. Ngất"),
    ("TSBT_BENH_TIEU_HOA", "16. Bệnh tiêu hóa"),
    ("TSBT_ROI_LOAN_GIAC_NGU", "17. Rối loạn giấc ngủ"),
    ("TSBT_TAI_BIEN", "18. Tai biến mạch máu não"),
    ("TSBT_BENH_COT_SONG", "19. Bệnh cột sống"),
    ("TSBT_RUOU_THUONG_XUYEN", "20. Uống rượu thường xuyên"),
    ("TSBT_MA_TUY", "21. Sử dụng ma túy"),
];

impl KskGpuiApp {
    pub fn render_exam_workspace(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if self.selected_record_id.is_none() && !self.is_creating_new {
            return div()
                .flex_1()
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    card_box()
                        .w(px(440.0))
                        .items_center()
                        .child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(rgb(TEXT_MAIN))
                                .child("Chưa chọn hồ sơ bệnh nhân"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .child("Chọn một hồ sơ ở danh sách bên trái, quét mã vạch, hoặc nhấn '+ Tiếp nhận hồ sơ mới'."),
                        ),
                )
                .into_any_element();
        }

        let is_locked = self.current_record_status == 2;
        let ho_ten = self.get_form_val("HO_TEN");
        let ma_bn = self.get_form_val("MA_Y_TE");
        let ma_tn = self.get_form_val("MA_TIEP_NHAN");

        let tabs = [
            ("reception", "1. Tiếp nhận"),
            ("history", "2. Tiền sử"),
            ("physical", "3. Thể lực"),
            ("clinical", "4. Lâm sàng"),
            ("paraclinical", "5. Cận lâm sàng"),
            ("conclusion", "6. Kết luận"),
        ];

        let active = self.active_exam_tab.clone();

        let body = match active.as_str() {
            "reception" => self.render_tab_reception(is_locked, window, cx),
            "history" => self.render_tab_history(is_locked, window, cx),
            "physical" => self.render_tab_physical(is_locked, window, cx),
            "clinical" => self.render_tab_clinical(is_locked, window, cx),
            "paraclinical" => self.render_tab_paraclinical(is_locked, window, cx),
            "conclusion" => self.render_tab_conclusion(is_locked, window, cx),
            _ => self.render_tab_reception(is_locked, window, cx),
        };

        div()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                // Patient banner + sub-tabs
                div()
                    .bg(rgb(BG_CARD))
                    .border_b_1()
                    .border_color(rgb(BORDER_COL))
                    .px(px(16.0))
                    .pt(px(10.0))
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(10.0))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(TEXT_MAIN))
                                            .child(if self.is_creating_new {
                                                "Hồ sơ tiếp nhận mới".to_string()
                                            } else if ho_ten.is_empty() {
                                                "(Chưa có họ tên)".to_string()
                                            } else {
                                                ho_ten
                                            }),
                                    )
                                    .when(!ma_bn.is_empty(), |el| {
                                        el.child(badge(format!("Mã BN: {}", ma_bn), BG_SUBTLE, TEXT_MAIN))
                                    })
                                    .when(!ma_tn.is_empty(), |el| {
                                        el.child(badge(format!("Số TN: {}", ma_tn), PRIMARY_LIGHT, PRIMARY_COL))
                                    }),
                            )
                            .child(match self.current_record_status {
                                2 => badge("Đã ký khóa", DANGER_BG, DANGER_COL),
                                1 => badge("Hoàn tất", SUCCESS_BG, SUCCESS_COL),
                                _ => badge("Đang khám", WARN_BG, WARN_COL),
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(4.0))
                            .children(tabs.iter().map(|(key, lbl)| {
                                let is_act = active == *key;
                                let k = key.to_string();
                                div()
                                    .id(SharedString::from(format!("exam-tab-{}", key)))
                                    .px(px(12.0))
                                    .py(px(8.0))
                                    .cursor_pointer()
                                    .border_b_2()
                                    .border_color(rgb(if is_act { PRIMARY_COL } else { BG_CARD }))
                                    .text_sm()
                                    .font_weight(if is_act {
                                        gpui::FontWeight::SEMIBOLD
                                    } else {
                                        gpui::FontWeight::NORMAL
                                    })
                                    .text_color(rgb(if is_act { PRIMARY_COL } else { TEXT_MUTED }))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.active_exam_tab = k.clone();
                                        this.open_dropdown = None;
                                        cx.notify();
                                    }))
                                    .child(*lbl)
                            })),
                    ),
            )
            .child(
                div()
                    .id("exam-body-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .p(px(16.0))
                    .child(body),
            )
            .into_any_element()
    }

    fn render_tab_reception(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_role = self.has_any_role(&["admin", "reception"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("reception");
        let gioi_tinh = self.get_form_val("GIOI_TINH");
        let ma_tinh = self.get_form_val("MATINH_CU_TRU");
        let ma_xa = self.get_form_val("MAXA_CU_TRU");
        let ma_dantoc = self.get_form_val("MA_DAN_TOC");
        let ma_nghe = self.get_form_val("MA_NGHE_NGHIEP");
        let nhom_mau = self.get_form_val("NHOM_MAU");
        let kcb = self.get_form_val("MA_LOAI_KCB");

        let gender_opts = vec![
            ("1".to_string(), "1 - Nam".to_string()),
            ("2".to_string(), "2 - Nữ".to_string()),
            ("3".to_string(), "3 - Khác".to_string()),
        ];
        let tinh_opts = self.query_catalog_options("TINH_TP", None);
        let xa_opts = if ma_tinh.is_empty() {
            vec![("".to_string(), "-- Chọn Tỉnh/TP trước --".to_string())]
        } else {
            self.query_catalog_options("XA_PHUONG", Some(&ma_tinh))
        };
        let dantoc_opts = self.query_catalog_options("DAN_TOC", None);
        let nghe_opts = self.query_catalog_options("NGHE_NGHIEP", None);
        let nhommau_opts = self.query_catalog_options("NHOM_MAU", None);
        let mut kcb_opts = self.query_catalog_options("LOAI_KCB", None);
        if kcb_opts.is_empty() {
            kcb_opts = vec![
                ("1".to_string(), "1 - Khám sức khỏe định kỳ".to_string()),
                ("2".to_string(), "2 - Khám sức khỏe tuyển dụng".to_string()),
                ("3".to_string(), "3 - Khám sức khỏe lái xe".to_string()),
            ];
        }

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "reception",
                        "Thông tin hành chính & Tiếp nhận",
                        can_role,
                        is_locked,
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("HO_TEN", "Họ và tên", "NGUYỄN VĂN A", true, editable, window, cx))
                            .child(self.render_form_input("MA_Y_TE", "Mã bệnh nhân (HIS)", "BN...", true, editable, window, cx))
                            .child(self.render_form_input("MA_TIEP_NHAN", "Số tiếp nhận", "TN...", false, editable, window, cx))
                            .child(self.render_form_input("NGAY_SINH", "Ngày sinh (YYYYMMDD)", "19900101", true, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_select_dropdown(
                                "rec-gioi-tinh",
                                "Giới tính",
                                &gioi_tinh,
                                gender_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("GIOI_TINH".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("SO_CCCD", "Số CCCD / CMND", "052...", false, editable, window, cx))
                            .child(self.render_form_input("NGAYCAP_CCCD", "Ngày cấp CCCD", "20210501", false, editable, window, cx))
                            .child(self.render_form_input("NOICAP_CCCD", "Nơi cấp CCCD", "Cục CSQLHC...", false, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("DIEN_THOAI", "Số điện thoại", "09...", false, editable, window, cx))
                            .child(self.render_select_dropdown(
                                "rec-tinh",
                                "Tỉnh / Thành phố cư trú",
                                &ma_tinh,
                                tinh_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("MATINH_CU_TRU".into(), val.clone());
                                    let def_xa = this.get_default_catalog_code("XA_PHUONG", Some(&val));
                                    this.form_values.insert("MAXA_CU_TRU".into(), def_xa);
                                },
                                cx,
                            ))
                            .child(self.render_select_dropdown(
                                "rec-xa",
                                "Phường / Xã cư trú",
                                &ma_xa,
                                xa_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("MAXA_CU_TRU".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("DIA_CHI", "Địa chỉ chi tiết (Số nhà, thôn/tổ)", "Số nhà, đường...", false, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_select_dropdown(
                                "rec-dantoc",
                                "Dân tộc",
                                &ma_dantoc,
                                dantoc_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("MA_DAN_TOC".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_select_dropdown(
                                "rec-nghe",
                                "Nghề nghiệp",
                                &ma_nghe,
                                nghe_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("MA_NGHE_NGHIEP".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("NOI_LAM_VIEC_HOC_TAP", "Nơi làm việc / Học tập", "Công ty / Đơn vị...", false, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_select_dropdown(
                                "rec-loai-kcb",
                                "Loại hình khám (QĐ 3176)",
                                &kcb,
                                kcb_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("MA_LOAI_KCB".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("NGAY_VAO", "Ngày vào khám (YYYYMMDDHHMM)", "", false, editable, window, cx))
                            .child(self.render_select_dropdown(
                                "rec-nhom-mau",
                                "Nhóm máu",
                                &nhom_mau,
                                nhommau_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("NHOM_MAU".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("LY_DO_VV", "Lý do khám", "Khám sức khỏe", false, editable, window, cx)),
                    )
                    .when(!self.is_creating_new && self.selected_record_id.is_some(), |el| {
                        el.child(
                            div()
                                .pt(px(8.0))
                                .border_t_1()
                                .border_color(rgb(BORDER_COL))
                                .flex()
                                .flex_row()
                                .justify_end()
                                .child(
                                    Button::new("btn-print-reception-docx")
                                        .label("In Phiếu Tiếp Nhận (DOCX)")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.export_docx(true, cx);
                                        })),
                                ),
                        )
                    }),
            )
            .into_any_element()
    }

    fn render_tab_history(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_role = self.has_any_role(&["admin", "reception", "tiensu", "bs_noi"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("history");
        let ts_gd = self.get_form_val("TSGD_MAC_BENH");
        let icd_val = self.get_form_val("TSBT_MA_BENH");

        let mut icd_opts = vec![("".to_string(), "-- Chọn nhanh ICD-10 --".to_string())];
        icd_opts.extend(self.query_catalog_options("ICD10", None));

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "history",
                        "Tiền sử bệnh tật (Gia đình & Bản thân - 21 mục)",
                        can_role,
                        is_locked,
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_select_dropdown(
                                "hist-tsgd",
                                "Tiền sử gia đình mắc bệnh?",
                                &ts_gd,
                                vec![
                                    ("0".to_string(), "0 - Không".to_string()),
                                    ("1".to_string(), "1 - Có".to_string()),
                                ],
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("TSGD_MAC_BENH".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("TSGD_MA_BENH", "Mã bệnh gia đình (nếu có)", "", false, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(8.0))
                            .children(TIEN_SU_21_ITEMS.iter().map(|(key, label)| {
                                let checked = self.get_form_val(key) == "1";
                                let k = key.to_string();
                                div()
                                    .w(px(280.0))
                                    .p(px(6.0))
                                    .rounded(px(6.0))
                                    .bg(rgb(if checked { WARN_BG } else { BG_SUBTLE }))
                                    .child(
                                        Checkbox::new(SharedString::from(format!("chk-{}", key)))
                                            .label(*label)
                                            .checked(checked)
                                            .disabled(!editable)
                                            .on_change(cx.listener(move |this, val, _, cx| {
                                                this.form_values.insert(
                                                    k.clone(),
                                                    if *val { "1".into() } else { "0".into() },
                                                );
                                                cx.notify();
                                            })),
                                    )
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("TSBT_MA_BENH_KHAC", "Bệnh khác (nếu có)", "", false, editable, window, cx))
                            .child(self.render_select_dropdown(
                                "hist-icd10",
                                "Chọn nhanh ICD-10 đang điều trị",
                                &icd_val,
                                icd_opts,
                                editable,
                                |this, val, window, cx| {
                                    this.form_values.insert("TSBT_MA_BENH".into(), val.clone());
                                    if let Some(inp) = this.form_inputs.get("TSBT_MA_BENH") {
                                        inp.update(cx, |s, cx| s.set_value(val, window, cx));
                                    }
                                },
                                cx,
                            ))
                            .child(self.render_form_input("TSBT_MA_BENH", "Mã bệnh đang điều trị", "", false, editable, window, cx)),
                    )
                    .child(self.render_form_textarea(
                        "TSBT_TEN_THUOC_LIEU_LUONG",
                        "Thuốc & liều lượng đang dùng",
                        "Ghi chi tiết tên thuốc, liều lượng đang điều trị...",
                        3,
                        editable,
                        window,
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn render_tab_physical(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_role = self.has_any_role(&["admin", "bs_theluc", "bs_noi"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("physical");

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "physical",
                        "Khám Thể lực & Sinh hiệu",
                        can_role,
                        is_locked,
                        cx,
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("CHIEU_CAO", "Chiều cao (cm)", "170", false, editable, window, cx))
                            .child(self.render_form_input("CAN_NANG", "Cân nặng (kg)", "65", false, editable, window, cx))
                            .child(self.render_form_input("CHI_SO_BMI", "Chỉ số BMI (tự tính)", "22.5", false, false, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("MACH", "Mạch (lần/phút)", "75", false, editable, window, cx))
                            .child(self.render_form_input("HUYET_AP", "Huyết áp (mmHg)", "120/80", false, editable, window, cx))
                            .child(self.render_form_input("BS_THE_LUC", "Người khám thể lực", "", false, editable, window, cx)),
                    )
                    .child(self.render_pl_radio_group(
                        "phys",
                        "KHAM_THE_LUC_PL",
                        "Phân loại thể lực",
                        editable,
                        cx,
                    )),
            )
            .into_any_element()
    }

    fn render_tab_clinical(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let subtabs = [
            ("noi", "1. Nội khoa"),
            ("ngoai", "2. Ngoại khoa"),
            ("dalieu", "3. Da liễu"),
            ("san", "4. Sản phụ khoa"),
            ("mat", "5. Mắt"),
            ("tmh", "6. Tai - Mũi - Họng"),
            ("rhm", "7. Răng - Hàm - Mặt"),
            ("tree", "Tra cứu 110+ bệnh"),
        ];

        let active_sub = self.active_clinical_subtab.clone();

        let subtab_bar = div()
            .bg(rgb(BG_CARD))
            .border_1()
            .border_color(rgb(BORDER_COL))
            .rounded(px(8.0))
            .p(px(8.0))
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(px(6.0))
            .children(subtabs.iter().map(|(key, lbl)| {
                let is_act = active_sub == *key;
                let k = key.to_string();
                Button::new(SharedString::from(format!("clin-sub-{}", key)))
                    .label(*lbl)
                    .small()
                    .when(is_act, |b| b.primary())
                    .when(!is_act, |b| b.secondary())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.active_clinical_subtab = k.clone();
                        cx.notify();
                    }))
            }));

        let body = match active_sub.as_str() {
            "ngoai" => {
                let can = self.has_any_role(&["admin", "bs_ngoai"]);
                let ed = !is_locked && can && self.editing_section.as_deref() == Some("clin_ngoai");
                self.render_simple_specialty_card(
                    "clin_ngoai",
                    "Chuyên khoa Ngoại",
                    "KET_QUA_KHAM_NGOAI_KHOA",
                    "KHAM_NGOAI_KHOA_PL",
                    "BS_NGOAI",
                    can,
                    ed,
                    is_locked,
                    window,
                    cx,
                )
            }
            "dalieu" => {
                let can = self.has_any_role(&["admin", "bs_dalieu"]);
                let ed = !is_locked && can && self.editing_section.as_deref() == Some("clin_dalieu");
                self.render_simple_specialty_card(
                    "clin_dalieu",
                    "Chuyên khoa Da liễu",
                    "KET_QUA_KHAM_DA_LIEU",
                    "KHAM_DA_LIEU_PL",
                    "BS_DA_LIEU",
                    can,
                    ed,
                    is_locked,
                    window,
                    cx,
                )
            }
            "san" => {
                let can = self.has_any_role(&["admin", "bs_san"]);
                let ed = !is_locked && can && self.editing_section.as_deref() == Some("clin_san");
                self.render_simple_specialty_card(
                    "clin_san",
                    "Chuyên khoa Sản phụ khoa",
                    "KET_QUA_KHAM_SAN_PHU_KHOA",
                    "KHAM_SAN_PHU_KHOA_PL",
                    "BS_SAN",
                    can,
                    ed,
                    is_locked,
                    window,
                    cx,
                )
            }
            "mat" => {
                let can_mat = self.has_any_role(&["admin", "bs_mat"]);
                let edit_mat = !is_locked && can_mat && self.editing_section.as_deref() == Some("clin_mat");
                card_box()
                    .child(self.render_section_action_bar("clin_mat", "Chuyên khoa Mắt", can_mat, is_locked, cx))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("KHONG_KINH_MAT_PHAI", "Thị lực không kính MP", "10/10", false, edit_mat, window, cx))
                            .child(self.render_form_input("KHONG_KINH_MAT_TRAI", "Thị lực không kính MT", "10/10", false, edit_mat, window, cx))
                            .child(self.render_form_input("CO_KINH_MAT_PHAI", "Thị lực có kính MP", "10/10", false, edit_mat, window, cx))
                            .child(self.render_form_input("CO_KINH_MAT_TRAI", "Thị lực có kính MT", "10/10", false, edit_mat, window, cx)),
                    )
                    .child(self.render_form_textarea("BENH_KHAC_MAT", "Kết quả khám & Các bệnh về mắt", "Bình thường / Không có bệnh lý...", 3, edit_mat, window, cx))
                    .child(self.render_pl_radio_group("mat", "KHAM_MAT_PL", "Phân loại Mắt", edit_mat, cx))
                    .child(self.render_form_input("BS_MAT", "Bác sĩ khám Mắt", "", false, edit_mat, window, cx))
                    .into_any_element()
            }
            "tmh" => {
                let can_tmh = self.has_any_role(&["admin", "bs_tmh"]);
                let edit_tmh = !is_locked && can_tmh && self.editing_section.as_deref() == Some("clin_tmh");
                card_box()
                    .child(self.render_section_action_bar("clin_tmh", "Chuyên khoa Tai - Mũi - Họng", can_tmh, is_locked, cx))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("TAI_PHAI_NOI_THUONG", "Tai phải nói thường (m)", "5m", false, edit_tmh, window, cx))
                            .child(self.render_form_input("TAI_PHAI_NOI_THAM", "Tai phải nói thầm (m)", "0.5m", false, edit_tmh, window, cx))
                            .child(self.render_form_input("TAI_TRAI_NOI_THUONG", "Tai trái nói thường (m)", "5m", false, edit_tmh, window, cx))
                            .child(self.render_form_input("TAI_TRAI_NOI_THAM", "Tai trái nói thầm (m)", "0.5m", false, edit_tmh, window, cx)),
                    )
                    .child(self.render_form_textarea("BENH_KHAC_TAI_MUI_HONG", "Kết quả khám & Bệnh Tai - Mũi - Họng", "Bình thường...", 3, edit_tmh, window, cx))
                    .child(self.render_pl_radio_group("tmh", "KHAM_TAI_MUI_HONG_PL", "Phân loại TMH", edit_tmh, cx))
                    .child(self.render_form_input("BS_TMH", "Bác sĩ khám TMH", "", false, edit_tmh, window, cx))
                    .into_any_element()
            }
            "rhm" => {
                let can_rhm = self.has_any_role(&["admin", "bs_rhm"]);
                let edit_rhm = !is_locked && can_rhm && self.editing_section.as_deref() == Some("clin_rhm");
                card_box()
                    .child(self.render_section_action_bar("clin_rhm", "Chuyên khoa Răng - Hàm - Mặt", can_rhm, is_locked, cx))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_textarea("HAM_TREN", "Hàm trên", "Bình thường", 2, edit_rhm, window, cx))
                            .child(self.render_form_textarea("HAM_DUOI", "Hàm dưới", "Bình thường", 2, edit_rhm, window, cx)),
                    )
                    .child(self.render_form_textarea("BENH_KHAC_RANG_HAM_MAT", "Kết quả khám & Bệnh Răng - Hàm - Mặt", "Bình thường...", 3, edit_rhm, window, cx))
                    .child(self.render_pl_radio_group("rhm", "KHAM_RANG_HAM_MAT_PL", "Phân loại RHM", edit_rhm, cx))
                    .child(self.render_form_input("BS_RHM", "Bác sĩ khám RHM", "", false, edit_rhm, window, cx))
                    .into_any_element()
            }
            "tree" => self.render_disease_classification_tree(window, cx),
            _ => {
                let can_noi = self.has_any_role(&["admin", "bs_noi"]);
                let edit_noi = !is_locked && can_noi && self.editing_section.as_deref() == Some("clin_noi");
                let noi_fields = [
                    ("NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "Tuần hoàn"),
                    ("NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL", "Hô hấp"),
                    ("NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "Tiêu hóa"),
                    ("NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TIETNIEU_PL", "Thận - Tiết niệu"),
                    ("NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "Nội tiết"),
                    ("NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL", "Cơ xương khớp"),
                    ("NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "Thần kinh"),
                    ("NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL", "Tâm thần"),
                ];
                let mut box_noi = card_box().child(self.render_section_action_bar(
                    "clin_noi",
                    "Chuyên khoa Nội & Tâm thần",
                    can_noi,
                    is_locked,
                    cx,
                ));
                for (f_kq, f_pl, lbl) in noi_fields {
                    box_noi = box_noi.child(
                        div()
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .bg(rgb(BG_SUBTLE))
                            .border_1()
                            .border_color(rgb(BORDER_COL))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(self.render_form_textarea(f_kq, lbl, "Bình thường...", 2, edit_noi, window, cx))
                            .child(self.render_pl_radio_group(f_pl, f_pl, "Phân loại", edit_noi, cx)),
                    );
                }
                box_noi
                    .child(self.render_form_input("BS_NOI", "Bác sĩ Nội khoa", "", false, edit_noi, window, cx))
                    .into_any_element()
            }
        };

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(subtab_bar)
            .child(body)
            .into_any_element()
    }

    #[allow(clippy::too_many_arguments)]
    fn render_simple_specialty_card(
        &mut self,
        sec_key: &str,
        title: &str,
        f_kq: &str,
        f_pl: &str,
        f_bs: &str,
        can_role: bool,
        editable: bool,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        card_box()
            .child(self.render_section_action_bar(sec_key, title, can_role, is_locked, cx))
            .child(self.render_form_textarea(
                f_kq,
                "Kết quả khám chuyên khoa",
                "Nhập chi tiết kết quả khám...",
                4,
                editable,
                window,
                cx,
            ))
            .child(self.render_pl_radio_group(f_pl, f_pl, "Phân loại sức khỏe chuyên khoa", editable, cx))
            .child(self.render_form_input(f_bs, "Bác sĩ khám", "", false, editable, window, cx))
            .into_any_element()
    }

    fn render_disease_classification_tree(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let search_q = self.disease_search_input.read(cx).value().to_lowercase();
        let sel_spec = self.selected_specialty_filter.clone();

        let db_specs = self.query_catalog_items("CHUYEN_KHOA", None);
        let db_diseases = self.query_catalog_items("BENH_TAT", None);
        let db_classes = self.query_catalog_items("TIEU_CHUAN_PL", None);

        let use_db = !db_diseases.is_empty();
        let cfg = phan_loai_config();

        let mut spec_opts = vec![("".to_string(), "Tất cả chuyên khoa".to_string())];
        if use_db && !db_specs.is_empty() {
            for s in &db_specs {
                spec_opts.push((s.code.clone(), s.name.clone()));
            }
        } else {
            for s in &cfg.specialties {
                spec_opts.push((s.id.clone(), s.name.clone()));
            }
        }

        // Tuples of (disease_id, disease_name, Vec<(cls_id, label, append_text)>)
        let matching_items: Vec<(String, String, Vec<(String, String, String)>)> = if use_db {
            db_diseases
                .iter()
                .filter(|d| {
                    (sel_spec.is_empty() || d.parent_code == sel_spec)
                        && (search_q.is_empty()
                            || d.name.to_lowercase().contains(&search_q)
                            || d.code.to_lowercase().contains(&search_q))
                })
                .take(50)
                .map(|d| {
                    let classes = db_classes
                        .iter()
                        .filter(|c| c.parent_code == d.code)
                        .map(|c| {
                            let label = c.name.clone();
                            let append_text = format!("{} ({})", d.name, c.name);
                            (format!("db-{}", c.id), label, append_text)
                        })
                        .collect();
                    (d.code.clone(), d.name.clone(), classes)
                })
                .collect()
        } else {
            cfg.diseases
                .iter()
                .filter(|d| {
                    (sel_spec.is_empty() || d.specialty_id == sel_spec)
                        && (search_q.is_empty() || d.name.to_lowercase().contains(&search_q))
                })
                .take(50)
                .map(|d| {
                    let classes = cfg
                        .classifications
                        .iter()
                        .filter(|c| c.disease_id == d.id)
                        .map(|c| {
                            let score_str = c
                                .score
                                .map(|s| format!("Loại {}", s))
                                .unwrap_or_else(|| "Ghi chú".into());
                            let label = format!("{}: {}", score_str, c.name);
                            let append_text = format!("{} ({})", d.name, score_str);
                            (c.id.clone(), label, append_text)
                        })
                        .collect();
                    (d.id.clone(), d.name.clone(), classes)
                })
                .collect()
        };

        card_box()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.0))
                    .child(div().flex_1().child(Input::new(&self.disease_search_input).small().cleanable(true)))
                    .child(self.render_select_dropdown(
                        "tree-spec-filter",
                        "",
                        &sel_spec,
                        spec_opts,
                        true,
                        |this, val, _, _| {
                            this.selected_specialty_filter = val;
                        },
                        cx,
                    )),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(TEXT_MUTED))
                    .child(format!(
                        "Hiển thị {} mặt bệnh (tối đa 50 dòng/lượt lọc). Nhấn vào mức phân loại để ghi vào kết luận bệnh:",
                        matching_items.len()
                    )),
            )
            .children(matching_items.into_iter().map(|(_did, d_name, classes)| {
                div()
                    .p(px(10.0))
                    .rounded(px(6.0))
                    .bg(rgb(BG_SUBTLE))
                    .border_1()
                    .border_color(rgb(BORDER_COL))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(TEXT_MAIN))
                            .child(d_name),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(6.0))
                            .children(classes.into_iter().map(|(cid, label, append_text)| {
                                Button::new(SharedString::from(format!("cls-btn-{}", cid)))
                                    .label(label)
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let cur = this.get_form_val("CAC_BENH_TAT_NEU_CO");
                                        let next = if cur.is_empty() {
                                            append_text.clone()
                                        } else {
                                            format!("{}; {}", cur, append_text)
                                        };
                                        this.form_values.insert("CAC_BENH_TAT_NEU_CO".into(), next.clone());
                                        if let Some(inp) = this.form_inputs.get("CAC_BENH_TAT_NEU_CO") {
                                            inp.update(cx, |s, cx| s.set_value(next.clone(), window, cx));
                                        }
                                        if let Some(ta) = this.form_textareas.get("CAC_BENH_TAT_NEU_CO") {
                                            ta.update(cx, |s, cx| s.set_value(next, window, cx));
                                        }
                                        this.show_toast(&format!("Đã thêm: {}", append_text), false, cx);
                                    }))
                            })),
                    )
            }))
            .into_any_element()
    }

    fn render_tab_paraclinical(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_role = self.has_any_role(&["admin", "bs_cls", "bs_ketluan"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("paraclinical");

        card_box()
            .child(self.render_section_action_bar(
                "paraclinical",
                "Kết quả Cận lâm sàng (Xét nghiệm & CĐHA)",
                can_role,
                is_locked,
                cx,
            ))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(self.render_form_input("CLS_DUONG_MAU", "Đường máu (mmol/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_URE", "Urê (mmol/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_CREATININ", "Creatinin (µmol/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_ASAT_GOT", "AST / GOT (U/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_ALAT_GPT", "ALT / GPT (U/L)", "", false, editable, window, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(self.render_form_textarea("CLS_HUYET_HOC", "Công thức máu (Huyết học)", "Bình thường...", 3, editable, window, cx))
                    .child(self.render_form_textarea("CLS_NUOC_TIEU", "Tổng phân tích nước tiểu", "Bình thường...", 3, editable, window, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(self.render_form_textarea("CLS_XQ_TIM_PHOI", "X-Quang tim phổi", "Bình thường...", 3, editable, window, cx))
                    .child(self.render_form_textarea("CLS_KHAC_MO_TA", "Cận lâm sàng khác (Siêu âm, ECG...)", "Nhập kết quả siêu âm, điện tim...", 3, editable, window, cx)),
            )
            .child(self.render_form_textarea(
                "CLS_DINH_KY_KET_QUA",
                "Kết luận Cận lâm sàng",
                "Bình thường...",
                3,
                editable,
                window,
                cx,
            ))
            .child(self.render_form_input("BS_CLS", "Bác sĩ CLS", "", false, editable, window, cx))
            .into_any_element()
    }

    fn render_tab_conclusion(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_role = self.has_any_role(&["admin", "bs_ketluan"]);
        let is_admin = self.has_any_role(&["admin"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("conclusion");

        card_box()
            .child(self.render_section_action_bar(
                "conclusion",
                "Kết luận Sức khỏe & Ký khóa hồ sơ",
                can_role,
                is_locked,
                cx,
            ))
            .child(self.render_pl_radio_group(
                "conc",
                "PHAN_LOAI_SK",
                "Phân loại sức khỏe chung",
                editable,
                cx,
            ))
            .child(self.render_form_textarea(
                "KET_LUAN_BENH",
                "Kết luận đủ sức khỏe / Bệnh tật",
                "Đủ sức khỏe làm việc / học tập...",
                3,
                editable,
                window,
                cx,
            ))
            .child(self.render_form_textarea(
                "CAC_BENH_TAT_NEU_CO",
                "Các bệnh tật nếu có",
                "Ghi rõ tên bệnh / mã phân loại nếu có...",
                3,
                editable,
                window,
                cx,
            ))
            .child(self.render_form_input("BS_KET_LUAN", "Bác sĩ kết luận", "", false, editable, window, cx))
            .child(
                div()
                    .pt(px(10.0))
                    .border_t_1()
                    .border_color(rgb(BORDER_COL))
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(10.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .when(!is_locked && can_role && self.selected_record_id.is_some(), |el| {
                                el.child(
                                    Button::new("btn-sign-lock")
                                        .label("Ký khóa hồ sơ (Hoàn tất)")
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.set_record_status(2, window, cx);
                                        })),
                                )
                            })
                            .when(is_locked && is_admin && self.selected_record_id.is_some(), |el| {
                                el.child(
                                    Button::new("btn-unlock-admin")
                                        .label("Mở khóa hồ sơ (Admin)")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.set_record_status(1, window, cx);
                                        })),
                                )
                            }),
                    )
                    .when(self.selected_record_id.is_some(), |el| {
                        el.child(
                            Button::new("btn-export-docx")
                                .label("Xuất Giấy KSK (DOCX)")
                                .small()
                                .secondary()
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.export_docx(false, cx);
                                })),
                        )
                    }),
            )
            .into_any_element()
    }
}
