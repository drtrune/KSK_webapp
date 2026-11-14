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
    ("TSBT_BENH_TRONG_5_NAM_QUA", "1. Bá»‡nh trong 5 nÄƒm qua"),
    ("TSBT_BENH_THAN_KINH", "2. Bá»‡nh tháº§n kinh"),
    ("TSBT_BENH_MAT", "3. Bá»‡nh máº¯t"),
    ("TSBT_BENH_TAI", "4. Bá»‡nh tai"),
    ("TSBT_BENH_TIM", "5. Bá»‡nh tim máº¡ch"),
    ("TSBT_PHAU_THUAT_TIM", "6. Pháº«u thuáº­t tim"),
    ("TSBT_TANG_HUYET_AP", "7. TÄƒng huyáº¿t Ã¡p"),
    ("TSBT_KHO_THO", "8. KhÃ³ thá»Ÿ"),
    ("TSBT_BENH_PHOI", "9. Bá»‡nh phá»•i / hen"),
    ("TSBT_BENH_THAN", "10. Bá»‡nh tháº­n"),
    ("TSBT_NGHIEN_RUOU", "11. Nghiá»‡n rÆ°á»£u"),
    ("TSBT_DAI_THAO_DUONG", "12. ÄÃ¡i thÃ¡o Ä‘Æ°á»ng"),
    ("TSBT_BENH_TAM_THAN", "13. Bá»‡nh tÃ¢m tháº§n"),
    ("TSBT_MAT_Y_THUC", "14. Máº¥t Ã½ thá»©c / Ä‘á»™ng kinh"),
    ("TSBT_NGAT", "15. Ngáº¥t"),
    ("TSBT_BENH_TIEU_HOA", "16. Bá»‡nh tiÃªu hÃ³a"),
    ("TSBT_ROI_LOAN_GIAC_NGU", "17. Rá»‘i loáº¡n giáº¥c ngá»§"),
    ("TSBT_TAI_BIEN", "18. Tai biáº¿n máº¡ch mÃ¡u nÃ£o"),
    ("TSBT_BENH_COT_SONG", "19. Bá»‡nh cá»™t sá»‘ng"),
    ("TSBT_RUOU_THUONG_XUYEN", "20. Uá»‘ng rÆ°á»£u thÆ°á»ng xuyÃªn"),
    ("TSBT_MA_TUY", "21. Sá»­ dá»¥ng ma tÃºy"),
];

fn pl_options() -> Vec<(String, String)> {
    vec![
        ("".to_string(), "-- ChÆ°a phÃ¢n loáº¡i --".to_string()),
        ("1".to_string(), "Loáº¡i 1 (Ráº¥t khá»e)".to_string()),
        ("2".to_string(), "Loáº¡i 2 (Khá»e)".to_string()),
        ("3".to_string(), "Loáº¡i 3 (Trung bÃ¬nh)".to_string()),
        ("4".to_string(), "Loáº¡i 4 (Yáº¿u)".to_string()),
        ("5".to_string(), "Loáº¡i 5 (Ráº¥t yáº¿u)".to_string()),
    ]
}

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
                                .child("ChÆ°a chá»n há»“ sÆ¡ bá»‡nh nhÃ¢n"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .child("Chá»n má»™t há»“ sÆ¡ á»Ÿ danh sÃ¡ch bÃªn trÃ¡i, quÃ©t mÃ£ váº¡ch, hoáº·c nháº¥n '+ Tiáº¿p nháº­n há»“ sÆ¡ má»›i'."),
                        ),
                )
                .into_any_element();
        }

        let is_locked = self.current_record_status == 2;
        let ho_ten = self.get_form_val("HO_TEN");
        let ma_bn = self.get_form_val("MA_Y_TE");
        let ma_tn = self.get_form_val("MA_TIEP_NHAN");

        let tabs = [
            ("reception", "1. Tiáº¿p nháº­n"),
            ("history", "2. Tiá»n sá»­"),
            ("physical", "3. Thá»ƒ lá»±c"),
            ("clinical", "4. LÃ¢m sÃ ng"),
            ("paraclinical", "5. Cáº­n lÃ¢m sÃ ng"),
            ("conclusion", "6. Káº¿t luáº­n"),
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
                                                "Há»“ sÆ¡ tiáº¿p nháº­n má»›i".to_string()
                                            } else if ho_ten.is_empty() {
                                                "(ChÆ°a cÃ³ há» tÃªn)".to_string()
                                            } else {
                                                ho_ten
                                            }),
                                    )
                                    .when(!ma_bn.is_empty(), |el| {
                                        el.child(badge(format!("MÃ£ BN: {}", ma_bn), BG_SUBTLE, TEXT_MAIN))
                                    })
                                    .when(!ma_tn.is_empty(), |el| {
                                        el.child(badge(format!("Sá»‘ TN: {}", ma_tn), PRIMARY_LIGHT, PRIMARY_COL))
                                    }),
                            )
                            .child(match self.current_record_status {
                                2 => badge("ÄÃ£ kÃ½ khÃ³a", DANGER_BG, DANGER_COL),
                                1 => badge("HoÃ n táº¥t", SUCCESS_BG, SUCCESS_COL),
                                _ => badge("Äang khÃ¡m", WARN_BG, WARN_COL),
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
        let can_role = self.has_any_role(&["admin", "reception", "tiensu"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("reception");
        let gioi_tinh = self.get_form_val("GIOI_TINH");
        let kcb = self.get_form_val("MA_LOAI_KCB");

        let gender_opts = vec![
            ("1".to_string(), "1 - Nam".to_string()),
            ("2".to_string(), "2 - Ná»¯".to_string()),
            ("3".to_string(), "3 - KhÃ¡c".to_string()),
        ];
        let kcb_opts = vec![
            ("1".to_string(), "1 - KhÃ¡m sá»©c khá»e Ä‘á»‹nh ká»³".to_string()),
            ("2".to_string(), "2 - KhÃ¡m sá»©c khá»e tuyá»ƒn dá»¥ng".to_string()),
            ("3".to_string(), "3 - KhÃ¡m sá»©c khá»e lÃ¡i xe".to_string()),
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "reception",
                        "ThÃ´ng tin hÃ nh chÃ­nh & Tiáº¿p nháº­n",
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
                            .child(self.render_form_input("HO_TEN", "Há» vÃ  tÃªn", "NGUYá»„N VÄ‚N A", true, editable, window, cx))
                            .child(self.render_form_input("MA_Y_TE", "MÃ£ bá»‡nh nhÃ¢n (HIS)", "BN...", true, editable, window, cx))
                            .child(self.render_form_input("MA_TIEP_NHAN", "Sá»‘ tiáº¿p nháº­n", "TN...", false, editable, window, cx))
                            .child(self.render_form_input("NGAY_SINH", "NgÃ y sinh (YYYYMMDD)", "19900101", true, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_select_dropdown(
                                "rec-gioi-tinh",
                                "Giá»›i tÃ­nh",
                                &gioi_tinh,
                                gender_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("GIOI_TINH".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("SO_CCCD", "Sá»‘ CCCD / CMND", "052...", false, editable, window, cx))
                            .child(self.render_form_input("NGAYCAP_CCCD", "NgÃ y cáº¥p CCCD", "20210501", false, editable, window, cx))
                            .child(self.render_form_input("NOICAP_CCCD", "NÆ¡i cáº¥p CCCD", "Cá»¥c CSQLHC...", false, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("DIEN_THOAI", "Sá»‘ Ä‘iá»‡n thoáº¡i", "09...", false, editable, window, cx))
                            .child(self.render_form_input("DIA_CHI", "Äá»‹a chá»‰ thÆ°á»ng trÃº", "Sá»‘ nhÃ , Ä‘Æ°á»ng...", false, editable, window, cx))
                            .child(self.render_form_input("NOI_LAM_VIEC_HOC_TAP", "NÆ¡i lÃ m viá»‡c / Há»c táº­p", "CÃ´ng ty / ÄÆ¡n vá»‹...", false, editable, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_select_dropdown(
                                "rec-loai-kcb",
                                "Loáº¡i hÃ¬nh khÃ¡m (QÄ 3176)",
                                &kcb,
                                kcb_opts,
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("MA_LOAI_KCB".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("NGAY_VAO", "NgÃ y vÃ o khÃ¡m (YYYYMMDDHHMM)", "", false, editable, window, cx))
                            .child(self.render_form_input("NHOM_MAU", "NhÃ³m mÃ¡u", "O / A / B / AB", false, editable, window, cx))
                            .child(self.render_form_input("LY_DO_VV", "LÃ½ do khÃ¡m", "KhÃ¡m sá»©c khá»e", false, editable, window, cx)),
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
                                        .label("In Phiáº¿u Tiáº¿p Nháº­n (DOCX)")
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

        let mut icd_opts = vec![("".to_string(), "-- Chá»n nhanh ICD-10 --".to_string())];
        for c in &self.catalogs {
            if c.cat_type == "icd10" {
                icd_opts.push((c.code.clone(), format!("{} - {}", c.code, c.name)));
            }
        }

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "history",
                        "Tiá»n sá»­ bá»‡nh táº­t (Gia Ä‘Ã¬nh & Báº£n thÃ¢n - 21 má»¥c)",
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
                                "Tiá»n sá»­ gia Ä‘Ã¬nh máº¯c bá»‡nh?",
                                &ts_gd,
                                vec![
                                    ("0".to_string(), "0 - KhÃ´ng".to_string()),
                                    ("1".to_string(), "1 - CÃ³".to_string()),
                                ],
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("TSGD_MAC_BENH".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("TSGD_MA_BENH", "MÃ£ bá»‡nh gia Ä‘Ã¬nh (náº¿u cÃ³)", "", false, editable, window, cx)),
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
                            .child(self.render_form_input("TSBT_MA_BENH_KHAC", "Bá»‡nh khÃ¡c (náº¿u cÃ³)", "", false, editable, window, cx))
                            .child(self.render_select_dropdown(
                                "hist-icd10",
                                "Chá»n nhanh ICD-10 Ä‘ang Ä‘iá»u trá»‹",
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
                            .child(self.render_form_input("TSBT_MA_BENH", "MÃ£ bá»‡nh Ä‘ang Ä‘iá»u trá»‹", "", false, editable, window, cx))
                            .child(self.render_form_input("TSBT_TEN_THUOC_LIEU_LUONG", "Thuá»‘c & liá»u lÆ°á»£ng Ä‘ang dÃ¹ng", "", false, editable, window, cx)),
                    ),
            )
            .into_any_element()
    }

    fn render_tab_physical(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let can_role = self.has_any_role(&["admin", "tiensu", "bs_noi"]);
        let editable = !is_locked && can_role && self.editing_section.as_deref() == Some("physical");
        let pl = self.get_form_val("KHAM_THE_LUC_PL");

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(self.render_section_action_bar(
                        "physical",
                        "KhÃ¡m Thá»ƒ lá»±c & Sinh hiá»‡u",
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
                            .child(self.render_form_input("CHIEU_CAO", "Chiá»u cao (cm)", "170", false, editable, window, cx))
                            .child(self.render_form_input("CAN_NANG", "CÃ¢n náº·ng (kg)", "65", false, editable, window, cx))
                            .child(self.render_form_input("CHI_SO_BMI", "Chá»‰ sá»‘ BMI (tá»± tÃ­nh)", "22.5", false, false, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("MACH", "Máº¡ch (láº§n/phÃºt)", "75", false, editable, window, cx))
                            .child(self.render_form_input("HUYET_AP", "Huyáº¿t Ã¡p (mmHg)", "120/80", false, editable, window, cx))
                            .child(self.render_select_dropdown(
                                "phys-pl",
                                "PhÃ¢n loáº¡i thá»ƒ lá»±c",
                                &pl,
                                pl_options(),
                                editable,
                                |this, val, _, _| {
                                    this.form_values.insert("KHAM_THE_LUC_PL".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("BS_THE_LUC", "NgÆ°á»i khÃ¡m thá»ƒ lá»±c", "", false, editable, window, cx)),
                    ),
            )
            .into_any_element()
    }

    fn render_tab_clinical(
        &mut self,
        is_locked: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let mode_btn_quick = Button::new("btn-clin-quick")
            .label("KhÃ¡m chuyÃªn khoa")
            .small()
            .when(!self.clinical_tree_mode, |b| b.primary())
            .when(self.clinical_tree_mode, |b| b.secondary())
            .on_click(cx.listener(|this, _, _, cx| {
                this.clinical_tree_mode = false;
                cx.notify();
            }));

        let mode_btn_tree = Button::new("btn-clin-tree")
            .label("Tra cá»©u phÃ¢n loáº¡i bá»‡nh (110+ bá»‡nh)")
            .small()
            .when(self.clinical_tree_mode, |b| b.primary())
            .when(!self.clinical_tree_mode, |b| b.secondary())
            .on_click(cx.listener(|this, _, _, cx| {
                this.clinical_tree_mode = true;
                cx.notify();
            }));

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(rgb(TEXT_MAIN))
                    .child("KhÃ¡m LÃ¢m sÃ ng ChuyÃªn khoa"),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(mode_btn_quick)
                    .child(mode_btn_tree),
            );

        if self.clinical_tree_mode {
            return div()
                .flex()
                .flex_col()
                .gap(px(14.0))
                .child(header)
                .child(self.render_disease_classification_tree(window, cx))
                .into_any_element();
        }

        // Quick / Specialty cards
        let can_noi = self.has_any_role(&["admin", "bs_noi", "bs_theluc"]);
        let edit_noi = !is_locked && can_noi && self.editing_section.as_deref() == Some("clin_noi");

        let can_ngoai = self.has_any_role(&["admin", "bs_ngoai"]);
        let edit_ngoai = !is_locked && can_ngoai && self.editing_section.as_deref() == Some("clin_ngoai");

        let can_dalieu = self.has_any_role(&["admin", "bs_dalieu"]);
        let edit_dalieu = !is_locked && can_dalieu && self.editing_section.as_deref() == Some("clin_dalieu");

        let can_san = self.has_any_role(&["admin", "bs_san"]);
        let edit_san = !is_locked && can_san && self.editing_section.as_deref() == Some("clin_san");

        let can_mat = self.has_any_role(&["admin", "bs_mat"]);
        let edit_mat = !is_locked && can_mat && self.editing_section.as_deref() == Some("clin_mat");

        let can_tmh = self.has_any_role(&["admin", "bs_tmh"]);
        let edit_tmh = !is_locked && can_tmh && self.editing_section.as_deref() == Some("clin_tmh");

        let can_rhm = self.has_any_role(&["admin", "bs_rhm"]);
        let edit_rhm = !is_locked && can_rhm && self.editing_section.as_deref() == Some("clin_rhm");

        let noi_fields = [
            ("NOI_KHOA_TUAN_HOAN", "NOI_KHOA_TUAN_HOAN_PL", "Tuáº§n hoÃ n"),
            ("NOI_KHOA_HO_HAP", "NOI_KHOA_HO_HAP_PL", "HÃ´ háº¥p"),
            ("NOI_KHOA_TIEU_HOA", "NOI_KHOA_TIEU_HOA_PL", "TiÃªu hÃ³a"),
            ("NOI_KHOA_THAN_TN_SD", "NOI_KHOA_THAN_TIETNIEU_PL", "Tháº­n - Tiáº¿t niá»‡u"),
            ("NOI_KHOA_NOI_TIET", "NOI_KHOA_NOI_TIET_PL", "Ná»™i tiáº¿t"),
            ("NOI_KHOA_CO_XUONG_KHOP", "NOI_KHOA_CO_XUONG_KHOP_PL", "CÆ¡ xÆ°Æ¡ng khá»›p"),
            ("NOI_KHOA_THAN_KINH", "NOI_KHOA_THAN_KINH_PL", "Tháº§n kinh"),
            ("NOI_KHOA_TAM_THAN", "NOI_KHOA_TAM_THAN_PL", "TÃ¢m tháº§n"),
        ];

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(header)
            // Ná»™i khoa
            .child({
                let mut box_noi = card_box().child(self.render_section_action_bar(
                    "clin_noi",
                    "1. Ná»™i khoa & TÃ¢m tháº§n",
                    can_noi,
                    is_locked,
                    cx,
                ));
                for (f_kq, f_pl, lbl) in noi_fields {
                    let pl_val = self.get_form_val(f_pl);
                    let f_pl_owned = f_pl.to_string();
                    box_noi = box_noi.child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .child(self.render_form_input(f_kq, lbl, "BÃ¬nh thÆ°á»ng", false, edit_noi, window, cx))
                            .child(self.render_select_dropdown(
                                &format!("dd-{}", f_pl),
                                "PhÃ¢n loáº¡i",
                                &pl_val,
                                pl_options(),
                                edit_noi,
                                move |this, val, _, _| {
                                    this.form_values.insert(f_pl_owned.clone(), val);
                                },
                                cx,
                            )),
                    );
                }
                box_noi.child(self.render_form_input("BS_NOI", "BÃ¡c sÄ© Ná»™i khoa", "", false, edit_noi, window, cx))
            })
            // Ngoáº¡i khoa + Da liá»…u + Sáº£n
            .child(self.render_simple_specialty_card(
                "clin_ngoai",
                "2. Ngoáº¡i khoa",
                "KET_QUA_KHAM_NGOAI_KHOA",
                "KHAM_NGOAI_KHOA_PL",
                "BS_NGOAI",
                can_ngoai,
                edit_ngoai,
                is_locked,
                window,
                cx,
            ))
            .child(self.render_simple_specialty_card(
                "clin_dalieu",
                "3. Da liá»…u",
                "KET_QUA_KHAM_DA_LIEU",
                "KHAM_DA_LIEU_PL",
                "BS_DA_LIEU",
                can_dalieu,
                edit_dalieu,
                is_locked,
                window,
                cx,
            ))
            .child(self.render_simple_specialty_card(
                "clin_san",
                "4. Sáº£n phá»¥ khoa",
                "KET_QUA_KHAM_SAN_PHU_KHOA",
                "KHAM_SAN_PHU_KHOA_PL",
                "BS_SAN",
                can_san,
                edit_san,
                is_locked,
                window,
                cx,
            ))
            // Máº¯t
            .child({
                let pl = self.get_form_val("KHAM_MAT_PL");
                card_box()
                    .child(self.render_section_action_bar("clin_mat", "5. Máº¯t", can_mat, is_locked, cx))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("KHONG_KINH_MAT_PHAI", "KhÃ´ng kÃ­nh MP", "10/10", false, edit_mat, window, cx))
                            .child(self.render_form_input("KHONG_KINH_MAT_TRAI", "KhÃ´ng kÃ­nh MT", "10/10", false, edit_mat, window, cx))
                            .child(self.render_form_input("CO_KINH_MAT_PHAI", "CÃ³ kÃ­nh MP", "10/10", false, edit_mat, window, cx))
                            .child(self.render_form_input("CO_KINH_MAT_TRAI", "CÃ³ kÃ­nh MT", "10/10", false, edit_mat, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("BENH_KHAC_MAT", "Bá»‡nh vá» máº¯t", "KhÃ´ng", false, edit_mat, window, cx))
                            .child(self.render_select_dropdown(
                                "dd-KHAM_MAT_PL",
                                "PhÃ¢n loáº¡i Máº¯t",
                                &pl,
                                pl_options(),
                                edit_mat,
                                |this, val, _, _| {
                                    this.form_values.insert("KHAM_MAT_PL".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("BS_MAT", "BÃ¡c sÄ© Máº¯t", "", false, edit_mat, window, cx)),
                    )
            })
            // TMH
            .child({
                let pl = self.get_form_val("KHAM_TAI_MUI_HONG_PL");
                card_box()
                    .child(self.render_section_action_bar("clin_tmh", "6. Tai - MÅ©i - Há»ng", can_tmh, is_locked, cx))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("TAI_PHAI_NOI_THUONG", "Tai pháº£i nÃ³i thÆ°á»ng (m)", "5m", false, edit_tmh, window, cx))
                            .child(self.render_form_input("TAI_PHAI_NOI_THAM", "Tai pháº£i nÃ³i tháº§m (m)", "0.5m", false, edit_tmh, window, cx))
                            .child(self.render_form_input("TAI_TRAI_NOI_THUONG", "Tai trÃ¡i nÃ³i thÆ°á»ng (m)", "5m", false, edit_tmh, window, cx))
                            .child(self.render_form_input("TAI_TRAI_NOI_THAM", "Tai trÃ¡i nÃ³i tháº§m (m)", "0.5m", false, edit_tmh, window, cx)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("BENH_KHAC_TAI_MUI_HONG", "Bá»‡nh TMH", "BÃ¬nh thÆ°á»ng", false, edit_tmh, window, cx))
                            .child(self.render_select_dropdown(
                                "dd-KHAM_TAI_MUI_HONG_PL",
                                "PhÃ¢n loáº¡i TMH",
                                &pl,
                                pl_options(),
                                edit_tmh,
                                |this, val, _, _| {
                                    this.form_values.insert("KHAM_TAI_MUI_HONG_PL".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("BS_TMH", "BÃ¡c sÄ© TMH", "", false, edit_tmh, window, cx)),
                    )
            })
            // RHM
            .child({
                let pl = self.get_form_val("KHAM_RANG_HAM_MAT_PL");
                card_box()
                    .child(self.render_section_action_bar("clin_rhm", "7. RÄƒng - HÃ m - Máº·t", can_rhm, is_locked, cx))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(12.0))
                            .child(self.render_form_input("HAM_TREN", "HÃ m trÃªn", "BÃ¬nh thÆ°á»ng", false, edit_rhm, window, cx))
                            .child(self.render_form_input("HAM_DUOI", "HÃ m dÆ°á»›i", "BÃ¬nh thÆ°á»ng", false, edit_rhm, window, cx))
                            .child(self.render_form_input("BENH_KHAC_RANG_HAM_MAT", "Bá»‡nh RHM", "KhÃ´ng", false, edit_rhm, window, cx))
                            .child(self.render_select_dropdown(
                                "dd-KHAM_RANG_HAM_MAT_PL",
                                "PhÃ¢n loáº¡i RHM",
                                &pl,
                                pl_options(),
                                edit_rhm,
                                |this, val, _, _| {
                                    this.form_values.insert("KHAM_RANG_HAM_MAT_PL".into(), val);
                                },
                                cx,
                            ))
                            .child(self.render_form_input("BS_RHM", "BÃ¡c sÄ© RHM", "", false, edit_rhm, window, cx)),
                    )
            })
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
        let pl_val = self.get_form_val(f_pl);
        let f_pl_owned = f_pl.to_string();
        card_box()
            .child(self.render_section_action_bar(sec_key, title, can_role, is_locked, cx))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(self.render_form_input(f_kq, "Káº¿t quáº£ khÃ¡m", "BÃ¬nh thÆ°á»ng", false, editable, window, cx))
                    .child(self.render_select_dropdown(
                        &format!("dd-{}", f_pl),
                        "PhÃ¢n loáº¡i",
                        &pl_val,
                        pl_options(),
                        editable,
                        move |this, val, _, _| {
                            this.form_values.insert(f_pl_owned.clone(), val);
                        },
                        cx,
                    ))
                    .child(self.render_form_input(f_bs, "BÃ¡c sÄ© khÃ¡m", "", false, editable, window, cx)),
            )
            .into_any_element()
    }

    fn render_disease_classification_tree(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let cfg = phan_loai_config();
        let search_q = self.disease_search_input.read(cx).value().to_lowercase();
        let sel_spec = self.selected_specialty_filter.clone();

        let mut spec_opts = vec![("".to_string(), "Táº¥t cáº£ chuyÃªn khoa".to_string())];
        for s in &cfg.specialties {
            spec_opts.push((s.id.clone(), s.name.clone()));
        }

        let matching_diseases: Vec<_> = cfg
            .diseases
            .iter()
            .filter(|d| {
                (sel_spec.is_empty() || d.specialty_id == sel_spec)
                    && (search_q.is_empty() || d.name.to_lowercase().contains(&search_q))
            })
            .take(50)
            .collect();

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
                    .child(format!("Hiá»ƒn thá»‹ {} máº·t bá»‡nh (tá»‘i Ä‘a 50 dÃ²ng/lÆ°á»£t lá»c). Nháº¥n vÃ o má»©c phÃ¢n loáº¡i Ä‘á»ƒ ghi vÃ o káº¿t luáº­n bá»‡nh:", matching_diseases.len())),
            )
            .children(matching_diseases.into_iter().map(|d| {
                let classes: Vec<_> = cfg
                    .classifications
                    .iter()
                    .filter(|c| c.disease_id == d.id)
                    .collect();
                let d_name = d.name.clone();
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
                            .child(d.name.clone()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(6.0))
                            .children(classes.into_iter().map(|c| {
                                let score_str = c.score.map(|s| format!("Loáº¡i {}", s)).unwrap_or_else(|| "Ghi chÃº".into());
                                let label = format!("{}: {}", score_str, c.name);
                                let append_text = format!("{} ({})", d_name, score_str);
                                let cid = c.id.clone();
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
                                            inp.update(cx, |s, cx| s.set_value(next, window, cx));
                                        }
                                        this.show_toast(&format!("ÄÃ£ thÃªm: {}", append_text), false, cx);
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
                "Káº¿t quáº£ Cáº­n lÃ¢m sÃ ng (XÃ©t nghiá»‡m & CÄHA)",
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
                    .child(self.render_form_input("CLS_HUYET_HOC", "CÃ´ng thá»©c mÃ¡u", "BÃ¬nh thÆ°á»ng", false, editable, window, cx))
                    .child(self.render_form_input("CLS_DUONG_MAU", "ÄÆ°á»ng mÃ¡u (mmol/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_URE", "UrÃª (mmol/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_CREATININ", "Creatinin (Âµmol/L)", "", false, editable, window, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(self.render_form_input("CLS_ASAT_GOT", "AST / GOT (U/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_ALAT_GPT", "ALT / GPT (U/L)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_NUOC_TIEU", "Tá»•ng phÃ¢n tÃ­ch nÆ°á»›c tiá»ƒu", "BÃ¬nh thÆ°á»ng", false, editable, window, cx))
                    .child(self.render_form_input("CLS_XQ_TIM_PHOI", "X-Quang tim phá»•i", "BÃ¬nh thÆ°á»ng", false, editable, window, cx)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(12.0))
                    .child(self.render_form_input("CLS_KHAC_MO_TA", "Cáº­n lÃ¢m sÃ ng khÃ¡c (SiÃªu Ã¢m, ECG...)", "", false, editable, window, cx))
                    .child(self.render_form_input("CLS_DINH_KY_KET_QUA", "Káº¿t luáº­n Cáº­n lÃ¢m sÃ ng", "BÃ¬nh thÆ°á»ng", false, editable, window, cx))
                    .child(self.render_form_input("BS_CLS", "BÃ¡c sÄ© CLS", "", false, editable, window, cx)),
            )
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
        let pl = self.get_form_val("PHAN_LOAI_SK");

        card_box()
            .child(self.render_section_action_bar(
                "conclusion",
                "Káº¿t luáº­n Sá»©c khá»e & KÃ½ khÃ³a há»“ sÆ¡",
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
                        "conc-phan-loai",
                        "PhÃ¢n loáº¡i sá»©c khá»e chung",
                        &pl,
                        pl_options(),
                        editable,
                        |this, val, _, _| {
                            this.form_values.insert("PHAN_LOAI_SK".into(), val);
                        },
                        cx,
                    ))
                    .child(self.render_form_input("KET_LUAN_BENH", "Káº¿t luáº­n Ä‘á»§ sá»©c khá»e / Bá»‡nh táº­t", "Äá»§ sá»©c khá»e lÃ m viá»‡c/há»c táº­p", false, editable, window, cx))
                    .child(self.render_form_input("BS_KET_LUAN", "BÃ¡c sÄ© káº¿t luáº­n", "", false, editable, window, cx)),
            )
            .child(self.render_form_input("CAC_BENH_TAT_NEU_CO", "CÃ¡c bá»‡nh táº­t náº¿u cÃ³", "", false, editable, window, cx))
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
                                        .label("KÃ½ khÃ³a há»“ sÆ¡ (HoÃ n táº¥t)")
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
                                        .label("Má»Ÿ khÃ³a há»“ sÆ¡ (Admin)")
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
                                .label("Xuáº¥t Giáº¥y KSK (DOCX)")
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
