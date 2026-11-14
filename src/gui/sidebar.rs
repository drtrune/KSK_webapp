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

        let records = self.records.clone();
        let selected_id = self.selected_record_id;

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
                    .when(can_create, |el| {
                        el.child(
                            Button::new("btn-new-patient")
                                .label("+ Tiếp nhận hồ sơ mới")
                                .small()
                                .primary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.start_new_patient(window, cx);
                                })),
                        )
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
                                "",
                                self.sidebar_from_date.clone(),
                                cx,
                            ))
                            .child(self.render_date_picker_entity(
                                "sb-to",
                                "",
                                self.sidebar_to_date.clone(),
                                cx,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(self.render_select_dropdown(
                                "sidebar-status",
                                "",
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
                                            chrono::Local::now().format("%Y-%m-%d").to_string();
                                        this.sidebar_from_date.update(cx, |s, cx| {
                                            s.set_value(today.clone(), window, cx);
                                        });
                                        this.sidebar_to_date.update(cx, |s, cx| {
                                            s.set_value(today, window, cx);
                                        });
                                        this.refresh_records(window, cx);
                                    })),
                            ),
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
                                .child("Không có hồ sơ phù hợp bộ lọc."),
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
                        let ngay = rec.ngay_vao.clone();
                        let workplace = super::payload_field(&rec, "NOI_LAM_VIEC_HOC_TAP");
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
                                this.load_record_into_form(rec_id, window, cx);
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
                    })),
            )
            .into_any_element()
    }
}
