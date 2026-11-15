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
        let q = self.grp_search.read(cx).value().trim().to_lowercase();
        let companies = self.companies_list.clone();
        let contracts = self.contracts_list.clone();

        div()
            .relative()
            .flex_1()
            .h_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                // Top toolbar
                div()
                    .px(px(16.0))
                    .py(px(12.0))
                    .bg(rgb(bg_card()))
                    .border_b_1()
                    .border_color(rgb(border_col()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(text_main()))
                                    .child(format!(
                                        "Công ty & Hợp đồng Khám đoàn ({} công ty • {} hợp đồng)",
                                        companies.len(),
                                        contracts.len()
                                    )),
                            )
                            .child(
                                div()
                                    .w(px(280.0))
                                    .child(Input::new(&self.grp_search).small().cleanable(true)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                Button::new("btn-grp-add-company")
                                    .label("+ Thêm Công ty")
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear_company_form(window, cx);
                                        this.grp_show_company_dialog = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-grp-add-contract")
                                    .label("+ Thêm Hợp đồng")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear_contract_form(window, cx);
                                        if this.grp_cont_company_id.is_none() {
                                            this.grp_cont_company_id = this
                                                .companies_list
                                                .iter()
                                                .find(|c| c.is_active == 1)
                                                .map(|c| c.id);
                                        }
                                        this.grp_show_contract_dialog = true;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-grp-refresh")
                                    .label("↻ Làm mới")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh_group_data(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                // 2-level unified list: Level 1 = Company, Level 2 = Contracts
                div()
                    .id("group-tree-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .p(px(16.0))
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
                    .when(companies.is_empty(), |el| {
                        el.child(
                            card_box().child(
                                div()
                                    .p(px(20.0))
                                    .text_sm()
                                    .text_color(rgb(text_muted()))
                                    .child("Chưa có công ty nào. Bấm '+ Thêm Công ty' ở góc trên phải để bắt đầu."),
                            ),
                        )
                    })
                    .children(companies.into_iter().filter_map(|comp| {
                        let comp_contracts: Vec<_> = contracts
                            .iter()
                            .filter(|ct| ct.company_id == comp.id)
                            .cloned()
                            .collect();

                        if !q.is_empty() {
                            let comp_hit = comp.name.to_lowercase().contains(&q)
                                || comp.code.to_lowercase().contains(&q)
                                || comp.tax_code.to_lowercase().contains(&q);
                            let cont_hit = comp_contracts.iter().any(|ct| {
                                ct.contract_code.to_lowercase().contains(&q)
                                    || ct.title.to_lowercase().contains(&q)
                            });
                            if !comp_hit && !cont_hit {
                                return None;
                            }
                        }

                        let cid = comp.id;
                        let is_collapsed = self.grp_collapsed_companies.contains(&cid);
                        let comp_edit = comp.clone();
                        let next_active = comp.is_active == 0;
                        let comp_cont_count = comp_contracts.len();

                        Some(
                            card_box()
                                .child(
                                    // Level 1: Company row
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(10.0))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .child(
                                                    Button::new(SharedString::from(format!("btn-comp-fold-{}", cid)))
                                                        .label(if is_collapsed { "▸" } else { "▾" })
                                                        .small()
                                                        .secondary()
                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                            if this.grp_collapsed_companies.contains(&cid) {
                                                                this.grp_collapsed_companies.remove(&cid);
                                                            } else {
                                                                this.grp_collapsed_companies.insert(cid);
                                                            }
                                                            cx.notify();
                                                        })),
                                                )
                                                .child(badge(comp.code.clone(), primary_light(), PRIMARY_COL))
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
                                                                        .text_sm()
                                                                        .font_weight(gpui::FontWeight::BOLD)
                                                                        .text_color(rgb(text_main()))
                                                                        .child(comp.name.clone()),
                                                                )
                                                                .child(badge(
                                                                    format!("{} hợp đồng", comp_cont_count),
                                                                    bg_subtle(),
                                                                    text_main(),
                                                                ))
                                                                .child(if comp.is_active == 1 {
                                                                    badge("Hoạt động", SUCCESS_BG, SUCCESS_COL)
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
                                                                    if comp.tax_code.is_empty() { "-" } else { &comp.tax_code },
                                                                    if comp.contact_person.is_empty() { "-" } else { &comp.contact_person },
                                                                    if comp.contact_phone.is_empty() { "-" } else { &comp.contact_phone },
                                                                    if comp.address.is_empty() { "-" } else { &comp.address },
                                                                )),
                                                        ),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .gap(px(6.0))
                                                .child(
                                                    Button::new(SharedString::from(format!("btn-comp-add-cont-{}", cid)))
                                                        .label("+ Thêm HĐ")
                                                        .small()
                                                        .primary()
                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                            this.clear_contract_form(window, cx);
                                                            this.grp_cont_company_id = Some(cid);
                                                            this.grp_show_contract_dialog = true;
                                                            cx.notify();
                                                        })),
                                                )
                                                .child(
                                                    Button::new(SharedString::from(format!("btn-comp-edit-{}", cid)))
                                                        .label("Sửa")
                                                        .small()
                                                        .secondary()
                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                            this.grp_comp_edit_id = Some(comp_edit.id);
                                                            this.grp_comp_code.update(cx, |s, cx| s.set_value(comp_edit.code.clone(), window, cx));
                                                            this.grp_comp_name.update(cx, |s, cx| s.set_value(comp_edit.name.clone(), window, cx));
                                                            this.grp_comp_tax.update(cx, |s, cx| s.set_value(comp_edit.tax_code.clone(), window, cx));
                                                            this.grp_comp_addr.update(cx, |s, cx| s.set_value(comp_edit.address.clone(), window, cx));
                                                            this.grp_comp_contact.update(cx, |s, cx| s.set_value(comp_edit.contact_person.clone(), window, cx));
                                                            this.grp_comp_phone.update(cx, |s, cx| s.set_value(comp_edit.contact_phone.clone(), window, cx));
                                                            this.grp_comp_note.update(cx, |s, cx| s.set_value(comp_edit.note.clone(), window, cx));
                                                            this.grp_show_company_dialog = true;
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
                                        ),
                                )
                                .when(!is_collapsed, |el| {
                                    el.child(
                                        // Level 2: Nested Contracts
                                        div()
                                            .pl(px(32.0))
                                            .pt(px(8.0))
                                            .border_t_1()
                                            .border_color(rgb(border_col()))
                                            .flex()
                                            .flex_col()
                                            .gap(px(8.0))
                                            .when(comp_contracts.is_empty(), |c_el| {
                                                c_el.child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(text_muted()))
                                                        .child("Chưa có hợp đồng nào. Bấm '+ Thêm HĐ' để tạo hợp đồng khám cho công ty này."),
                                                )
                                            })
                                            .children(comp_contracts.into_iter().map(|ct| {
                                                let cont_id = ct.id;
                                                let has_scan = !ct.scan_file_path.trim().is_empty();
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
                                                                    .child(badge(ct.contract_code.clone(), primary_light(), PRIMARY_COL))
                                                                    .child(
                                                                        div()
                                                                            .text_sm()
                                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                            .text_color(rgb(text_main()))
                                                                            .child(ct.title.clone()),
                                                                    )
                                                                    .child(if ct.status == 0 {
                                                                        badge("Đang mở khám", SUCCESS_BG, SUCCESS_COL)
                                                                    } else {
                                                                        badge("Đã đóng", WARN_BG, WARN_COL)
                                                                    })
                                                                    .when(has_scan, |b| {
                                                                        b.child(badge("Có file scan", primary_light(), PRIMARY_COL))
                                                                    }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .text_xs()
                                                                    .text_color(rgb(text_muted()))
                                                                    .child(format!(
                                                                        "Ngày khám: {}  •  Đã tạo: {}/{}  •  Hoàn tất: {}  •  Đã ký: {}",
                                                                        if ct.exam_date.is_empty() { "-" } else { &ct.exam_date },
                                                                        ct.total_records,
                                                                        ct.expected_count,
                                                                        ct.done_records,
                                                                        ct.signed_records,
                                                                    )),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .flex_row()
                                                            .gap(px(6.0))
                                                            .child(
                                                                Button::new(SharedString::from(format!("btn-cont-detail-{}", cont_id)))
                                                                    .label("Chi tiết hợp đồng")
                                                                    .small()
                                                                    .primary()
                                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                                        this.open_contract_detail(cont_id, cx);
                                                                    })),
                                                            )
                                                            .child(
                                                                Button::new(SharedString::from(format!("btn-cont-exam-{}", cont_id)))
                                                                    .label("Khám đoàn này")
                                                                    .small()
                                                                    .secondary()
                                                                    .on_click(cx.listener(move |this, _, window, cx| {
                                                                        this.active_nav = "exam".into();
                                                                        this.sidebar_exam_mode = "doan".into();
                                                                        this.sidebar_contract_id = Some(cont_id);
                                                                        this.selected_record_id = None;
                                                                        this.refresh_records(window, cx);
                                                                    })),
                                                            ),
                                                    )
                                            })),
                                    )
                                }),
                        )
                    })),
            )
            .when_some(self.grp_detail_contract_id, |el, cid| {
                el.child(self.render_contract_detail_window(cid, window, cx))
            })
            .when(self.grp_show_import_dialog, |el| {
                el.child(self.render_group_import_dialog(cx))
            })
            .when(self.grp_show_company_dialog, |el| {
                el.child(self.render_company_dialog(cx))
            })
            .when(self.grp_show_contract_dialog, |el| {
                el.child(self.render_contract_dialog(cx))
            })
            .into_any_element()
    }

    fn render_company_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let is_editing = self.grp_comp_edit_id.is_some();
        div()
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000080))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(560.0))
                    .p(px(20.0))
                    .rounded(px(10.0))
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
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
                                    .child(if is_editing {
                                        "Cập nhật thông tin Công ty"
                                    } else {
                                        "Thêm Công ty / Đơn vị mới"
                                    }),
                            )
                            .child(
                                Button::new("btn-dlg-comp-close")
                                    .label("✕")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_show_company_dialog = false;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(160.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã công ty", true))
                                    .child(Input::new(&self.grp_comp_code).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên công ty / Đơn vị", true))
                                    .child(Input::new(&self.grp_comp_name).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã số thuế", false))
                                    .child(Input::new(&self.grp_comp_tax).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Người liên hệ", false))
                                    .child(Input::new(&self.grp_comp_contact).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("SĐT liên hệ", false))
                                    .child(Input::new(&self.grp_comp_phone).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Địa chỉ", false))
                            .child(Input::new(&self.grp_comp_addr).small()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Ghi chú", false))
                            .child(Input::new(&self.grp_comp_note).small()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(
                                Button::new("btn-dlg-comp-cancel")
                                    .label("Hủy")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear_company_form(window, cx);
                                        this.grp_show_company_dialog = false;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-dlg-comp-save")
                                    .label(if is_editing { "Lưu thay đổi" } else { "Tạo công ty" })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_company(window, cx);
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_contract_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let is_editing = self.grp_cont_edit_id.is_some();
        let comp_val = self
            .grp_cont_company_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let mut comp_opts: Vec<(String, String)> = vec![("".into(), "-- Chọn công ty --".into())];
        for c in self.companies_list.iter().filter(|c| c.is_active == 1) {
            comp_opts.push((c.id.to_string(), format!("{} - {}", c.code, c.name)));
        }

        div()
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000080))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(580.0))
                    .p(px(20.0))
                    .rounded(px(10.0))
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .gap(px(12.0))
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
                                    .child(if is_editing {
                                        "Sửa thông tin Hợp đồng"
                                    } else {
                                        "Thêm Hợp đồng / Đợt khám mới"
                                    }),
                            )
                            .child(
                                Button::new("btn-dlg-cont-close")
                                    .label("✕")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_show_contract_dialog = false;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(self.render_select_dropdown(
                        "dlg-cont-company",
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
                            .flex()
                            .flex_row()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(170.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã hợp đồng", true))
                                    .child(Input::new(&self.grp_cont_code).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên đợt khám", true))
                                    .child(Input::new(&self.grp_cont_title).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(200.0))
                                    .child(self.render_date_picker_entity(
                                        "dlg-cont-date",
                                        "Ngày khám dự kiến",
                                        self.grp_cont_date.clone(),
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .w(px(150.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Số lượng dự kiến", false))
                                    .child(Input::new(&self.grp_cont_expected).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
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
                            .child(
                                Button::new("btn-dlg-cont-cancel")
                                    .label("Hủy")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.clear_contract_form(window, cx);
                                        this.grp_show_contract_dialog = false;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-dlg-cont-save")
                                    .label(if is_editing { "Lưu hợp đồng" } else { "Tạo hợp đồng" })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_contract(window, cx);
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }

    fn render_contract_detail_window(
        &mut self,
        cid: i64,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(contract) = self.contracts_list.iter().find(|c| c.id == cid).cloned() else {
            return div().into_any_element();
        };

        let next_status = if contract.status == 0 { 1 } else { 0 };
        let scan_path = contract.scan_file_path.clone();
        let has_scan = !scan_path.trim().is_empty();

        let total = self.grp_detail_records.len();
        let in_prog = self.grp_detail_records.iter().filter(|r| r.status == 0).count();
        let done = self.grp_detail_records.iter().filter(|r| r.status == 1).count();
        let signed = self.grp_detail_records.iter().filter(|r| r.status == 2).count();
        let pct = if total > 0 {
            ((done + signed) * 100) / total
        } else {
            0
        };

        let st_filter = self.grp_detail_status_filter;
        let filtered_recs: Vec<_> = self
            .grp_detail_records
            .iter()
            .filter(|r| st_filter.map(|s| r.status == s).unwrap_or(true))
            .cloned()
            .collect();

        let c_edit = contract.clone();
        let scan_open = scan_path.clone();

        div()
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000080))
            .p(px(28.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .h_full()
                    .max_w(px(1080.0))
                    .rounded(px(10.0))
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        // Sub-window Header
                        div()
                            .px(px(18.0))
                            .py(px(12.0))
                            .bg(rgb(bg_subtle()))
                            .border_b_1()
                            .border_color(rgb(border_col()))
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
                                    .child(badge(contract.contract_code.clone(), primary_light(), PRIMARY_COL))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child(format!("{} — {}", contract.company_name, contract.title)),
                                    )
                                    .child(if contract.status == 0 {
                                        badge("Đang mở khám", SUCCESS_BG, SUCCESS_COL)
                                    } else {
                                        badge("Đã đóng", WARN_BG, WARN_COL)
                                    }),
                            )
                            .child(
                                Button::new("btn-detail-close")
                                    .label("✕ Đóng")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_detail_contract_id = None;
                                        this.grp_detail_records.clear();
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        // Action bar
                        div()
                            .px(px(18.0))
                            .py(px(10.0))
                            .border_b_1()
                            .border_color(rgb(border_col()))
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
                                    .flex_wrap()
                                    .gap(px(8.0))
                                    .child(
                                        Button::new("btn-detail-import")
                                            .label("Import danh sách (.xlsx / .csv)")
                                            .small()
                                            .primary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.grp_import_contract_id = Some(cid);
                                                this.grp_import_rows.clear();
                                                this.grp_import_file_path.clear();
                                                this.grp_show_import_dialog = true;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-detail-print-group-rec")
                                            .label("In phôi tiếp nhận cả đoàn (A4)")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.print_group_html(cid, true, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-detail-print-group-full")
                                            .label("In phiếu KSK cả đoàn (A4)")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.print_group_html(cid, false, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-detail-edit")
                                            .label("Sửa hợp đồng")
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
                                                this.grp_show_contract_dialog = true;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-detail-toggle-status")
                                            .label(if next_status == 1 { "Đóng hợp đồng" } else { "Mở lại hợp đồng" })
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
                                    )
                                    .child(
                                        Button::new("btn-detail-go-exam")
                                            .label("Mở đoàn này để khám")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.grp_detail_contract_id = None;
                                                this.active_nav = "exam".into();
                                                this.sidebar_exam_mode = "doan".into();
                                                this.sidebar_contract_id = Some(cid);
                                                this.selected_record_id = None;
                                                this.refresh_records(window, cx);
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(text_muted()))
                                    .child(format!(
                                        "Ngày khám: {} • Ghi chú: {}",
                                        if contract.exam_date.is_empty() { "-" } else { &contract.exam_date },
                                        if contract.note.is_empty() { "-" } else { &contract.note },
                                    )),
                            ),
                    )
                    .child(
                        // Scan file attachment bar
                        div()
                            .px(px(18.0))
                            .py(px(10.0))
                            .border_b_1()
                            .border_color(rgb(border_col()))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .gap(px(10.0))
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
                                            .child("File scan hợp đồng:"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(if has_scan { PRIMARY_COL } else { text_muted() }))
                                            .child(if has_scan {
                                                scan_path
                                            } else {
                                                "Chưa đính kèm file scan (PDF / Ảnh)".to_string()
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .child(
                                        Button::new("btn-detail-pick-scan")
                                            .label(if has_scan { "Đổi file scan" } else { "Nhập file scan HĐ" })
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if let Some(p) = rfd::FileDialog::new()
                                                    .add_filter("PDF / Ảnh", &["pdf", "png", "jpg", "jpeg", "webp"])
                                                    .pick_file()
                                                {
                                                    let path_str = p.to_string_lossy().to_string();
                                                    let state = this.state.clone();
                                                    let path_clone = path_str.clone();
                                                    let res = this.run_sync(async move {
                                                        let pool = state.db.read().await;
                                                        db::update_exam_contract_scan_file(&pool, cid, &path_clone).await
                                                    });
                                                    match res {
                                                        Ok(_) => {
                                                            this.refresh_group_data(cx);
                                                            this.show_toast("Đã lưu đường dẫn file scan hợp đồng!", false, cx);
                                                        }
                                                        Err(e) => this.show_toast(&e, true, cx),
                                                    }
                                                }
                                            })),
                                    )
                                    .when(has_scan, |el| {
                                        el.child(
                                            Button::new("btn-detail-open-scan")
                                                .label("Mở file scan")
                                                .small()
                                                .secondary()
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    let res = std::process::Command::new("cmd")
                                                        .args(["/C", "start", "", &scan_open])
                                                        .spawn();
                                                    if let Err(e) = res {
                                                        this.show_toast(&format!("Không mở được file: {}", e), true, cx);
                                                    }
                                                })),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        // Progress & examinee status tracking
                        div()
                            .px(px(18.0))
                            .py(px(10.0))
                            .border_b_1()
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
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(badge(format!("Tổng: {}/{} dự kiến", total, contract.expected_count), bg_subtle(), text_main()))
                                    .child(badge(format!("Đang khám: {}", in_prog), WARN_BG, WARN_COL))
                                    .child(badge(format!("Hoàn tất: {}", done), SUCCESS_BG, SUCCESS_COL))
                                    .child(badge(format!("Đã ký: {}", signed), DANGER_BG, DANGER_COL))
                                    .child(badge(format!("Tiến độ: {}%", pct), primary_light(), PRIMARY_COL)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(4.0))
                                    .child(
                                        Button::new("btn-det-f-all")
                                            .label("Tất cả")
                                            .small()
                                            .when(st_filter.is_none(), |b| b.primary())
                                            .when(st_filter.is_some(), |b| b.secondary())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.grp_detail_status_filter = None;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-det-f-0")
                                            .label("Đang khám")
                                            .small()
                                            .when(st_filter == Some(0), |b| b.primary())
                                            .when(st_filter != Some(0), |b| b.secondary())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.grp_detail_status_filter = Some(0);
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-det-f-1")
                                            .label("Hoàn tất")
                                            .small()
                                            .when(st_filter == Some(1), |b| b.primary())
                                            .when(st_filter != Some(1), |b| b.secondary())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.grp_detail_status_filter = Some(1);
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-det-f-2")
                                            .label("Đã ký")
                                            .small()
                                            .when(st_filter == Some(2), |b| b.primary())
                                            .when(st_filter != Some(2), |b| b.secondary())
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.grp_detail_status_filter = Some(2);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("contract-detail-records-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .p(px(16.0))
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .when(filtered_recs.is_empty(), |el| {
                                el.child(
                                    div()
                                        .p(px(16.0))
                                        .text_sm()
                                        .text_color(rgb(text_muted()))
                                        .child("Chưa có phiếu khám nào trong hợp đồng này (hoặc không có phiếu khớp bộ lọc). Bấm 'Import danh sách (.xlsx / .csv)' ở trên để tạo phiếu."),
                                )
                            })
                            .children(filtered_recs.into_iter().enumerate().map(|(idx, rec)| {
                                let (st_lbl, st_bg, st_fg) = match rec.status {
                                    2 => ("Đã ký khóa", DANGER_BG, DANGER_COL),
                                    1 => ("Hoàn tất", SUCCESS_BG, SUCCESS_COL),
                                    _ => ("Đang khám", WARN_BG, WARN_COL),
                                };
                                let rec_id = rec.id;
                                div()
                                    .p(px(8.0))
                                    .rounded(px(6.0))
                                    .border_1()
                                    .border_color(rgb(border_col()))
                                    .bg(rgb(bg_subtle()))
                                    .flex()
                                    .flex_row()
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
                                                    .w(px(30.0))
                                                    .text_xs()
                                                    .text_color(rgb(text_muted()))
                                                    .child(format!("#{}", idx + 1)),
                                            )
                                            .child(badge(rec.ma_tiep_nhan.clone(), primary_light(), PRIMARY_COL))
                                            .when(!rec.ma_y_te.is_empty(), |b| {
                                                b.child(badge(rec.ma_y_te.clone(), bg_card(), text_main()))
                                            })
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .child(rec.ho_ten.clone()),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(text_muted()))
                                                    .child(format!(
                                                        "NS: {} • CCCD: {}",
                                                        if rec.ngay_sinh.is_empty() { "-" } else { &rec.ngay_sinh },
                                                        if rec.so_cccd.is_empty() { "-" } else { &rec.so_cccd },
                                                    )),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(badge(st_lbl, st_bg, st_fg))
                                            .child(
                                                Button::new(SharedString::from(format!("btn-det-open-rec-{}", rec_id)))
                                                    .label("Mở hồ sơ")
                                                    .small()
                                                    .secondary()
                                                    .on_click(cx.listener(move |this, _, window, cx| {
                                                        this.grp_detail_contract_id = None;
                                                        this.active_nav = "exam".into();
                                                        this.sidebar_exam_mode = "doan".into();
                                                        this.sidebar_contract_id = Some(cid);
                                                        this.refresh_records(window, cx);
                                                        this.load_record_into_form(rec_id, window, cx);
                                                    })),
                                            ),
                                    )
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_group_import_dialog(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let cont_label = self
            .grp_import_contract_id
            .and_then(|cid| self.contracts_list.iter().find(|c| c.id == cid))
            .map(|c| format!("{} - {} ({})", c.contract_code, c.company_name, c.title))
            .unwrap_or_else(|| "Hợp đồng đã chọn".into());

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
            .absolute()
            .inset_0()
            .bg(gpui::rgba(0x00000090))
            .p(px(48.0))
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w_full()
                    .h_full()
                    .max_w(px(920.0))
                    .rounded(px(10.0))
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(border_col()))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .child(
                        div()
                            .px(px(18.0))
                            .py(px(12.0))
                            .bg(rgb(bg_subtle()))
                            .border_b_1()
                            .border_color(rgb(border_col()))
                            .flex()
                            .flex_row()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_base()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child("Import phần hành chính tạo phiếu Khám đoàn"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(PRIMARY_COL))
                                            .child(format!("Hợp đồng: {}", cont_label)),
                                    ),
                            )
                            .child(
                                Button::new("btn-dlg-imp-close")
                                    .label("✕ Đóng")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.grp_show_import_dialog = false;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .px(px(18.0))
                            .py(px(12.0))
                            .border_b_1()
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
                                    .gap(px(8.0))
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
                                    )
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
                            )
                            .when(total_rows > 0, |el| {
                                el.child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(badge(format!("Tổng: {} dòng", total_rows), bg_subtle(), text_main()))
                                        .child(badge(format!("Hợp lệ: {}", valid_rows), SUCCESS_BG, SUCCESS_COL))
                                        .when(error_rows > 0, |b| {
                                            b.child(badge(format!("Lỗi/Trùng: {}", error_rows), DANGER_BG, DANGER_COL))
                                        }),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("dlg-imp-preview-scroll")
                            .flex_1()
                            .overflow_y_scroll()
                            .p(px(16.0))
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .when(total_rows == 0, |el| {
                                el.child(
                                    div()
                                        .p(px(20.0))
                                        .text_sm()
                                        .text_color(rgb(text_muted()))
                                        .child("Chọn file Excel (.xlsx/.xls) hoặc CSV có sẵn Mã tiếp nhận, Mã y tế và thông tin hành chính để xem trước."),
                                )
                            })
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
                    ),
            )
            .into_any_element()
    }

    fn open_contract_detail(&mut self, cid: i64, cx: &mut Context<Self>) {
        self.grp_detail_contract_id = Some(cid);
        self.grp_detail_status_filter = None;
        self.refresh_contract_detail_records(cid, cx);
    }

    fn refresh_contract_detail_records(&mut self, cid: i64, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let mk = self.master_key;
        let recs = self.run_sync(async move {
            let pool = state.db.read().await;
            db::list_contract_records(&pool, cid, mk.as_ref()).await
        });
        self.grp_detail_records = recs;
        cx.notify();
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
                self.grp_show_company_dialog = false;
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
                self.grp_show_contract_dialog = false;
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
                self.grp_show_import_dialog = false;
                self.sidebar_contract_id = Some(cid);
                self.refresh_group_data(cx);
                if self.grp_detail_contract_id == Some(cid) {
                    self.refresh_contract_detail_records(cid, cx);
                }
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
