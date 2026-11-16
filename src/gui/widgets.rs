use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{Datelike, NaiveDate};
use gpui::{
    AnyElement, Context, Entity, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use gpui_component::{
    Sizable,
    button::{Button, ButtonVariants},
    input::{Input, InputState, Textarea},
};

use super::KskGpuiApp;

static DARK_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_dark_mode(dark: bool) {
    DARK_MODE.store(dark, Ordering::Relaxed);
}

pub fn is_dark_mode() -> bool {
    DARK_MODE.load(Ordering::Relaxed)
}

pub fn bg_app() -> u32 {
    if is_dark_mode() { 0x0f172a } else { 0xf1f5f9 }
}

pub fn bg_card() -> u32 {
    if is_dark_mode() { 0x1e293b } else { 0xffffff }
}

pub fn bg_subtle() -> u32 {
    if is_dark_mode() { 0x263244 } else { 0xf8fafc }
}

pub fn border_col() -> u32 {
    if is_dark_mode() { 0x334155 } else { 0xe2e8f0 }
}

pub fn text_main() -> u32 {
    if is_dark_mode() { 0xf1f5f9 } else { 0x0f172a }
}

pub fn text_muted() -> u32 {
    if is_dark_mode() { 0x94a3b8 } else { 0x64748b }
}

pub fn primary_light() -> u32 {
    if is_dark_mode() { 0x134e4a } else { 0xccfbf1 }
}

pub const PRIMARY_COL: u32 = 0x0f766e;
pub const DANGER_COL: u32 = 0xbe123c;
pub const DANGER_BG: u32 = 0xffe4e6;
pub const SUCCESS_COL: u32 = 0x15803d;
pub const SUCCESS_BG: u32 = 0xdcfce7;
pub const WARN_COL: u32 = 0xb45309;
pub const WARN_BG: u32 = 0xfef3c7;

pub fn card_box() -> gpui::Div {
    div()
        .bg(rgb(bg_card()))
        .border_1()
        .border_color(rgb(border_col()))
        .rounded(px(8.0))
        .p(px(16.0))
        .flex()
        .flex_col()
        .gap(px(12.0))
}

pub fn badge(label: impl Into<SharedString>, bg_hex: u32, fg_hex: u32) -> AnyElement {
    div()
        .px(px(8.0))
        .py(px(2.0))
        .rounded(px(999.0))
        .bg(rgb(bg_hex))
        .text_color(rgb(fg_hex))
        .text_xs()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(label.into())
        .into_any_element()
}

pub fn role_label_vi(role: &str) -> &'static str {
    match role {
        "admin" => "Quản trị viên",
        "business" => "Phòng kinh doanh",
        "reception" => "Tiếp nhận",
        "tiensu" => "Tiền sử",
        "bs_theluc" => "BS Thể lực",
        "bs_noi" => "Bác sĩ Nội",
        "bs_ngoai" => "Bác sĩ Ngoại",
        "bs_san" => "Bác sĩ Sản",
        "bs_mat" => "Bác sĩ Mắt",
        "bs_tmh" => "Bác sĩ TMH",
        "bs_rhm" => "Bác sĩ RHM",
        "bs_dalieu" => "Bác sĩ Da liễu",
        "bs_cls" => "Cận lâm sàng",
        "bs_ketluan" => "BS Kết luận",
        _ => "Nhân viên",
    }
}

pub fn field_label(text: impl Into<SharedString>, required: bool) -> AnyElement {
    let label: SharedString = text.into();
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_row()
        .items_start()
        .gap(px(3.0))
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(text_muted()))
        .child(div().min_w_0().flex_1().child(label))
        .when(required, |el| {
            el.child(div().flex_shrink_0().text_color(rgb(DANGER_COL)).child("*"))
        })
        .into_any_element()
}

pub fn section_group_header(icon: &str, title: &str) -> AnyElement {
    div()
        .pt(px(6.0))
        .pb(px(4.0))
        .border_b_1()
        .border_color(rgb(border_col()))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(
            div()
                .px(px(6.0))
                .py(px(2.0))
                .rounded(px(6.0))
                .bg(rgb(primary_light()))
                .text_color(rgb(PRIMARY_COL))
                .text_xs()
                .font_weight(gpui::FontWeight::BOLD)
                .child(icon.to_string()),
        )
        .child(
            div()
                .text_sm()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(text_main()))
                .child(title.to_string()),
        )
        .into_any_element()
}

pub fn bmi_status_badge(bmi_str: &str) -> Option<AnyElement> {
    let val = bmi_str.trim().replace(',', ".").parse::<f64>().ok()?;
    if val < 10.0 || val > 80.0 {
        return None;
    }
    let (lbl, bg, fg) = if val < 18.5 {
        ("BMI: Thiếu cân (<18.5)", WARN_BG, WARN_COL)
    } else if val < 25.0 {
        ("BMI: Bình thường (18.5–24.9)", SUCCESS_BG, SUCCESS_COL)
    } else if val < 30.0 {
        ("BMI: Thừa cân (25–29.9)", WARN_BG, WARN_COL)
    } else {
        ("BMI: Béo phì (≥30)", DANGER_BG, DANGER_COL)
    };
    Some(badge(lbl, bg, fg))
}

pub fn bp_status_badge(bp_str: &str) -> Option<AnyElement> {
    let mut parts = bp_str.trim().split('/');
    let sys = parts.next()?.trim().parse::<i64>().ok()?;
    let dia = parts.next()?.trim().parse::<i64>().ok()?;
    if sys < 50 || dia < 30 {
        return None;
    }
    let (lbl, bg, fg) = if sys >= 140 || dia >= 90 {
        ("HA: Tăng huyết áp (≥140/90)", DANGER_BG, DANGER_COL)
    } else if sys >= 130 || dia >= 85 {
        ("HA: Tiền tăng HA (130–139/85–89)", WARN_BG, WARN_COL)
    } else if sys < 90 || dia < 60 {
        ("HA: Huyết áp thấp (<90/60)", WARN_BG, WARN_COL)
    } else {
        ("HA: Bình thường", SUCCESS_BG, SUCCESS_COL)
    };
    Some(badge(lbl, bg, fg))
}

impl KskGpuiApp {
    pub fn render_form_input(
        &mut self,
        key: &str,
        label: &str,
        placeholder: &str,
        required: bool,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.get_or_create_form_input(key, placeholder, false, window, cx);
        if editable && !self.registered_tab_inputs.iter().any(|k| k == key) {
            self.registered_tab_inputs.push(key.to_string());
        }
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_w(px(200.0))
            .flex_1()
            .overflow_hidden()
            .child(field_label(label.to_string(), required))
            .child(
                Input::new(&state)
                    .small()
                    .disabled(!editable)
                    .cleanable(editable),
            )
            .into_any_element()
    }

    pub fn render_form_textarea(
        &mut self,
        key: &str,
        label: &str,
        placeholder: &str,
        rows: usize,
        editable: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let state = self.get_or_create_form_textarea(key, placeholder, rows, window, cx);
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_w(px(260.0))
            .flex_1()
            .child(field_label(label.to_string(), false))
            .child(Textarea::new(&state).small().disabled(!editable))
            .into_any_element()
    }

    pub fn render_pl_radio_group(
        &mut self,
        id_prefix: &str,
        field_key: &str,
        label: &str,
        editable: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let current_val = self.get_form_val(field_key);
        let display_val = if current_val.is_empty() {
            "1".to_string()
        } else {
            current_val
        };
        let opts = [
            ("1", "Loại 1"),
            ("2", "Loại 2"),
            ("3", "Loại 3"),
            ("4", "Loại 4"),
            ("5", "Loại 5"),
        ];
        let fk = field_key.to_string();
        let prefix = id_prefix.to_string();

        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .when(!label.is_empty(), |el| {
                el.child(field_label(label.to_string(), false))
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(6.0))
                    .children(opts.iter().map(|(val, lbl)| {
                        let is_sel = display_val == *val;
                        let v_owned = (*val).to_string();
                        let fk_c = fk.clone();
                        div()
                            .id(SharedString::from(format!("rad-{}-{}", prefix, val)))
                            .h(px(28.0))
                            .px(px(10.0))
                            .rounded(px(999.0))
                            .border_1()
                            .border_color(rgb(if is_sel { PRIMARY_COL } else { border_col() }))
                            .bg(rgb(if is_sel {
                                primary_light()
                            } else if editable {
                                bg_card()
                            } else {
                                bg_subtle()
                            }))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(5.0))
                            .text_xs()
                            .font_weight(if is_sel {
                                gpui::FontWeight::SEMIBOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .text_color(rgb(if is_sel {
                                PRIMARY_COL
                            } else if editable {
                                text_main()
                            } else {
                                text_muted()
                            }))
                            .when(editable, |el| {
                                el.cursor_pointer().on_click(cx.listener(move |this, _, _, cx| {
                                    if this.get_form_val(&fk_c) == v_owned {
                                        this.form_values.insert(fk_c.clone(), String::new());
                                    } else {
                                        this.form_values.insert(fk_c.clone(), v_owned.clone());
                                    }
                                    if this.editing_section.is_some() {
                                        this.is_dirty = true;
                                    }
                                    cx.notify();
                                }))
                            })
                            .child(
                                div()
                                    .w(px(12.0))
                                    .h(px(12.0))
                                    .rounded(px(999.0))
                                    .border_1()
                                    .border_color(rgb(if is_sel { PRIMARY_COL } else { text_muted() }))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .when(is_sel, |el| {
                                        el.child(
                                            div()
                                                .w(px(6.0))
                                                .h(px(6.0))
                                                .rounded(px(999.0))
                                                .bg(rgb(PRIMARY_COL)),
                                        )
                                    }),
                            )
                            .child(*lbl)
                    })),
            )
            .into_any_element()
    }

    pub fn render_select_dropdown(
        &mut self,
        dropdown_id: &str,
        label: &str,
        current_val: &str,
        options: Vec<(String, String)>,
        editable: bool,
        on_select: impl Fn(&mut Self, String, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if !options.is_empty() && options.len() < 4 {
            let on_select_rc = std::rc::Rc::new(on_select);
            let id_owned = dropdown_id.to_string();
            return div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .min_w(px(200.0))
                .flex_1()
                .overflow_hidden()
                .when(!label.is_empty(), |el| {
                    el.child(field_label(label.to_string(), false))
                })
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(6.0))
                        .children(
                            options
                                .into_iter()
                                .filter(|(v, _)| !v.is_empty())
                                .enumerate()
                                .map(|(idx, (val, lbl))| {
                                    let is_sel = val == current_val;
                                    let cb = on_select_rc.clone();
                                    let val_c = val.clone();
                                    div()
                                        .id(SharedString::from(format!("dd-rad-{}-{}", id_owned, idx)))
                                        .px(px(10.0))
                                        .py(px(5.0))
                                        .rounded(px(6.0))
                                        .border_1()
                                        .border_color(rgb(if is_sel { PRIMARY_COL } else { border_col() }))
                                        .bg(rgb(if is_sel {
                                            primary_light()
                                        } else if editable {
                                            bg_card()
                                        } else {
                                            bg_subtle()
                                        }))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(5.0))
                                        .text_xs()
                                        .font_weight(if is_sel {
                                            gpui::FontWeight::SEMIBOLD
                                        } else {
                                            gpui::FontWeight::NORMAL
                                        })
                                        .text_color(rgb(if is_sel {
                                            PRIMARY_COL
                                        } else if editable {
                                            text_main()
                                        } else {
                                            text_muted()
                                        }))
                                        .when(editable, |el| {
                                            el.cursor_pointer().on_click(cx.listener(
                                                move |this, _, window, cx| {
                                                    if this.editing_section.is_some() {
                                                        this.is_dirty = true;
                                                    }
                                                    cb(this, val_c.clone(), window, cx);
                                                    cx.notify();
                                                },
                                            ))
                                        })
                                        .child(
                                            div()
                                                .w(px(12.0))
                                                .h(px(12.0))
                                                .rounded(px(999.0))
                                                .border_1()
                                                .border_color(rgb(if is_sel {
                                                    PRIMARY_COL
                                                } else {
                                                    text_muted()
                                                }))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .when(is_sel, |el| {
                                                    el.child(
                                                        div()
                                                            .w(px(6.0))
                                                            .h(px(6.0))
                                                            .rounded(px(999.0))
                                                            .bg(rgb(PRIMARY_COL)),
                                                    )
                                                }),
                                        )
                                        .child(lbl)
                                }),
                        ),
                )
                .into_any_element();
        }

        let is_open = self.open_dropdown.as_deref() == Some(dropdown_id);
        let display_text = options
            .iter()
            .find(|(v, _)| v == current_val)
            .map(|(_, l)| l.clone())
            .unwrap_or_else(|| {
                if current_val.is_empty() {
                    "-- Chọn --".to_string()
                } else {
                    current_val.to_string()
                }
            });

        let id_owned = dropdown_id.to_string();
        let on_select_rc = std::rc::Rc::new(on_select);
        let needs_search = options.len() > 10;

        div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_w(px(160.0))
            .flex_1()
            .when(!label.is_empty(), |el| {
                el.child(field_label(label.to_string(), false))
            })
            .child(
                div()
                    .id(SharedString::from(format!("dd-btn-{}", id_owned)))
                    .h(px(30.0))
                    .px(px(10.0))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(rgb(if is_open { PRIMARY_COL } else { border_col() }))
                    .bg(rgb(if editable { bg_card() } else { bg_subtle() }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .text_sm()
                    .text_color(rgb(if editable { text_main() } else { text_muted() }))
                    .when(editable, |el| {
                        let id_toggle = id_owned.clone();
                        el.cursor_pointer().on_click(cx.listener(move |this, _, window, cx| {
                            if this.open_dropdown.as_deref() == Some(&id_toggle) {
                                this.open_dropdown = None;
                            } else {
                                this.open_dropdown = Some(id_toggle.clone());
                                this.dropdown_search_input
                                    .update(cx, |s, cx| s.set_value("", window, cx));
                            }
                            cx.notify();
                        }))
                    })
                    .child(div().truncate().child(display_text))
                    .child(div().text_xs().text_color(rgb(text_muted())).child("▾")),
            )
            .when(is_open && editable, |el| {
                let q = if needs_search {
                    self.dropdown_search_input.read(cx).value().trim().to_lowercase()
                } else {
                    String::new()
                };
                let filtered: Vec<(String, String)> = options
                    .into_iter()
                    .filter(|(val, lbl)| {
                        q.is_empty()
                            || val.to_lowercase().contains(&q)
                            || lbl.to_lowercase().contains(&q)
                    })
                    .collect();
                let total_matched = filtered.len();
                let shown: Vec<(String, String)> = filtered.into_iter().take(100).collect();

                let menu = div()
                    .id(SharedString::from(format!("dd-menu-{}", id_owned)))
                    .absolute()
                    .top(px(if label.is_empty() { 34.0 } else { 54.0 }))
                    .left_0()
                    .min_w(px(260.0))
                    .w_full()
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(PRIMARY_COL))
                    .rounded(px(6.0))
                    .shadow_lg()
                    .p(px(4.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.open_dropdown = None;
                        cx.notify();
                    }))
                    .when(needs_search, |m| {
                        m.child(
                            div()
                                .p(px(2.0))
                                .child(Input::new(&self.dropdown_search_input).small().cleanable(true)),
                        )
                    })
                    .child(
                        div()
                            .id(SharedString::from(format!("dd-list-{}", id_owned)))
                            .max_h(px(240.0))
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .when(shown.is_empty(), |l| {
                                l.child(
                                    div()
                                        .px(px(8.0))
                                        .py(px(6.0))
                                        .text_xs()
                                        .text_color(rgb(text_muted()))
                                        .child("Không tìm thấy mục phù hợp"),
                                )
                            })
                            .children(shown.into_iter().enumerate().map(|(idx, (val, lbl))| {
                                let is_sel = val == current_val;
                                let cb = on_select_rc.clone();
                                let val_c = val.clone();
                                div()
                                    .id(SharedString::from(format!("dd-opt-{}-{}", id_owned, idx)))
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .text_sm()
                                    .cursor_pointer()
                                    .bg(rgb(if is_sel { primary_light() } else { bg_card() }))
                                    .text_color(rgb(if is_sel { PRIMARY_COL } else { text_main() }))
                                    .hover(|s| s.bg(rgb(bg_subtle())))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_dropdown = None;
                                        if this.editing_section.is_some() {
                                            this.is_dirty = true;
                                        }
                                        cb(this, val_c.clone(), window, cx);
                                        cx.notify();
                                    }))
                                    .child(lbl)
                            })),
                    )
                    .when(total_matched > 100, |m| {
                        m.child(
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .text_xs()
                                .text_color(rgb(text_muted()))
                                .child(format!("Hiển thị 100/{} mục — nhập thêm để lọc", total_matched)),
                        )
                    });
                el.child(gpui::deferred(menu).with_priority(100))
            })
            .into_any_element()
    }

    pub fn render_section_action_bar(
        &self,
        _section_key: &str,
        title: &str,
        can_edit_role: bool,
        is_locked: bool,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pb(px(8.0))
            .border_b_1()
            .border_color(rgb(border_col()))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(text_main()))
                            .child(title.to_string()),
                    )
                    .when(is_locked, |el| {
                        el.child(badge("Đã ký khóa", DANGER_BG, DANGER_COL))
                    })
                    .when(!is_locked && !can_edit_role, |el| {
                        el.child(badge("Chỉ xem", bg_subtle(), text_muted()))
                    }),
            )
            .into_any_element()
    }

    pub fn render_icd_multi_select(
        &mut self,
        dropdown_id: &str,
        field_key: &str,
        label: &str,
        editable: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let raw_val = self.get_form_val(field_key);
        let mut selected: Vec<String> = raw_val
            .split(|c| c == ',' || c == ';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        selected.dedup();

        let options = self.query_catalog_options("ICD10", None);
        let is_open = self.open_dropdown.as_deref() == Some(dropdown_id);
        let id_owned = dropdown_id.to_string();
        let fk_owned = field_key.to_string();

        let label_for = |code: &str| -> String {
            options
                .iter()
                .find(|(c, _)| c == code)
                .map(|(_, l)| l.clone())
                .unwrap_or_else(|| code.to_string())
        };

        div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_w(px(260.0))
            .flex_1()
            .when(!label.is_empty(), |el| {
                el.child(field_label(label.to_string(), false))
            })
            .when(!selected.is_empty(), |el| {
                let fk_rm = fk_owned.clone();
                let id_rm = id_owned.clone();
                el.child(
                    div()
                        .w_full()
                        .flex()
                        .flex_row()
                        .flex_wrap()
                        .gap(px(4.0))
                        .children(selected.iter().enumerate().map(|(i, code)| {
                            let code_rm = code.clone();
                            let fk_c = fk_rm.clone();
                            let full_lbl = label_for(code);
                            div()
                                .max_w_full()
                                .overflow_hidden()
                                .px(px(8.0))
                                .py(px(3.0))
                                .rounded(px(999.0))
                                .bg(rgb(primary_light()))
                                .border_1()
                                .border_color(rgb(PRIMARY_COL))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap(px(4.0))
                                .text_xs()
                                .text_color(rgb(PRIMARY_COL))
                                .child(div().min_w_0().overflow_hidden().child(full_lbl))
                                .when(editable, |chip| {
                                    chip.child(
                                        div()
                                            .id(SharedString::from(format!("icd-rm-{}-{}", id_rm, i)))
                                            .cursor_pointer()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                let cur = this.get_form_val(&fk_c);
                                                let next: Vec<String> = cur
                                                    .split(|c| c == ',' || c == ';')
                                                    .map(|s| s.trim().to_string())
                                                    .filter(|s| !s.is_empty() && s != &code_rm)
                                                    .collect();
                                                let joined = next.join(";");
                                                if this.editing_section.is_some() {
                                                    this.is_dirty = true;
                                                }
                                                this.form_values.insert(fk_c.clone(), joined.clone());
                                                if let Some(inp) = this.form_inputs.get(&fk_c) {
                                                    inp.update(cx, |s, cx| s.set_value(joined, window, cx));
                                                }
                                                cx.notify();
                                            }))
                                            .child("×"),
                                    )
                                })
                        })),
                )
            })
            .child(
                div()
                    .id(SharedString::from(format!("dd-btn-{}", id_owned)))
                    .h(px(30.0))
                    .px(px(10.0))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(rgb(if is_open { PRIMARY_COL } else { border_col() }))
                    .bg(rgb(if editable { bg_card() } else { bg_subtle() }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .text_sm()
                    .text_color(rgb(if editable { text_main() } else { text_muted() }))
                    .when(editable, |el| {
                        let id_toggle = id_owned.clone();
                        el.cursor_pointer().on_click(cx.listener(move |this, _, window, cx| {
                            if this.open_dropdown.as_deref() == Some(&id_toggle) {
                                this.open_dropdown = None;
                            } else {
                                this.open_dropdown = Some(id_toggle.clone());
                                this.dropdown_search_input
                                    .update(cx, |s, cx| s.set_value("", window, cx));
                            }
                            cx.notify();
                        }))
                    })
                    .child(
                        div().truncate().child(if selected.is_empty() {
                            "-- Chọn một hoặc nhiều mã ICD-10 --".to_string()
                        } else {
                            format!("Đã chọn {} mã ICD-10 (+ thêm / sửa)", selected.len())
                        }),
                    )
                    .child(div().text_xs().text_color(rgb(text_muted())).child("▾")),
            )
            .when(is_open && editable, |el| {
                let q = self.dropdown_search_input.read(cx).value().trim().to_lowercase();
                let filtered: Vec<(String, String)> = options
                    .into_iter()
                    .filter(|(val, lbl)| {
                        q.is_empty()
                            || val.to_lowercase().contains(&q)
                            || lbl.to_lowercase().contains(&q)
                    })
                    .collect();
                let total_matched = filtered.len();
                let shown: Vec<(String, String)> = filtered.into_iter().take(100).collect();

                let menu = div()
                    .id(SharedString::from(format!("dd-menu-{}", id_owned)))
                    .absolute()
                    .bottom_auto()
                    .left_0()
                    .min_w(px(320.0))
                    .w_full()
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(PRIMARY_COL))
                    .rounded(px(6.0))
                    .shadow_lg()
                    .p(px(4.0))
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.open_dropdown = None;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .p(px(2.0))
                            .child(Input::new(&self.dropdown_search_input).small().cleanable(true)),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("dd-list-{}", id_owned)))
                            .max_h(px(240.0))
                            .overflow_y_scroll()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .when(shown.is_empty(), |l| {
                                l.child(
                                    div()
                                        .px(px(8.0))
                                        .py(px(6.0))
                                        .text_xs()
                                        .text_color(rgb(text_muted()))
                                        .child("Không tìm thấy mã ICD-10 phù hợp"),
                                )
                            })
                            .children(shown.into_iter().enumerate().map(|(idx, (val, lbl))| {
                                let is_sel = selected.iter().any(|s| s == &val);
                                let val_c = val.clone();
                                let fk_c = fk_owned.clone();
                                div()
                                    .id(SharedString::from(format!("dd-opt-{}-{}", id_owned, idx)))
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .text_sm()
                                    .cursor_pointer()
                                    .bg(rgb(if is_sel { primary_light() } else { bg_card() }))
                                    .text_color(rgb(if is_sel { PRIMARY_COL } else { text_main() }))
                                    .hover(|s| s.bg(rgb(bg_subtle())))
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let cur = this.get_form_val(&fk_c);
                                        let mut list: Vec<String> = cur
                                            .split(|c| c == ',' || c == ';')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        if let Some(pos) = list.iter().position(|x| x == &val_c) {
                                            list.remove(pos);
                                        } else {
                                            list.push(val_c.clone());
                                        }
                                        let joined = list.join(";");
                                        if this.editing_section.is_some() {
                                            this.is_dirty = true;
                                        }
                                        this.form_values.insert(fk_c.clone(), joined.clone());
                                        if let Some(inp) = this.form_inputs.get(&fk_c) {
                                            inp.update(cx, |s, cx| s.set_value(joined, window, cx));
                                        }
                                        cx.notify();
                                    }))
                                    .child(if is_sel { "☑" } else { "☐" })
                                    .child(lbl)
                            })),
                    )
                    .when(total_matched > 100, |m| {
                        m.child(
                            div()
                                .px(px(8.0))
                                .py(px(2.0))
                                .text_xs()
                                .text_color(rgb(text_muted()))
                                .child(format!("Hiển thị 100/{} mã — nhập thêm để lọc", total_matched)),
                        )
                    });
                el.child(gpui::deferred(menu).with_priority(100))
            })
            .into_any_element()
    }

    pub fn render_date_picker_form(
        &mut self,
        picker_id: &str,
        field_key: &str,
        label: &str,
        placeholder: &str,
        required: bool,
        editable: bool,
        _include_hhmm: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let inp = self.get_or_create_form_input(field_key, placeholder, false, window, cx);
        let cur_val = self.get_form_val(field_key);
        let fk_owned = field_key.to_string();
        let inp_for_cb = inp.clone();
        self.render_date_picker_inner(
            picker_id,
            label,
            &cur_val,
            inp,
            editable,
            required,
            DateStoreFmt::DmySlash,
            move |this, formatted, window, cx| {
                this.form_values.insert(fk_owned.clone(), formatted.clone());
                if this.editing_section.is_some() {
                    this.is_dirty = true;
                }
                inp_for_cb.update(cx, |s, cx| s.set_value(formatted, window, cx));
            },
            cx,
        )
    }

    pub fn render_date_picker_entity(
        &mut self,
        picker_id: &str,
        label: &str,
        inp: Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let cur_val = inp.read(cx).value().to_string();
        let inp_for_cb = inp.clone();
        self.render_date_picker_inner(
            picker_id,
            label,
            &cur_val,
            inp,
            true,
            false,
            DateStoreFmt::DmySlash,
            move |_this, formatted, window, cx| {
                inp_for_cb.update(cx, |s, cx| s.set_value(formatted, window, cx));
            },
            cx,
        )
    }

    fn render_date_picker_inner(
        &mut self,
        picker_id: &str,
        label: &str,
        cur_val: &str,
        inp: Entity<InputState>,
        editable: bool,
        required: bool,
        fmt: DateStoreFmt,
        on_pick: impl Fn(&mut Self, String, &mut Window, &mut Context<Self>) + 'static,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let dd_id = format!("dp-{}", picker_id);
        let is_open = self.open_dropdown.as_deref() == Some(dd_id.as_str());
        let selected_date = parse_any_date(cur_val);
        let today = chrono::Local::now().date_naive();
        let on_pick_rc = std::rc::Rc::new(on_pick);
        let cur_owned = cur_val.to_string();
        let dd_id_toggle = dd_id.clone();

        div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_w(px(105.0))
            .flex_1()
            .when(!label.is_empty(), |el| {
                el.child(field_label(label.to_string(), required))
            })
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(4.0))
                    .child(div().flex_1().child(Input::new(&inp).small().disabled(!editable)))
                    .when(editable, |row| {
                        let init_d = selected_date.unwrap_or(today);
                        row.child(
                            Button::new(SharedString::from(format!("dp-btn-{}", picker_id)))
                                .label("📅")
                                .small()
                                .secondary()
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if this.open_dropdown.as_deref() == Some(dd_id_toggle.as_str()) {
                                        this.open_dropdown = None;
                                    } else {
                                        this.date_picker_view_ym = (init_d.year(), init_d.month());
                                        this.open_dropdown = Some(dd_id_toggle.clone());
                                    }
                                    cx.notify();
                                })),
                        )
                    }),
            )
            .when(is_open && editable, |el| {
                let (vy, vm) = self.date_picker_view_ym;
                let first_day = NaiveDate::from_ymd_opt(vy, vm, 1).unwrap_or(today);
                let days_in_month = days_in_ym(vy, vm);
                let start_offset = first_day.weekday().num_days_from_monday() as usize;
                let mut cells: Vec<Option<NaiveDate>> = vec![None; start_offset];
                for day in 1..=days_in_month {
                    cells.push(NaiveDate::from_ymd_opt(vy, vm, day));
                }
                while cells.len() % 7 != 0 {
                    cells.push(None);
                }

                let pid = picker_id.to_string();
                let on_pick_today = on_pick_rc.clone();
                let on_pick_clear = on_pick_rc.clone();
                let cur_for_today = cur_owned.clone();

                let popover = div()
                    .id(SharedString::from(format!("dp-pop-{}", pid)))
                    .absolute()
                    .top(px(if label.is_empty() { 34.0 } else { 54.0 }))
                    .left_0()
                    .w(px(252.0))
                    .bg(rgb(bg_card()))
                    .border_1()
                    .border_color(rgb(PRIMARY_COL))
                    .rounded(px(8.0))
                    .shadow_lg()
                    .p(px(8.0))
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.open_dropdown = None;
                        cx.notify();
                    }))
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
                                    .gap(px(2.0))
                                    .child(
                                        Button::new(SharedString::from(format!("dp-py-{}", pid)))
                                            .label("«")
                                            .xsmall()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.date_picker_view_ym.0 -= 1;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("dp-pm-{}", pid)))
                                            .label("‹")
                                            .xsmall()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let (y, m) = this.date_picker_view_ym;
                                                this.date_picker_view_ym =
                                                    if m == 1 { (y - 1, 12) } else { (y, m - 1) };
                                                cx.notify();
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(text_main()))
                                    .child(format!("Tháng {:02} / {}", vm, vy)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(2.0))
                                    .child(
                                        Button::new(SharedString::from(format!("dp-nm-{}", pid)))
                                            .label("›")
                                            .xsmall()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                let (y, m) = this.date_picker_view_ym;
                                                this.date_picker_view_ym =
                                                    if m == 12 { (y + 1, 1) } else { (y, m + 1) };
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("dp-ny-{}", pid)))
                                            .label("»")
                                            .xsmall()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.date_picker_view_ym.0 += 1;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .children(["T2", "T3", "T4", "T5", "T6", "T7", "CN"].iter().map(|d| {
                                div()
                                    .w(px(32.0))
                                    .text_center()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(text_muted()))
                                    .child(*d)
                            })),
                    )
                    .children(cells.chunks(7).enumerate().map(|(wk_idx, week)| {
                        div()
                            .flex()
                            .flex_row()
                            .children(week.iter().enumerate().map(|(d_idx, opt_date)| {
                                if let Some(d) = opt_date {
                                    let d_val = *d;
                                    let is_sel = selected_date == Some(d_val);
                                    let is_today = d_val == today;
                                    let cb = on_pick_rc.clone();
                                    let cur_c = cur_owned.clone();
                                    div()
                                        .id(SharedString::from(format!(
                                            "dp-d-{}-{}-{}",
                                            pid, wk_idx, d_idx
                                        )))
                                        .w(px(32.0))
                                        .h(px(26.0))
                                        .rounded(px(4.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_xs()
                                        .cursor_pointer()
                                        .bg(rgb(if is_sel {
                                            PRIMARY_COL
                                        } else if is_today {
                                            primary_light()
                                        } else {
                                            bg_card()
                                        }))
                                        .text_color(rgb(if is_sel {
                                            0xffffff
                                        } else if is_today {
                                            PRIMARY_COL
                                        } else {
                                            text_main()
                                        }))
                                        .hover(|s| {
                                            if is_sel {
                                                s
                                            } else {
                                                s.bg(rgb(bg_subtle()))
                                            }
                                        })
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            let formatted = format_picked_date(d_val, fmt, &cur_c);
                                            this.open_dropdown = None;
                                            cb(this, formatted, window, cx);
                                            cx.notify();
                                        }))
                                        .child(format!("{}", d_val.day()))
                                        .into_any_element()
                                } else {
                                    div().w(px(32.0)).h(px(26.0)).into_any_element()
                                }
                            }))
                    }))
                    .child(
                        div()
                            .pt(px(4.0))
                            .border_t_1()
                            .border_color(rgb(border_col()))
                            .flex()
                            .flex_row()
                            .justify_between()
                            .child(
                                Button::new(SharedString::from(format!("dp-td-{}", pid)))
                                    .label("Hôm nay")
                                    .xsmall()
                                    .secondary()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        let formatted =
                                            format_picked_date(today, fmt, &cur_for_today);
                                        this.open_dropdown = None;
                                        on_pick_today(this, formatted, window, cx);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!("dp-clr-{}", pid)))
                                    .label("Xóa")
                                    .xsmall()
                                    .secondary()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_dropdown = None;
                                        on_pick_clear(this, String::new(), window, cx);
                                        cx.notify();
                                    })),
                            ),
                    );
                el.child(gpui::deferred(popover).with_priority(100))
            })
            .into_any_element()
    }
}

#[derive(Clone, Copy)]
enum DateStoreFmt {
    DmySlash,
}

fn days_in_ym(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let first_this = NaiveDate::from_ymd_opt(y, m, 1);
    let first_next = NaiveDate::from_ymd_opt(ny, nm, 1);
    match (first_this, first_next) {
        (Some(a), Some(b)) => (b - a).num_days() as u32,
        _ => 30,
    }
}

fn parse_any_date(raw: &str) -> Option<NaiveDate> {
    let s = raw.trim();
    if s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-') {
        return NaiveDate::parse_from_str(&s[0..10], "%Y-%m-%d").ok();
    }
    if s.len() >= 8 && s.chars().take(8).all(|c| c.is_ascii_digit()) {
        return NaiveDate::parse_from_str(&s[0..8], "%Y%m%d").ok();
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%d/%m/%Y") {
        return Some(d);
    }
    None
}

fn format_picked_date(d: NaiveDate, fmt: DateStoreFmt, _existing: &str) -> String {
    match fmt {
        DateStoreFmt::DmySlash => d.format("%d/%m/%Y").to_string(),
    }
}

pub trait WhenExt: Sized {
    fn when(self, cond: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if cond { f(self) } else { self }
    }
}
impl<T> WhenExt for T {}
