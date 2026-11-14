use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder as _, px, rgb,
};
use gpui_component::{
    Sizable,
    button::{Button, ButtonVariants},
    input::Input,
};

use super::{
    KskGpuiApp,
    widgets::{
        bg_card, bg_subtle, border_col, DANGER_BG, DANGER_COL, PRIMARY_COL, primary_light,
        SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, badge, card_box,
        field_label,
    },
};
use crate::{db, group_import};

impl KskGpuiApp {
    pub fn render_group_exam_tab(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let sub = self.grp_sub_tab.clone();

        let body: AnyElement = match sub.as_str() {
            "companies" => self.render_group_companies(window, cx),
            "contracts" => self.render_group_contracts(window, cx),
            _ => self.render_group_import(window, cx),
        };

        div()
            .id("group-exam-tab-scroll")
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .p(px(16.0))
            .flex()
            .flex_col()
            .gap(px(14.0))
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
                            .gap(px(8.0))
                            .child(
                                Button::new("grp-sub-import")
                                    .label("1. Import hành chính & Tạo phiếu đoàn")
                                    .small()
                                    .when(sub == "import", |b| b.primary())
                                    .when(sub != "import", |b| b.secondary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_sub_tab = "import".into();
                                        this.refresh_group_data(cx);
                                    })),
                            )
                            .child(
                                Button::new("grp-sub-contracts")
                                    .label(format!("2. Hợp đồng khám ({})", self.contracts_list.len()))
                                    .small()
                                    .when(sub == "contracts", |b| b.primary())
                                    .when(sub != "contracts", |b| b.secondary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_sub_tab = "contracts".into();
                                        this.refresh_group_data(cx);
                                    })),
                            )
                            .child(
                                Button::new("grp-sub-companies")
                                    .label(format!("3. Danh mục Công ty ({})", self.companies_list.len()))
                                    .small()
                                    .when(sub == "companies", |b| b.primary())
                                    .when(sub != "companies", |b| b.secondary())
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_sub_tab = "companies".into();
                                        this.refresh_group_data(cx);
                                    })),
                            ),
                    )
                    .child(
                        Button::new("grp-refresh-btn")
                            .label("↻ Làm mới")
                            .small()
                            .secondary()
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.refresh_group_data(cx);
                            })),
                    ),
            )
            .child(body)
            .into_any_element()
    }

    fn render_group_companies(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_editing = self.grp_comp_edit_id.is_some();
        let companies = self.companies_list.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(if is_editing {
                                "Cập nhật thông tin Công ty"
                            } else {
                                "Thêm Công ty / Đơn vị mới"
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(140.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã công ty", true))
                                    .child(Input::new(&self.grp_comp_code).small()),
                            )
                            .child(
                                div()
                                    .w(px(280.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên công ty / Đơn vị", true))
                                    .child(Input::new(&self.grp_comp_name).small()),
                            )
                            .child(
                                div()
                                    .w(px(150.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã số thuế", false))
                                    .child(Input::new(&self.grp_comp_tax).small()),
                            )
                            .child(
                                div()
                                    .w(px(260.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Địa chỉ", false))
                                    .child(Input::new(&self.grp_comp_addr).small()),
                            )
                            .child(
                                div()
                                    .w(px(180.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Người liên hệ", false))
                                    .child(Input::new(&self.grp_comp_contact).small()),
                            )
                            .child(
                                div()
                                    .w(px(150.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("SĐT liên hệ", false))
                                    .child(Input::new(&self.grp_comp_phone).small()),
                            )
                            .child(
                                div()
                                    .w(px(220.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Ghi chú", false))
                                    .child(Input::new(&self.grp_comp_note).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .when(is_editing, |el| {
                                el.child(
                                    Button::new("btn-comp-cancel")
                                        .label("Hủy sửa")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.clear_company_form(window, cx);
                                        })),
                                )
                            })
                            .child(
                                Button::new("btn-comp-save")
                                    .label(if is_editing { "Cập nhật công ty" } else { "+ Thêm công ty" })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_company(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                card_box()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(format!("Danh sách Công ty ({})", companies.len())),
                    )
                    .when(companies.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(12.0))
                                .text_sm()
                                .text_color(rgb(text_muted()))
                                .child("Chưa có công ty nào. Hãy thêm công ty ở form phía trên."),
                        )
                    })
                    .children(companies.into_iter().map(|c| {
                        let cid = c.id;
                        let c_edit = c.clone();
                        let next_active = c.is_active == 0;
                        div()
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(rgb(border_col()))
                            .bg(rgb(bg_subtle()))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(3.0))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(badge(c.code.clone(), primary_light(), PRIMARY_COL))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(text_main()))
                                                    .child(c.name.clone()),
                                            )
                                            .child(if c.is_active == 1 {
                                                badge("Đang hoạt động", SUCCESS_BG, SUCCESS_COL)
                                            } else {
                                                badge("Đã khóa", DANGER_BG, DANGER_COL)
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted()))
                                            .child(format!(
                                                "MST: {}  •  Liên hệ: {} ({})  •  ĐC: {}",
                                                if c.tax_code.is_empty() { "-" } else { &c.tax_code },
                                                if c.contact_person.is_empty() { "-" } else { &c.contact_person },
                                                if c.contact_phone.is_empty() { "-" } else { &c.contact_phone },
                                                if c.address.is_empty() { "-" } else { &c.address },
                                            )),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .child(
                                        Button::new(SharedString::from(format!("btn-comp-edit-{}", cid)))
                                            .label("Sửa")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.grp_comp_edit_id = Some(c_edit.id);
                                                this.grp_comp_code.update(cx, |s, cx| s.set_value(c_edit.code.clone(), window, cx));
                                                this.grp_comp_name.update(cx, |s, cx| s.set_value(c_edit.name.clone(), window, cx));
                                                this.grp_comp_tax.update(cx, |s, cx| s.set_value(c_edit.tax_code.clone(), window, cx));
                                                this.grp_comp_addr.update(cx, |s, cx| s.set_value(c_edit.address.clone(), window, cx));
                                                this.grp_comp_contact.update(cx, |s, cx| s.set_value(c_edit.contact_person.clone(), window, cx));
                                                this.grp_comp_phone.update(cx, |s, cx| s.set_value(c_edit.contact_phone.clone(), window, cx));
                                                this.grp_comp_note.update(cx, |s, cx| s.set_value(c_edit.note.clone(), window, cx));
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-comp-lock-{}", cid)))
                                            .label(if next_active { "Mở khóa" } else { "Khóa" })
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                let state = this.state.clone();
                                                let res = this.run_sync(async move {
                                                    let pool = state.db.read().await;
                                                    db::toggle_company_active(&pool, cid, next_active).await
                                                });
                                                match res {
                                                    Ok(_) => this.refresh_group_data(cx),
                                                    Err(e) => this.show_toast(&e, true, cx),
                                                }
                                            })),
                                    ),
                            )
                    })),
            )
            .into_any_element()
    }

    fn render_group_contracts(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_editing = self.grp_cont_edit_id.is_some();
        let comp_val = self
            .grp_cont_company_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let mut comp_opts: Vec<(String, String)> = vec![("".into(), "-- Chọn công ty --".into())];
        for c in self.companies_list.iter().filter(|c| c.is_active == 1) {
            comp_opts.push((c.id.to_string(), format!("{} - {}", c.code, c.name)));
        }
        let contracts = self.contracts_list.clone();

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(if is_editing {
                                "Cập nhật Hợp đồng / Đợt khám đoàn"
                            } else {
                                "Tạo Hợp đồng / Đợt khám đoàn mới"
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(10.0))
                            .child(self.render_select_dropdown(
                                "grp-cont-company",
                                "Công ty / Đơn vị",
                                &comp_val,
                                comp_opts,
                                true,
                                |this, val, _, _| {
                                    this.grp_cont_company_id = val.parse::<i64>().ok();
                                },
                                cx,
                            ))
                            .child(
                                div()
                                    .w(px(160.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã hợp đồng", true))
                                    .child(Input::new(&self.grp_cont_code).small()),
                            )
                            .child(
                                div()
                                    .w(px(280.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên đợt khám", true))
                                    .child(Input::new(&self.grp_cont_title).small()),
                            )
                            .child(
                                div()
                                    .w(px(160.0))
                                    .child(self.render_date_picker_entity(
                                        "grp-cont-date",
                                        "Ngày khám dự kiến",
                                        self.grp_cont_date.clone(),
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .w(px(130.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("SL dự kiến", false))
                                    .child(Input::new(&self.grp_cont_expected).small()),
                            )
                            .child(
                                div()
                                    .w(px(220.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Ghi chú", false))
                                    .child(Input::new(&self.grp_cont_note).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .when(is_editing, |el| {
                                el.child(
                                    Button::new("btn-cont-cancel")
                                        .label("Hủy sửa")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.clear_contract_form(window, cx);
                                        })),
                                )
                            })
                            .child(
                                Button::new("btn-cont-save")
                                    .label(if is_editing { "Cập nhật hợp đồng" } else { "+ Tạo hợp đồng" })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_contract(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(
                card_box()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child(format!("Danh sách Hợp đồng khám ({})", contracts.len())),
                    )
                    .when(contracts.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(12.0))
                                .text_sm()
                                .text_color(rgb(text_muted()))
                                .child("Chưa có hợp đồng khám nào. Hãy tạo Công ty trước rồi tạo Hợp đồng."),
                        )
                    })
                    .children(contracts.into_iter().map(|c| {
                        let cid = c.id;
                        let c_edit = c.clone();
                        let next_status = if c.status == 0 { 1 } else { 0 };
                        div()
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(rgb(border_col()))
                            .bg(rgb(bg_subtle()))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(badge(c.contract_code.clone(), primary_light(), PRIMARY_COL))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(text_main()))
                                                    .child(format!("{} — {}", c.company_name, c.title)),
                                            )
                                            .child(if c.status == 0 {
                                                badge("Đang mở khám", SUCCESS_BG, SUCCESS_COL)
                                            } else {
                                                badge("Đã đóng", WARN_BG, WARN_COL)
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted()))
                                            .child(format!(
                                                "Ngày khám: {}  •  Đã tạo: {} / {} dự kiến  •  Hoàn tất: {}  •  Đã ký: {}",
                                                if c.exam_date.is_empty() { "-" } else { &c.exam_date },
                                                c.total_records,
                                                c.expected_count,
                                                c.done_records,
                                                c.signed_records,
                                            )),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cont-imp-{}", cid)))
                                            .label("Import DS")
                                            .small()
                                            .primary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.grp_import_contract_id = Some(cid);
                                                this.grp_sub_tab = "import".into();
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cont-open-{}", cid)))
                                            .label("Khám đoàn này")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.active_nav = "exam".into();
                                                this.sidebar_exam_mode = "doan".into();
                                                this.sidebar_contract_id = Some(cid);
                                                this.selected_record_id = None;
                                                this.refresh_records(window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cont-edit-{}", cid)))
                                            .label("Sửa")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.grp_cont_edit_id = Some(c_edit.id);
                                                this.grp_cont_company_id = Some(c_edit.company_id);
                                                this.grp_cont_code.update(cx, |s, cx| s.set_value(c_edit.contract_code.clone(), window, cx));
                                                this.grp_cont_title.update(cx, |s, cx| s.set_value(c_edit.title.clone(), window, cx));
                                                this.grp_cont_date.update(cx, |s, cx| s.set_value(c_edit.exam_date.clone(), window, cx));
                                                this.grp_cont_expected.update(cx, |s, cx| s.set_value(c_edit.expected_count.to_string(), window, cx));
                                                this.grp_cont_note.update(cx, |s, cx| s.set_value(c_edit.note.clone(), window, cx));
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cont-toggle-{}", cid)))
                                            .label(if next_status == 1 { "Đóng HĐ" } else { "Mở lại" })
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                let state = this.state.clone();
                                                let res = this.run_sync(async move {
                                                    let pool = state.db.read().await;
                                                    db::set_exam_contract_status(&pool, cid, next_status).await
                                                });
                                                match res {
                                                    Ok(_) => this.refresh_group_data(cx),
                                                    Err(e) => this.show_toast(&e, true, cx),
                                                }
                                            })),
                                    ),
                            )
                    })),
            )
            .into_any_element()
    }

    fn render_group_import(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let cont_val = self
            .grp_import_contract_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let mut cont_opts: Vec<(String, String)> = vec![("".into(), "-- Chọn hợp đồng khám --".into())];
        for c in &self.contracts_list {
            let st_txt = if c.status == 1 { " [Đã đóng]" } else { "" };
            cont_opts.push((
                c.id.to_string(),
                format!("{} - {} ({}){}", c.contract_code, c.company_name, c.title, st_txt),
            ));
        }

        let total_rows = self.grp_import_rows.len();
        let valid_rows = self.grp_import_rows.iter().filter(|r| r.error.is_none()).count();
        let error_rows = total_rows.saturating_sub(valid_rows);
        let preview_rows = self.grp_import_rows.clone();
        let file_label = if self.grp_import_file_path.is_empty() {
            "Chưa chọn file danh sách".to_string()
        } else {
            self.grp_import_file_path.clone()
        };

        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child("Import phần hành chính tạo phiếu Khám đoàn"),
                            )
                            .child(
                                Button::new("btn-grp-sample-csv")
                                    .label("Tải file mẫu (.csv)")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        if let Some(p) = rfd::FileDialog::new()
                                            .set_file_name("mau_import_kham_doan.csv")
                                            .add_filter("CSV UTF-8", &["csv"])
                                            .save_file()
                                        {
                                            let bytes = group_import::sample_group_import_csv_bytes();
                                            match std::fs::write(&p, bytes) {
                                                Ok(_) => this.show_toast(
                                                    &format!("Đã lưu file mẫu tại {}", p.display()),
                                                    false,
                                                    cx,
                                                ),
                                                Err(e) => this.show_toast(&format!("Lỗi lưu file mẫu: {}", e), true, cx),
                                            }
                                        }
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted()))
                            .child("Hỗ trợ file Excel (.xlsx, .xls) hoặc CSV có sẵn cột Mã tiếp nhận, Mã y tế và đầy đủ thông tin hành chính."),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_end()
                            .gap(px(10.0))
                            .child(self.render_select_dropdown(
                                "grp-imp-contract",
                                "Hợp đồng / Đợt khám",
                                &cont_val,
                                cont_opts,
                                true,
                                |this, val, _, _| {
                                    this.grp_import_contract_id = val.parse::<i64>().ok();
                                },
                                cx,
                            ))
                            .child(
                                Button::new("btn-grp-pick-file")
                                    .label("Chọn file danh sách (.xlsx / .csv)")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.pick_and_parse_group_file(cx);
                                    })),
                            )
                            .when(valid_rows > 0, |el| {
                                el.child(
                                    Button::new("btn-grp-commit-import")
                                        .label(format!("Tạo {} phiếu khám đoàn", valid_rows))
                                        .small()
                                        .primary()
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.commit_group_import(cx);
                                        })),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted()))
                            .child(format!("File đang chọn: {}", file_label)),
                    ),
            )
            .when(total_rows > 0, |el| {
                el.child(
                    card_box()
                        .child(
                            div()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .child("Xem trước danh sách import"),
                                )
                                .child(badge(format!("Tổng: {} dòng", total_rows), bg_subtle(), text_main()))
                                .child(badge(format!("Hợp lệ: {}", valid_rows), SUCCESS_BG, SUCCESS_COL))
                                .when(error_rows > 0, |b| {
                                    b.child(badge(format!("Lỗi/Trùng: {}", error_rows), DANGER_BG, DANGER_COL))
                                }),
                        )
                        .children(preview_rows.into_iter().take(200).map(|r| {
                            let has_err = r.error.is_some();
                            let st_badge = if let Some(ref e) = r.error {
                                badge(e.clone(), DANGER_BG, DANGER_COL)
                            } else {
                                badge("Hợp lệ", SUCCESS_BG, SUCCESS_COL)
                            };
                            let gt_txt = match r.gioi_tinh.as_str() {
                                "2" => "Nữ",
                                "3" => "Khác",
                                _ => "Nam",
                            };
                            div()
                                .p(px(8.0))
                                .rounded(px(6.0))
                                .border_1()
                                .border_color(rgb(if has_err { DANGER_COL } else { border_col() }))
                                .bg(rgb(if has_err { DANGER_BG } else { bg_subtle() }))
                                .flex()
                                .flex_row()
                                .items_center()
                                .justify_between()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(2.0))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(text_muted()))
                                                        .child(format!("Dòng {}", r.row_index)),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(gpui::FontWeight::BOLD)
                                                        .child(r.ho_ten.clone()),
                                                )
                                                .child(badge(format!("TN: {}", r.ma_tiep_nhan), primary_light(), PRIMARY_COL))
                                                .when(!r.ma_y_te.is_empty(), |b| {
                                                    b.child(badge(format!("YT: {}", r.ma_y_te), bg_card(), text_main()))
                                                }),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(text_muted()))
                                                .child(format!(
                                                    "NS: {} • {} • CCCD: {} • SĐT: {} • ĐV: {}",
                                                    r.ngay_sinh, gt_txt, r.so_cccd, r.dien_thoai, r.noi_lam_viec_hoc_tap
                                                )),
                                        ),
                                )
                                .child(st_badge)
                        })),
                )
            })
            .into_any_element()
    }

    fn clear_company_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.grp_comp_edit_id = None;
        self.grp_comp_code.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_comp_name.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_comp_tax.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_comp_addr.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_comp_contact.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_comp_phone.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_comp_note.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }

    fn save_company(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let edit_id = self.grp_comp_edit_id;
        let code = self.grp_comp_code.read(cx).value().to_string();
        let name = self.grp_comp_name.read(cx).value().to_string();
        let tax = self.grp_comp_tax.read(cx).value().to_string();
        let addr = self.grp_comp_addr.read(cx).value().to_string();
        let contact = self.grp_comp_contact.read(cx).value().to_string();
        let phone = self.grp_comp_phone.read(cx).value().to_string();
        let note = self.grp_comp_note.read(cx).value().to_string();
        let state = self.state.clone();

        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            db::upsert_company(&pool, edit_id, &code, &name, &tax, &addr, &contact, &phone, &note).await
        });
        match res {
            Ok(_) => {
                self.clear_company_form(window, cx);
                self.refresh_group_data(cx);
                self.show_toast("Đã lưu thông tin công ty!", false, cx);
            }
            Err(e) => self.show_toast(&e, true, cx),
        }
    }

    fn clear_contract_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let today = chrono::Local::now().format("%d/%m/%Y").to_string();
        self.grp_cont_edit_id = None;
        self.grp_cont_code.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_cont_title.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_cont_date.update(cx, |s, cx| s.set_value(today, window, cx));
        self.grp_cont_expected.update(cx, |s, cx| s.set_value("", window, cx));
        self.grp_cont_note.update(cx, |s, cx| s.set_value("", window, cx));
        cx.notify();
    }

    fn save_contract(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let edit_id = self.grp_cont_edit_id;
        let cid = self.grp_cont_company_id.unwrap_or(0);
        let code = self.grp_cont_code.read(cx).value().to_string();
        let title = self.grp_cont_title.read(cx).value().to_string();
        let date = self.grp_cont_date.read(cx).value().to_string();
        let expected = self
            .grp_cont_expected
            .read(cx)
            .value()
            .trim()
            .parse::<i64>()
            .unwrap_or(0);
        let note = self.grp_cont_note.read(cx).value().to_string();
        let state = self.state.clone();

        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            db::upsert_exam_contract(&pool, edit_id, cid, &code, &title, &date, expected, &note).await
        });
        match res {
            Ok(new_id) => {
                self.grp_import_contract_id = Some(new_id);
                self.clear_contract_form(window, cx);
                self.refresh_group_data(cx);
                self.show_toast("Đã lưu hợp đồng khám đoàn!", false, cx);
            }
            Err(e) => self.show_toast(&e, true, cx),
        }
    }

    fn pick_and_parse_group_file(&mut self, cx: &mut Context<Self>) {
        let Some(cid) = self.grp_import_contract_id else {
            self.show_toast("Vui lòng chọn Hợp đồng khám trước khi chọn file!", true, cx);
            return;
        };
        let Some(contract) = self.contracts_list.iter().find(|c| c.id == cid).cloned() else {
            self.show_toast("Không tìm thấy hợp đồng đã chọn!", true, cx);
            return;
        };
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Excel / CSV", &["xlsx", "xls", "ods", "csv"])
            .pick_file()
        else {
            return;
        };

        let state = self.state.clone();
        let mk = self.master_key;
        let (existing_tn, existing_cccd) = self.run_sync(async move {
            let pool = state.db.read().await;
            let tns = db::get_existing_ma_tiep_nhan_set(&pool).await;
            let cccds = db::get_existing_contract_cccd_set(&pool, cid, mk.as_ref()).await;
            (tns, cccds)
        });

        match group_import::parse_group_import_file(
            &path,
            &contract.company_name,
            &contract.exam_date,
            &self.catalogs,
            &existing_tn,
            &existing_cccd,
        ) {
            Ok(rows) => {
                self.grp_import_file_path = path.to_string_lossy().to_string();
                self.grp_import_rows = rows;
                cx.notify();
            }
            Err(e) => self.show_toast(&e, true, cx),
        }
    }

    fn commit_group_import(&mut self, cx: &mut Context<Self>) {
        let Some(cid) = self.grp_import_contract_id else {
            self.show_toast("Vui lòng chọn Hợp đồng khám!", true, cx);
            return;
        };
        if self.grp_import_rows.is_empty() {
            return;
        }
        let rows = self.grp_import_rows.clone();
        let state = self.state.clone();
        let mk = self.master_key;
        let ma_cskcb = self.cfg.ma_cskcb.clone();
        let ma_gtin = self.cfg.ma_gtin_cskcb.clone();
        let updated_by = self
            .current_user
            .as_ref()
            .map(|u| u.username.clone())
            .unwrap_or_else(|| "business".into());

        let res = self.run_sync(async move {
            let pool = state.db.read().await;
            db::batch_insert_group_records(
                &pool,
                cid,
                &ma_cskcb,
                &ma_gtin,
                &updated_by,
                mk.as_ref(),
                &rows,
            )
            .await
        });

        match res {
            Ok((inserted, skipped)) => {
                self.grp_import_rows.clear();
                self.grp_import_file_path.clear();
                self.sidebar_contract_id = Some(cid);
                self.refresh_group_data(cx);
                self.show_toast(
                    &format!("Đã tạo {} phiếu khám đoàn (bỏ qua {} dòng lỗi)!", inserted, skipped),
                    false,
                    cx,
                );
            }
            Err(e) => self.show_toast(&e, true, cx),
        }
    }
}
