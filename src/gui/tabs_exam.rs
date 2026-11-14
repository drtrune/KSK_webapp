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
        bg_card, bg_subtle, border_col, DANGER_BG, DANGER_COL, PRIMARY_COL, primary_light,
        SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, badge, card_box,
    },
};

const TIEN_SU_21_ITEMS: &[(&str, &str)] = &[
    ("TSBT_BENH_TRONG_5_NAM_QUA", "1. Có bệnh hay bị thương trong 5 năm qua"),
    ("TSBT_BENH_THAN_KINH", "2. Có bệnh thần kinh hay bị thương ở đầu"),
    ("TSBT_BENH_MAT", "3. Bệnh mắt hoặc giảm thị lực (trừ đeo kính thuốc)"),
    ("TSBT_BENH_TAI", "4. Bệnh ở tai, giảm sức nghe hoặc thăng bằng"),
    ("TSBT_BENH_TIM", "5. Bệnh ở tim, hoặc nhồi máu cơ tim, bệnh tim mạch khác"),
    ("TSBT_PHAU_THUAT_TIM", "6. Phẫu thuật can thiệp tim - mạch (thay van, bắc cầu nối, tạo hình mạch, máy tạo nhịp, đặt stent mạch, ghép tim)"),
    ("TSBT_TANG_HUYET_AP", "7. Tăng huyết áp"),
    ("TSBT_KHO_THO", "8. Khó thở"),
    ("TSBT_BENH_PHOI", "9. Bệnh phổi, hen, khí phế thũng, viêm phế quản mạn tính"),
    ("TSBT_BENH_THAN", "10. Bệnh thận, lọc máu"),
    ("TSBT_NGHIEN_RUOU", "11. Nghiện rượu, bia"),
    ("TSBT_DAI_THAO_DUONG", "12. Đái tháo đường hoặc kiểm soát tăng đường huyết"),
    ("TSBT_BENH_TAM_THAN", "13. Bệnh tâm thần"),
    ("TSBT_MAT_Y_THUC", "14. Mất ý thức, rối loạn ý thức"),
    ("TSBT_NGAT", "15. Ngất, chóng mặt"),
    ("TSBT_BENH_TIEU_HOA", "16. Bệnh tiêu hóa"),
    ("TSBT_ROI_LOAN_GIAC_NGU", "17. Rối loạn giấc ngủ, ngừng thở khi ngủ, ngủ rũ ban ngày, ngáy to"),
    ("TSBT_TAI_BIEN", "18. Tai biến mạch máu não hoặc liệt"),
    ("TSBT_BENH_COT_SONG", "19. Bệnh hoặc tổn thương cột sống"),
    ("TSBT_RUOU_THUONG_XUYEN", "20. Sử dụng rượu thường xuyên, liên tục"),
    ("TSBT_MA_TUY", "21. Sử dụng ma túy và chất gây nghiện"),
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
                                .text_color(rgb(text_main()))
                                .child("Chưa chọn hồ sơ bệnh nhân"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(text_muted()))
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
                    .bg(rgb(bg_card()))
                    .border_b_1()
                    .border_color(rgb(border_col()))
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
                                            .text_color(rgb(text_main()))
                                            .child(if self.is_creating_new {
                                                "Hồ sơ tiếp nhận mới".to_string()
                                            } else if ho_ten.is_empty() {
                                                "(Chưa có họ tên)".to_string()
                                            } else {
                                                ho_ten
                                            }),
                                    )
                                    .when(!ma_bn.is_empty(), |el| {
                                        el.child(badge(format!("Mã BN: {}", ma_bn), bg_subtle(), text_main()))
                                    })
                                    .when(!ma_tn.is_empty(), |el| {
                                        el.child(badge(format!("Số TN: {}", ma_tn), primary_light(), PRIMARY_COL))
                                    })
                                    .child(if self.current_record_exam_mode == "doan" {
                                        let label = self
                                            .current_record_contract_id
                                            .and_then(|cid| self.contracts_list.iter().find(|c| c.id == cid))
                                            .map(|c| format!("Đoàn: {} ({})", c.company_name, c.contract_code))
                                            .unwrap_or_else(|| "Khám đoàn".to_string());
                                        badge(label, WARN_BG, WARN_COL)
                                    } else {
                                        badge("Khám lẻ", primary_light(), PRIMARY_COL)
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
                                    .border_color(rgb(if is_act { PRIMARY_COL } else { bg_card() }))
                                    .text_sm()
                                    .font_weight(if is_act {
                                        gpui::FontWeight::SEMIBOLD
                                    } else {
                                        gpui::FontWeight::NORMAL
                                    })
                                    .text_color(rgb(if is_act { PRIMARY_COL } else { text_muted() }))
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
            ("1".to_string(), "Nam".to_string()),
            ("2".to_string(), "Nữ".to_string()),
            ("3".to_string(), "Khác".to_string()),
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
                ("1".to_string(), "Khám sức khỏe định kỳ".to_string()),
                ("2".to_string(), "Khám sức khỏe tuyển dụng".to_string()),
                ("3".to_string(), "Khám sức khỏe lái xe".to_string()),
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
                            .child(self.render_date_picker_form("rec-ngay-sinh", "NGAY_SINH", "Ngày sinh (dd/mm/yyyy)", "01/01/1990", true, editable, false, window, cx)),
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
                            .child(self.render_date_picker_form("rec-ngaycap-cccd", "NGAYCAP_CCCD", "Ngày cấp CCCD (dd/mm/yyyy)", "01/05/2021", false, editable, false, window, cx))
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
                            .child(self.render_date_picker_form("rec-ngay-vao", "NGAY_VAO", "Ngày vào khám (dd/mm/yyyy)", "dd/mm/yyyy", false, editable, false, window, cx))
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
                                .border_color(rgb(border_col()))
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
        let ts_benh_khac = self.get_form_val("TSBT_BENH_KHAC");
        let ts_dang_dieu_tri = self.get_form_val("TSBT_DANG_DIEU_TRI_BENH");
        let gioi_tinh = self.get_form_val("GIOI_TINH");
        let ts_thai_san = self.get_form_val("TSBT_THAI_SAN");

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "history",
                        "A. Tiền sử gia đình",
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
                                "Có ai trong gia đình mắc bệnh không?",
                                &ts_gd,
                                vec![
                                    ("0".to_string(), "Không".to_string()),
                                    ("1".to_string(), "Có".to_string()),
                                ],
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("TSGD_MAC_BENH".into(), val);
                                },
                                cx,
                            ))
                            .when(ts_gd == "1", |el| {
                                el.child(self.render_icd_multi_select(
                                    "hist-tsgd-icd",
                                    "TSGD_MA_BENH",
                                    "Mã bệnh tiền sử gia đình (ICD-10 - chọn nhiều)",
                                    editable,
                                    cx,
                                ))
                            }),
                    ),
            )
            .child(
                card_box()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(text_main()))
                            .child("B. Tiền sử bản thân (22 bệnh lý)"),
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
                                    .w(px(380.0))
                                    .p(px(6.0))
                                    .rounded(px(6.0))
                                    .bg(rgb(if checked { WARN_BG } else { bg_subtle() }))
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
                            }))
                            .child({
                                let checked = ts_benh_khac == "1";
                                div()
                                    .w(px(380.0))
                                    .p(px(6.0))
                                    .rounded(px(6.0))
                                    .bg(rgb(if checked { WARN_BG } else { bg_subtle() }))
                                    .child(
                                        Checkbox::new("chk-TSBT_BENH_KHAC")
                                            .label("22. Bệnh khác (ghi rõ)")
                                            .checked(checked)
                                            .disabled(!editable)
                                            .on_change(cx.listener(|this, val, _, cx| {
                                                this.form_values.insert(
                                                    "TSBT_BENH_KHAC".into(),
                                                    if *val { "1".into() } else { "0".into() },
                                                );
                                                cx.notify();
                                            })),
                                    )
                            }),
                    )
                    .when(ts_benh_khac == "1", |el| {
                        el.child(self.render_icd_multi_select(
                            "hist-tsbt-khac-icd",
                            "TSBT_MA_BENH_KHAC",
                            "Mã bệnh khác (ICD-10 - chọn nhiều)",
                            editable,
                            cx,
                        ))
                    }),
            )
            .child(
                card_box()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(text_main()))
                            .child("C. Hiện tại có đang điều trị bệnh nào không?"),
                    )
                    .child(self.render_select_dropdown(
                        "hist-dang-dieu-tri",
                        "Đang điều trị bệnh",
                        &ts_dang_dieu_tri,
                        vec![
                            ("0".to_string(), "Không".to_string()),
                            ("1".to_string(), "Có".to_string()),
                        ],
                        editable,
                        |this, val, _, _| {
                            this.form_values.insert("TSBT_DANG_DIEU_TRI_BENH".into(), val);
                        },
                        cx,
                    ))
                    .when(ts_dang_dieu_tri == "1", |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_row()
                                .flex_wrap()
                                .gap(px(12.0))
                                .child(self.render_icd_multi_select(
                                    "hist-icd10",
                                    "TSBT_MA_BENH",
                                    "Mã bệnh đang điều trị (ICD-10 - chọn nhiều)",
                                    editable,
                                    cx,
                                )),
                        )
                        .child(self.render_form_textarea(
                            "TSBT_TEN_THUOC_LIEU_LUONG",
                            "Thuốc & liều lượng đang dùng",
                            "Ghi chi tiết tên thuốc, liều lượng đang điều trị...",
                            3,
                            editable,
                            window,
                            cx,
                        ))
                    }),
            )
            .when(gioi_tinh == "2", |el| {
                el.child(
                    card_box()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(text_main()))
                                        .child("D. Tiền sử sản khoa (Chỉ áp dụng đối với Nữ)"),
                                )
                                .child(badge("Bệnh nhân Nữ", primary_light(), PRIMARY_COL)),
                        )
                        .child(self.render_select_dropdown(
                            "hist-thai-san",
                            "Tiền sử thai sản",
                            &ts_thai_san,
                            vec![
                                ("0".to_string(), "0 - Không".to_string()),
                                ("1".to_string(), "1 - Có".to_string()),
                            ],
                            editable,
                            |this, val, _, _| {
                                this.form_values.insert("TSBT_THAI_SAN".into(), val);
                            },
                            cx,
                        ))
                        .when(ts_thai_san == "1", |sub| {
                            sub.child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap(px(12.0))
                                    .child(self.render_icd_multi_select(
                                        "hist-icd10-thaisan",
                                        "TSBT_MA_BENH_THAI_SAN",
                                        "Mã bệnh / Tình trạng thai sản (ICD-10 - chọn nhiều)",
                                        editable,
                                        cx,
                                    ))
                                    .child(self.render_form_input(
                                        "TSBT_TEN_THUOC_THAI_SAN",
                                        "Thuốc điều trị thai sản (nếu có)",
                                        "Tên thuốc...",
                                        false,
                                        editable,
                                        window,
                                        cx,
                                    )),
                            )
                        }),
                )
            })
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
            .bg(rgb(bg_card()))
            .border_1()
            .border_color(rgb(border_col()))
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
                    "CK_NGOAI",
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
                    "CK_DA_LIEU",
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
                    "CK_PHU_SAN",
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
                    .when(self.clinical_catalog_mode, |el| {
                        el.child(self.render_specialty_classification_block("CK_MAT", "Chuyên khoa Mắt", edit_mat, window, cx))
                    })
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
                    .when(self.clinical_catalog_mode, |el| {
                        el.child(self.render_specialty_classification_block("CK_TMH", "Tai - Mũi - Họng", edit_tmh, window, cx))
                    })
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
                    .when(self.clinical_catalog_mode, |el| {
                        el.child(self.render_specialty_classification_block("CK_RHM", "Răng - Hàm - Mặt", edit_rhm, window, cx))
                    })
                    .into_any_element()
            }
            "tree" => self.render_disease_classification_tree(window, cx),
            _ => {
                let can_noi = self.has_any_role(&["admin", "bs_noi"]);
                let edit_noi = !is_locked && can_noi && self.editing_section.as_deref() == Some("clin_noi");
                let noi_specs = [
                    ("CK_NOI_TIM_MACH", "NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "Tuần hoàn"),
                    ("CK_NOI_HO_HAP", "NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL", "Hô hấp"),
                    ("CK_NOI_TIEU_HOA", "NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "Tiêu hóa"),
                    ("CK_NOI_TIET_NIEU_SD", "NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TIETNIEU_PL", "Thận - Tiết niệu"),
                    ("CK_NOI_NOITIET", "NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "Nội tiết"),
                    ("CK_NOI_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL", "Cơ xương khớp"),
                    ("CK_NOI_THAN_KINH", "NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "Thần kinh"),
                    ("CK_NOI_TAM_THAN", "NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL", "Tâm thần"),
                ];
                let active_noi = self.active_noi_subtab.clone();
                let current_noi = noi_specs
                    .iter()
                    .find(|(sp, _, _, _)| *sp == active_noi.as_str())
                    .copied()
                    .unwrap_or(noi_specs[0]);

                let noi_subtab_bar = div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(6.0))
                    .children(noi_specs.iter().map(|(sp, _, f_pl, lbl)| {
                        let is_act = current_noi.0 == *sp;
                        let sp_own = sp.to_string();
                        let cur_pl = self.get_form_val(f_pl);
                        let label = if cur_pl.is_empty() || cur_pl == "1" {
                            lbl.to_string()
                        } else {
                            format!("{} (L{})", lbl, cur_pl)
                        };
                        Button::new(SharedString::from(format!("noi-sub-{}", sp)))
                            .label(label)
                            .small()
                            .when(is_act, |b| b.primary())
                            .when(!is_act, |b| b.secondary())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.active_noi_subtab = sp_own.clone();
                                cx.notify();
                            }))
                    }));

                card_box()
                    .child(self.render_section_action_bar(
                        "clin_noi",
                        "Chuyên khoa Nội & Tâm thần",
                        can_noi,
                        is_locked,
                        cx,
                    ))
                    .child(noi_subtab_bar)
                    .child(self.render_form_textarea(
                        current_noi.1,
                        &format!("Kết quả {}", current_noi.3),
                        "Bình thường...",
                        3,
                        edit_noi,
                        window,
                        cx,
                    ))
                    .child(self.render_pl_radio_group(
                        current_noi.2,
                        current_noi.2,
                        &format!("Phân loại {}", current_noi.3),
                        edit_noi,
                        cx,
                    ))
                    .child(self.render_form_input("BS_NOI", "Bác sĩ Nội khoa", "", false, edit_noi, window, cx))
                    .when(self.clinical_catalog_mode, |el| {
                        el.child(self.render_specialty_classification_block(
                            current_noi.0,
                            current_noi.3,
                            edit_noi,
                            window,
                            cx,
                        ))
                    })
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
        spec_id: &str,
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
                "Bình thường...",
                3,
                editable,
                window,
                cx,
            ))
            .child(self.render_pl_radio_group(f_pl, f_pl, "Phân loại sức khỏe chuyên khoa", editable, cx))
            .child(self.render_form_input(f_bs, "Bác sĩ khám", "", false, editable, window, cx))
            .when(self.clinical_catalog_mode, |el| {
                el.child(self.render_specialty_classification_block(spec_id, title, editable, window, cx))
            })
            .into_any_element()
    }

    fn auto_scores_from_form(&self) -> std::collections::HashMap<String, (String, i64)> {
        use crate::models::{
            auto_classify_bmi, auto_classify_bp, auto_classify_hearing, auto_classify_pulse,
            auto_classify_vision, calc_patient_age,
        };
        let mut map = std::collections::HashMap::new();
        let (v_id, v_sc) = auto_classify_vision(
            &self.get_form_val("KHONG_KINH_MAT_PHAI"),
            &self.get_form_val("KHONG_KINH_MAT_TRAI"),
            &self.get_form_val("CO_KINH_MAT_PHAI"),
            &self.get_form_val("CO_KINH_MAT_TRAI"),
        );
        map.insert("BT_MAT_THI_LUC".to_string(), (v_id.to_string(), v_sc));

        let (h_id, h_sc) = auto_classify_hearing(
            &self.get_form_val("TAI_PHAI_NOI_THUONG"),
            &self.get_form_val("TAI_PHAI_NOI_THAM"),
            &self.get_form_val("TAI_TRAI_NOI_THUONG"),
            &self.get_form_val("TAI_TRAI_NOI_THAM"),
        );
        map.insert("BT_TMH_TAI_TRONG".to_string(), (h_id.to_string(), h_sc));

        let age = calc_patient_age(&self.get_form_val("NGAY_SINH"));
        let (bp_id, bp_sc) = auto_classify_bp(&self.get_form_val("HUYET_AP"), age);
        map.insert("BT_TM_HUYET_AP".to_string(), (bp_id.to_string(), bp_sc));

        let (p_id, p_sc) = auto_classify_pulse(&self.get_form_val("MACH"));
        map.insert("BT_TM_MACH".to_string(), (p_id.to_string(), p_sc));

        let (b_id, b_sc) = auto_classify_bmi(
            &self.get_form_val("CHI_SO_BMI"),
            &self.get_form_val("CHIEU_CAO"),
            &self.get_form_val("CAN_NANG"),
        );
        map.insert("BT_NT_BEO_PHE".to_string(), (b_id.to_string(), b_sc));
        map
    }

    fn auto_disease_detail(&self, disease_id: &str) -> Option<String> {
        match disease_id {
            "BT_MAT_THI_LUC" => Some(format!(
                "Không kính: MP {} — MT {} | Có kính: MP {} — MT {}",
                self.get_form_val("KHONG_KINH_MAT_PHAI"),
                self.get_form_val("KHONG_KINH_MAT_TRAI"),
                self.get_form_val("CO_KINH_MAT_PHAI"),
                self.get_form_val("CO_KINH_MAT_TRAI"),
            )),
            "BT_TMH_TAI_TRONG" => Some(format!(
                "Tai phải: {}m (thầm {}m) | Tai trái: {}m (thầm {}m)",
                self.get_form_val("TAI_PHAI_NOI_THUONG"),
                self.get_form_val("TAI_PHAI_NOI_THAM"),
                self.get_form_val("TAI_TRAI_NOI_THUONG"),
                self.get_form_val("TAI_TRAI_NOI_THAM"),
            )),
            "BT_TM_HUYET_AP" => {
                let bp = self.get_form_val("HUYET_AP");
                let age = crate::models::calc_patient_age(&self.get_form_val("NGAY_SINH"));
                Some(format!(
                    "Huyết áp đo: {} mmHg{}",
                    if bp.is_empty() { "Chưa đo" } else { &bp },
                    age.map(|a| format!(" ({} tuổi)", a)).unwrap_or_default()
                ))
            }
            "BT_TM_MACH" => {
                let m = self.get_form_val("MACH");
                Some(format!(
                    "Tần số mạch: {} lần/phút",
                    if m.is_empty() { "Chưa đo" } else { &m }
                ))
            }
            "BT_NT_BEO_PHE" => {
                let bmi = self.get_form_val("CHI_SO_BMI");
                Some(format!(
                    "Chỉ số BMI: {}",
                    if bmi.is_empty() { "Chưa tính" } else { &bmi }
                ))
            }
            _ => None,
        }
    }

    fn parse_cls_score(name: &str, fallback: Option<i64>) -> i64 {
        if let Some(sc) = fallback {
            return sc;
        }
        for s in 1..=6 {
            if name.contains(&format!("Loại {}", s)) || name.contains(&format!("loại {}", s)) {
                return s;
            }
        }
        1
    }

    fn score_badge_colors(score: i64) -> (u32, u32) {
        match score {
            1 => (SUCCESS_BG, SUCCESS_COL),
            2 | 3 => (WARN_BG, WARN_COL),
            _ => (DANGER_BG, DANGER_COL),
        }
    }

    /// Returns `(disease_code, disease_name, Vec<(cls_code, cls_name, resolved_score, ref_rule, is_ref)>)`
    fn load_specialty_diseases(
        &self,
        spec_id: &str,
        auto_scores: &std::collections::HashMap<String, (String, i64)>,
    ) -> Vec<(String, String, Vec<(String, String, i64, Option<String>, bool)>)> {
        let cfg = phan_loai_config();
        let db_diseases = self.query_catalog_items("BENH_TAT", Some(spec_id));
        let db_classes = self.query_catalog_items("TIEU_CHUAN_PL", None);

        if !db_diseases.is_empty() {
            db_diseases
                .into_iter()
                .filter(|d| d.parent_code == spec_id)
                .map(|d| {
                    let mut classes: Vec<(String, String, i64, Option<String>, bool)> = db_classes
                        .iter()
                        .filter(|c| c.parent_code == d.code)
                        .map(|c| {
                            let cfg_cls = cfg.classifications.iter().find(|x| x.id == c.code);
                            let is_ref = cfg_cls.and_then(|x| x.ref_disease_id.as_ref()).is_some();
                            let ref_rule = cfg_cls.and_then(|x| x.ref_rule.clone());
                            let sc = if cfg_cls.is_some() {
                                crate::models::resolve_classification_score(
                                    &c.code,
                                    &self.selected_classifications,
                                    auto_scores,
                                    0,
                                )
                            } else {
                                Self::parse_cls_score(&c.name, None)
                            };
                            (c.code.clone(), c.name.clone(), sc, ref_rule, is_ref)
                        })
                        .collect();
                    classes.sort_by_key(|c| c.2);
                    (d.code, d.name, classes)
                })
                .collect()
        } else {
            cfg.diseases
                .iter()
                .filter(|d| d.specialty_id == spec_id)
                .map(|d| {
                    let mut classes: Vec<(String, String, i64, Option<String>, bool)> = cfg
                        .classifications
                        .iter()
                        .filter(|c| c.disease_id == d.id)
                        .map(|c| {
                            let is_ref = c.ref_disease_id.is_some();
                            let sc = crate::models::resolve_classification_score(
                                &c.id,
                                &self.selected_classifications,
                                auto_scores,
                                0,
                            );
                            (c.id.clone(), c.name.clone(), sc, c.ref_rule.clone(), is_ref)
                        })
                        .collect();
                    classes.sort_by_key(|c| c.2);
                    (d.id.clone(), d.name.clone(), classes)
                })
                .collect()
        }
    }

    fn sync_specialty_from_classifications(
        &mut self,
        spec_id: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let auto_scores = self.auto_scores_from_form();
        for (did, (cid, _)) in &auto_scores {
            self.selected_classifications.insert(did.clone(), cid.clone());
        }
        let diseases = self.load_specialty_diseases(spec_id, &auto_scores);
        let mut max_score: i64 = 1;
        let mut abn_names: Vec<String> = Vec::new();

        for (d_code, d_name, classes) in &diseases {
            let sel_cls = if let Some((auto_cid, auto_sc)) = auto_scores.get(d_code) {
                classes
                    .iter()
                    .find(|(c, _, _, _, _)| c == auto_cid)
                    .map(|(_, n, _, _, _)| (n.clone(), *auto_sc))
                    .or_else(|| Some((d_name.clone(), *auto_sc)))
            } else {
                let sel_id = self.selected_classifications.get(d_code);
                sel_id
                    .and_then(|cid| classes.iter().find(|(c, _, _, _, _)| c == cid))
                    .or_else(|| classes.iter().find(|(_, _, sc, _, _)| *sc == 1))
                    .or_else(|| classes.first())
                    .map(|(_, n, sc, _, _)| (n.clone(), *sc))
            };
            if let Some((c_name, sc)) = sel_cls {
                if sc > max_score {
                    max_score = sc;
                }
                if sc > 1 {
                    let clean_name = strip_loai_suffix(&c_name);
                    if clean_name.is_empty() {
                        abn_names.push(d_name.clone());
                    } else {
                        abn_names.push(format!("{}: {}", d_name, clean_name));
                    }
                }
            }
        }

        let (f_kq, f_pl) = match spec_id {
            "CK_NOI_TIM_MACH" => ("NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL"),
            "CK_NOI_HO_HAP" => ("NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL"),
            "CK_NOI_TIEU_HOA" => ("NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL"),
            "CK_NOI_TIET_NIEU_SD" => ("NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TIETNIEU_PL"),
            "CK_NOI_NOITIET" => ("NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL"),
            "CK_NOI_CO_XUONG_KHOP" => ("NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL"),
            "CK_NOI_THAN_KINH" => ("NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL"),
            "CK_NOI_TAM_THAN" => ("NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL"),
            "CK_NGOAI" => ("KET_QUA_KHAM_NGOAI_KHOA", "KHAM_NGOAI_KHOA_PL"),
            "CK_DA_LIEU" => ("KET_QUA_KHAM_DA_LIEU", "KHAM_DA_LIEU_PL"),
            "CK_PHU_SAN" => ("KET_QUA_KHAM_SAN_PHU_KHOA", "KHAM_SAN_PHU_KHOA_PL"),
            "CK_MAT" => ("BENH_KHAC_MAT", "KHAM_MAT_PL"),
            "CK_TMH" => ("BENH_KHAC_TAI_MUI_HONG", "KHAM_TAI_MUI_HONG_PL"),
            "CK_RHM" => ("BENH_KHAC_RANG_HAM_MAT", "KHAM_RANG_HAM_MAT_PL"),
            _ => return,
        };

        let kq_val = if abn_names.is_empty() {
            "Bình thường".to_string()
        } else {
            abn_names.join("; ")
        };
        let pl_val = max_score.to_string();

        self.form_values.insert(f_pl.into(), pl_val);
        self.form_values.insert(f_kq.into(), kq_val.clone());
        if let Some(ta) = self.form_textareas.get(f_kq) {
            ta.update(cx, |s, cx| s.set_value(kq_val.clone(), window, cx));
        }
        if let Some(inp) = self.form_inputs.get(f_kq) {
            inp.update(cx, |s, cx| s.set_value(kq_val, window, cx));
        }
        if let Ok(json) = serde_json::to_string(&self.selected_classifications) {
            self.form_values.insert("SELECTED_CLASSIFICATIONS_JSON".into(), json);
        }
    }
}

fn strip_loai_suffix(s: &str) -> String {
    let mut out = s.trim();
    for pat in [
        "(Loại 1)", "(Loại 2)", "(Loại 3)", "(Loại 4)", "(Loại 5)", "(Loại 6)",
        "(loại 1)", "(loại 2)", "(loại 3)", "(loại 4)", "(loại 5)", "(loại 6)",
    ] {
        if let Some(stripped) = out.strip_suffix(pat) {
            out = stripped.trim();
            break;
        }
    }
    out.to_string()
}

impl KskGpuiApp {
    fn render_specialty_classification_block(
        &mut self,
        spec_id: &str,
        spec_title: &str,
        editable: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let auto_scores = self.auto_scores_from_form();
        let diseases = self.load_specialty_diseases(spec_id, &auto_scores);
        let mut max_score: i64 = 1;
        let mut abnormal_rows: Vec<(String, String, i64)> = Vec::new();

        for (d_code, d_name, classes) in &diseases {
            let sel_cls = if let Some((auto_cid, auto_sc)) = auto_scores.get(d_code) {
                classes
                    .iter()
                    .find(|(c, _, _, _, _)| c == auto_cid)
                    .map(|(_, n, _, _, _)| (n.clone(), *auto_sc))
                    .or_else(|| Some((d_name.clone(), *auto_sc)))
            } else {
                let sel_id = self.selected_classifications.get(d_code);
                sel_id
                    .and_then(|cid| classes.iter().find(|(c, _, _, _, _)| c == cid))
                    .or_else(|| classes.iter().find(|(_, _, sc, _, _)| *sc == 1))
                    .or_else(|| classes.first())
                    .map(|(_, n, sc, _, _)| (n.clone(), *sc))
            };
            if let Some((c_name, sc)) = sel_cls {
                if sc > max_score {
                    max_score = sc;
                }
                if sc > 1 {
                    abnormal_rows.push((d_name.clone(), c_name, sc));
                }
            }
        }

        let (max_bg, max_col) = Self::score_badge_colors(max_score);
        let expand_all = self.expand_all_diseases;
        let has_collapsible = diseases.iter().any(|(_, _, cls)| cls.len() >= 4);
        let spec_own = spec_id.to_string();

        div()
            .flex()
            .flex_col()
            .gap(px(10.0))
            .child(
                // Header + Kết luận tự động chuyên khoa
                div()
                    .p(px(12.0))
                    .rounded(px(8.0))
                    .bg(rgb(bg_subtle()))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(text_main()))
                                            .child(format!("Phân loại bệnh tật sẵn — {}", spec_title)),
                                    )
                                    .child(badge(&format!("Loại {}", max_score), max_bg, max_col)),
                            )
                            .when(has_collapsible, |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .gap(px(6.0))
                                        .child(
                                            Button::new(SharedString::from(format!("exp-all-{}", spec_id)))
                                                .label("Mở tất cả")
                                                .xsmall()
                                                .secondary()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.expand_all_diseases = true;
                                                    cx.notify();
                                                })),
                                        )
                                        .child(
                                            Button::new(SharedString::from(format!("col-all-{}", spec_id)))
                                                .label("Thu gọn tất cả")
                                                .xsmall()
                                                .secondary()
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.expand_all_diseases = false;
                                                    this.expanded_diseases.clear();
                                                    cx.notify();
                                                })),
                                        ),
                                )
                            }),
                    )
                    .when(abnormal_rows.is_empty(), |el| {
                        el.child(
                            div()
                                .px(px(10.0))
                                .py(px(6.0))
                                .rounded(px(6.0))
                                .bg(rgb(SUCCESS_BG))
                                .text_xs()
                                .text_color(rgb(SUCCESS_COL))
                                .child("✓ Tất cả các chỉ tiêu đều bình thường (Loại 1)"),
                        )
                    })
                    .when(!abnormal_rows.is_empty(), |el| {
                        el.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .children(abnormal_rows.into_iter().map(|(dn, cn, sc)| {
                                    let (bg, col) = Self::score_badge_colors(sc);
                                    div()
                                        .px(px(10.0))
                                        .py(px(6.0))
                                        .rounded(px(6.0))
                                        .bg(rgb(bg_card()))
                                        .border_1()
                                        .border_color(rgb(border_col()))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_xs()
                                                .text_color(rgb(text_main()))
                                                .child(format!("{} — {}", dn, cn)),
                                        )
                                        .child(badge(&format!("Loại {}", sc), bg, col))
                                })),
                        )
                    }),
            )
            .when(diseases.is_empty(), |el| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(rgb(text_muted()))
                        .child("Chưa có mục bệnh tật chi tiết cho chuyên khoa này trong danh mục."),
                )
            })
            .children(diseases.into_iter().map(|(d_code, d_name, classes)| {
                if let Some((auto_cid, auto_sc)) = auto_scores.get(&d_code) {
                    let (a_bg, a_col) = Self::score_badge_colors(*auto_sc);
                    let std_name = classes
                        .iter()
                        .find(|(c, _, _, _, _)| c == auto_cid)
                        .map(|(_, n, _, _, _)| n.clone())
                        .unwrap_or_else(|| format!("Tự động xếp Loại {}", auto_sc));
                    let detail = self.auto_disease_detail(&d_code).unwrap_or_default();

                    return div()
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(rgb(PRIMARY_COL))
                        .bg(rgb(bg_card()))
                        .px(px(12.0))
                        .py(px(8.0))
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .gap(px(8.0))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(2.0))
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .text_color(rgb(text_main()))
                                                .child(d_name),
                                        )
                                        .child(badge("TỰ ĐỘNG", primary_light(), PRIMARY_COL)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted()))
                                        .child(format!("{} • {}", detail, std_name)),
                                ),
                        )
                        .child(badge(&format!("Loại {}", auto_sc), a_bg, a_col))
                        .into_any_element();
                }

                let is_radio_inline = !classes.is_empty() && classes.len() < 4;
                let is_open = is_radio_inline || expand_all || self.expanded_diseases.contains(&d_code);
                let sel_id = self.selected_classifications.get(&d_code).cloned();
                let current_cls = sel_id
                    .as_ref()
                    .and_then(|cid| classes.iter().find(|(c, _, _, _, _)| c == cid))
                    .or_else(|| classes.iter().find(|(_, _, sc, _, _)| *sc == 1))
                    .or_else(|| classes.first());
                let (cur_cls_code, cur_cls_name, cur_score) = current_cls
                    .map(|(c, n, s, _, _)| (c.clone(), n.clone(), *s))
                    .unwrap_or_else(|| (String::new(), "Mặc định: Bình thường (Loại 1)".into(), 1));
                let (s_bg, s_col) = Self::score_badge_colors(cur_score);
                let d_code_toggle = d_code.clone();

                div()
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .bg(rgb(bg_card()))
                    .overflow_hidden()
                    .child(
                        div()
                            .id(SharedString::from(format!("dis-hdr-{}", d_code)))
                            .px(px(12.0))
                            .py(px(8.0))
                            .bg(rgb(bg_subtle()))
                            .cursor_pointer()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(8.0))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if this.expanded_diseases.contains(&d_code_toggle) {
                                    this.expanded_diseases.remove(&d_code_toggle);
                                } else {
                                    this.expanded_diseases.insert(d_code_toggle.clone());
                                }
                                cx.notify();
                            }))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(text_main()))
                                            .child(d_name),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted()))
                                            .child(cur_cls_name),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(badge(&format!("Loại {}", cur_score), s_bg, s_col))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted()))
                                            .child(if is_open { "▲" } else { "▼" }),
                                    ),
                            ),
                    )
                    .when(is_open, |el| {
                        el.child(
                            div()
                                .p(px(8.0))
                                .border_t_1()
                                .border_color(rgb(border_col()))
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .children(classes.into_iter().map(|(c_code, c_name, sc, ref_rule, is_ref)| {
                                    let checked = c_code == cur_cls_code;
                                    let (c_bg, c_col) = Self::score_badge_colors(sc);
                                    let d_c = d_code.clone();
                                    let c_c = c_code.clone();
                                    let sp_c = spec_own.clone();
                                    let badge_txt = if is_ref {
                                        format!("Loại {} *", sc)
                                    } else {
                                        format!("Loại {}", sc)
                                    };

                                    div()
                                        .id(SharedString::from(format!("cls-opt-{}-{}", d_code, c_code)))
                                        .px(px(8.0))
                                        .py(px(6.0))
                                        .rounded(px(6.0))
                                        .border_1()
                                        .border_color(if checked {
                                            rgb(PRIMARY_COL)
                                        } else {
                                            rgb(bg_card())
                                        })
                                        .bg(if checked {
                                            rgb(primary_light())
                                        } else {
                                            rgb(bg_card())
                                        })
                                        .when(editable, |d| {
                                            d.cursor_pointer().on_click(cx.listener(move |this, _, window, cx| {
                                                this.selected_classifications.insert(d_c.clone(), c_c.clone());
                                                this.sync_specialty_from_classifications(&sp_c, window, cx);
                                                cx.notify();
                                            }))
                                        })
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .w(px(14.0))
                                                        .h(px(14.0))
                                                        .rounded(px(7.0))
                                                        .border_1()
                                                        .border_color(if checked {
                                                            rgb(PRIMARY_COL)
                                                        } else {
                                                            rgb(border_col())
                                                        })
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .when(checked, |dot| {
                                                            dot.child(
                                                                div()
                                                                    .w(px(8.0))
                                                                    .h(px(8.0))
                                                                    .rounded(px(4.0))
                                                                    .bg(rgb(PRIMARY_COL)),
                                                            )
                                                        }),
                                                )
                                                .child(
                                                    div()
                                                        .flex_1()
                                                        .flex()
                                                        .flex_col()
                                                        .gap(px(2.0))
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(rgb(text_main()))
                                                                .child(c_name),
                                                        )
                                                        .when_some(ref_rule, |r_el, rr| {
                                                            r_el.child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(rgb(WARN_COL))
                                                                    .child(format!("↳ {}", rr)),
                                                            )
                                                        }),
                                                ),
                                        )
                                        .child(badge(&badge_txt, c_bg, c_col))
                                })),
                        )
                    })
                    .into_any_element()
            }))
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
                    .text_color(rgb(text_muted()))
                    .child(format!(
                        "Hiển thị {} mặt bệnh (tối đa 50 dòng/lượt lọc). Nhấn vào mức phân loại để ghi vào kết luận bệnh:",
                        matching_items.len()
                    )),
            )
            .children(matching_items.into_iter().map(|(_did, d_name, classes)| {
                div()
                    .p(px(10.0))
                    .rounded(px(6.0))
                    .bg(rgb(bg_subtle()))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(text_main()))
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
                    .border_color(rgb(border_col()))
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
