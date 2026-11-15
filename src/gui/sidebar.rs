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
        SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, badge,
    },
};

impl KskGpuiApp {
    pub fn render_sidebar(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        if !self.sidebar_open {
            return div()
                .w(px(36.0))
                .h_full()
                .bg(rgb(bg_card()))
                .border_r_1()
                .border_color(rgb(border_col()))
                .flex()
                .flex_col()
                .items_center()
                .pt(px(12.0))
                .child(
                    Button::new("btn-expand-sidebar")
                        .label("»")
                        .small()
                        .secondary()
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.sidebar_open = true;
                            cx.notify();
                        })),
                )
                .into_any_element();
        }

        let can_create = self.has_any_role(&["admin", "reception"]);
        let status_val = self
            .sidebar_status_filter
            .map(|s| s.to_string())
            .unwrap_or_default();

        let status_opts = vec![
            ("".to_string(), "Tất cả trạng thái".to_string()),
            ("0".to_string(), "Đang khám".to_string()),
            ("1".to_string(), "Hoàn tất (chờ ký)".to_string()),
            ("2".to_string(), "Đã ký khóa".to_string()),
        ];

        let mut records = self.records.clone();
        if self.sidebar_only_my_pending {
            let user_opt = self.current_user.clone();
            records.retain(|rec| {
                if rec.status == 2 {
                    return false;
                }
                let Some(user) = user_opt.as_ref() else {
                    return rec.status == 0;
                };
                if user.has_role("admin") {
                    return rec.status == 0;
                }
                let map: serde_json::Map<String, serde_json::Value> =
                    serde_json::from_str(&rec.payload_json).unwrap_or_default();
                let mut any_pending = false;
                let checks: &[(&str, &str)] = &[
                    ("reception", "reception"),
                    ("tiensu", "history"),
                    ("bs_theluc", "physical"),
                    ("bs_noi", "clin_noi"),
                    ("bs_ngoai", "clin_ngoai"),
                    ("bs_dalieu", "clin_dalieu"),
                    ("bs_san", "clin_san"),
                    ("bs_mat", "clin_mat"),
                    ("bs_tmh", "clin_tmh"),
                    ("bs_rhm", "clin_rhm"),
                    ("bs_cls", "paraclinical"),
                    ("cls", "paraclinical"),
                    ("xetnghiem", "paraclinical"),
                    ("cdha", "paraclinical"),
                    ("bs_ketluan", "conclusion"),
                ];
                for (role, sec) in checks {
                    if user.has_role(role)
                        && super::section_progress_from_map(&map, sec)
                            != super::SectionProgress::Done
                    {
                        any_pending = true;
                        break;
                    }
                }
                any_pending
            });
        }
        let selected_id = self.selected_record_id;
        let is_group_mode = self.sidebar_exam_mode == "doan";
        let selected_contract_str = self
            .sidebar_contract_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let mut contract_opts: Vec<(String, String)> = vec![("".into(), "-- Chọn đoàn khám --".into())];
        for c in &self.contracts_list {
            let st_txt = if c.status == 1 { " [Đã đóng]" } else { "" };
            contract_opts.push((
                c.id.to_string(),
                format!("{} - {} ({} HS){}", c.contract_code, c.company_name, c.total_records, st_txt),
            ));
        }

        div()
            .w(px(310.0))
            .h_full()
            .bg(rgb(bg_card()))
            .border_r_1()
            .border_color(rgb(border_col()))
            .flex()
            .flex_col()
            .child(
                // Sidebar header
                div()
                    .px(px(12.0))
                    .py(px(10.0))
                    .border_b_1()
                    .border_color(rgb(border_col()))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(text_main()))
                            .child(format!("Danh sách hồ sơ ({})", records.len())),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(4.0))
                            .child(
                                Button::new("btn-refresh-sidebar")
                                    .label("↻")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.refresh_records(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-collapse-sidebar")
                                    .label("«")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sidebar_open = false;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            .child(
                // Filters
                div()
                    .p(px(10.0))
                    .border_b_1()
                    .border_color(rgb(border_col()))
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        // Mode switcher: Khám lẻ / Khám đoàn
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(6.0))
                            .child(
                                Button::new("btn-mode-le")
                                    .label("Khám lẻ")
                                    .small()
                                    .when(!is_group_mode, |b| b.primary())
                                    .when(is_group_mode, |b| b.secondary())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.sidebar_exam_mode = "le".into();
                                        this.selected_record_id = None;
                                        this.is_creating_new = false;
                                        this.refresh_records(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-mode-doan")
                                    .label("Khám đoàn")
                                    .small()
                                    .when(is_group_mode, |b| b.primary())
                                    .when(!is_group_mode, |b| b.secondary())
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.sidebar_exam_mode = "doan".into();
                                        this.selected_record_id = None;
                                        this.is_creating_new = false;
                                        this.refresh_group_data(cx);
                                        this.refresh_records(window, cx);
                                    })),
                            ),
                    )
                    .when(!is_group_mode && can_create, |el| {
                        el.child(
                            Button::new("btn-new-patient")
                                .label("+ Tiếp nhận hồ sơ mới")
                                .small()
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.guard_unsaved(super::PendingNav::NewPatient, window, cx);
                                })),
                        )
                    })
                    .when(is_group_mode, |el| {
                        el.child(self.render_select_dropdown(
                            "sidebar-contract-select",
                            "Đoàn khám / Hợp đồng",
                            &selected_contract_str,
                            contract_opts,
                            true,
                            |this, val, window, cx| {
                                this.sidebar_contract_id = val.parse::<i64>().ok();
                                this.selected_record_id = None;
                                this.refresh_records(window, cx);
                            },
                            cx,
                        ))
                    })
                    .child(
                        Input::new(&self.sidebar_search)
                            .small()
                            .cleanable(true),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(6.0))
                            .child(self.render_date_picker_entity(
                                "sb-from",
                                "Từ ngày",
                                self.sidebar_from_date.clone(),
                                cx,
                            ))
                            .child(self.render_date_picker_entity(
                                "sb-to",
                                "Đến ngày",
                                self.sidebar_to_date.clone(),
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap(px(6.0))
                            .child(self.render_select_dropdown(
                                "sidebar-status",
                                "Trạng thái",
                                &status_val,
                                status_opts,
                                true,
                                |this, val, window, cx| {
                                    this.sidebar_status_filter = val.parse::<i64>().ok();
                                    this.refresh_records(window, cx);
                                },
                                cx,
                            ))
                            .child(
                                Button::new("btn-today-filter")
                                    .label("Hôm nay")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        let today =
                                            chrono::Local::now().format("%d/%m/%Y").to_string();
                                        this.sidebar_from_date.update(cx, |s, cx| {
                                            s.set_value(today.clone(), window, cx);
                                        });
                                        this.sidebar_to_date.update(cx, |s, cx| {
                                            s.set_value(today, window, cx);
                                        });
                                        this.refresh_records(window, cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child({
                                let active = self.sidebar_only_my_pending;
                                let btn = Button::new("btn-my-pending-filter")
                                    .label(if active {
                                        "✓ Chờ tôi khám"
                                    } else {
                                        "Chờ tôi khám"
                                    })
                                    .small();
                                if active { btn.primary() } else { btn.secondary() }
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.sidebar_only_my_pending = !this.sidebar_only_my_pending;
                                        cx.notify();
                                    }))
                            }),
                    ),
            )
            .child(
                // Patient list
                div()
                    .id("sidebar-patient-scroll")
                    .flex_1()
                    .overflow_y_scroll()
                    .p(px(8.0))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .when(records.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(16.0))
                                .text_sm()
                                .text_color(rgb(text_muted()))
                                .child(if is_group_mode && self.sidebar_contract_id.is_none() {
                                    "Vui lòng chọn một đoàn khám từ danh sách trên."
                                } else {
                                    "Không có hồ sơ phù hợp bộ lọc."
                                }),
                        )
                    })
                    .children(records.into_iter().map(|rec| {
                        let is_sel = selected_id == Some(rec.id);
                        let (st_lbl, st_bg, st_fg) = match rec.status {
                            2 => ("Đã ký", DANGER_BG, DANGER_COL),
                            1 => ("Hoàn tất", SUCCESS_BG, SUCCESS_COL),
                            _ => ("Đang khám", WARN_BG, WARN_COL),
                        };
                        let name = if rec.ho_ten.is_empty() {
                            "(Chưa có tên)".to_string()
                        } else {
                            rec.ho_ten.clone()
                        };
                        let ma_bn = rec.ma_y_te.clone();
                        let ngay = super::format_ngay_vao_display(&rec.ngay_vao);
                        let workplace = super::payload_field(&rec, "NOI_LAM_VIEC_HOC_TAP");
                        let (done_cnt, total_cnt, missing_secs) = super::record_progress_summary(&rec);
                        let missing_text = if missing_secs.is_empty() {
                            "Đủ các mục".to_string()
                        } else {
                            let preview: Vec<&str> = missing_secs.iter().copied().take(2).collect();
                            if missing_secs.len() > 2 {
                                format!("Thiếu: {} +{}", preview.join(", "), missing_secs.len() - 2)
                            } else {
                                format!("Thiếu: {}", preview.join(", "))
                            }
                        };
                        let rec_id = rec.id;

                        div()
                            .id(SharedString::from(format!("rec-item-{}", rec_id)))
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .border_1()
                            .border_color(rgb(if is_sel { PRIMARY_COL } else { border_col() }))
                            .bg(rgb(if is_sel { primary_light() } else { bg_card() }))
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(if is_sel { primary_light() } else { bg_subtle() })))
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.guard_unsaved(super::PendingNav::Record(rec_id), window, cx);
                            }))
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(6.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .text_color(rgb(text_main()))
                                            .truncate()
                                            .child(name),
                                    )
                                    .child(badge(st_lbl, st_bg, st_fg)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .text_xs()
                                    .text_color(rgb(text_muted()))
                                    .child(format!("Mã: {}", ma_bn))
                                    .child(ngay),
                            )
                            .when(!workplace.is_empty(), |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(text_muted()))
                                        .truncate()
                                        .child(workplace),
                                )
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .justify_between()
                                    .gap(px(6.0))
                                    .text_xs()
                                    .text_color(rgb(if missing_secs.is_empty() {
                                        SUCCESS_COL
                                    } else {
                                        text_muted()
                                    }))
                                    .child(format!("{}/{} mục", done_cnt, total_cnt))
                                    .child(div().truncate().child(missing_text)),
                            )
                    })),
            )
            .into_any_element()
    }
}
