use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, rgb,
};
use gpui_component::{
    Sizable,
    button::{Button, ButtonVariants},
    input::{Input, Textarea},
};

use super::KskGpuiApp;

pub const BG_APP: u32 = 0xf1f5f9;
pub const BG_CARD: u32 = 0xffffff;
pub const BG_SUBTLE: u32 = 0xf8fafc;
pub const BORDER_COL: u32 = 0xe2e8f0;
pub const PRIMARY_COL: u32 = 0x0f766e;
pub const PRIMARY_LIGHT: u32 = 0xccfbf1;
pub const TEXT_MAIN: u32 = 0x0f172a;
pub const TEXT_MUTED: u32 = 0x64748b;
pub const DANGER_COL: u32 = 0xbe123c;
pub const DANGER_BG: u32 = 0xffe4e6;
pub const SUCCESS_COL: u32 = 0x15803d;
pub const SUCCESS_BG: u32 = 0xdcfce7;
pub const WARN_COL: u32 = 0xb45309;
pub const WARN_BG: u32 = 0xfef3c7;

pub fn card_box() -> gpui::Div {
    div()
        .bg(rgb(BG_CARD))
        .border_1()
        .border_color(rgb(BORDER_COL))
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
        .flex()
        .flex_row()
        .items_center()
        .gap(px(3.0))
        .text_xs()
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(TEXT_MUTED))
        .child(label)
        .when(required, |el| {
            el.child(div().text_color(rgb(DANGER_COL)).child("*"))
        })
        .into_any_element()
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
        div()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .min_w(px(160.0))
            .flex_1()
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
                        let is_sel = current_val == *val;
                        let v_owned = (*val).to_string();
                        let fk_c = fk.clone();
                        div()
                            .id(SharedString::from(format!("rad-{}-{}", prefix, val)))
                            .h(px(28.0))
                            .px(px(10.0))
                            .rounded(px(999.0))
                            .border_1()
                            .border_color(rgb(if is_sel { PRIMARY_COL } else { BORDER_COL }))
                            .bg(rgb(if is_sel {
                                PRIMARY_LIGHT
                            } else if editable {
                                BG_CARD
                            } else {
                                BG_SUBTLE
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
                                TEXT_MAIN
                            } else {
                                TEXT_MUTED
                            }))
                            .when(editable, |el| {
                                el.cursor_pointer().on_click(cx.listener(move |this, _, _, cx| {
                                    if this.get_form_val(&fk_c) == v_owned {
                                        this.form_values.insert(fk_c.clone(), String::new());
                                    } else {
                                        this.form_values.insert(fk_c.clone(), v_owned.clone());
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
                                    .border_color(rgb(if is_sel { PRIMARY_COL } else { TEXT_MUTED }))
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
                    .border_color(rgb(if is_open { PRIMARY_COL } else { BORDER_COL }))
                    .bg(rgb(if editable { BG_CARD } else { BG_SUBTLE }))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .text_sm()
                    .text_color(rgb(if editable { TEXT_MAIN } else { TEXT_MUTED }))
                    .when(editable, |el| {
                        let id_toggle = id_owned.clone();
                        el.cursor_pointer().on_click(cx.listener(move |this, _, _, cx| {
                            if this.open_dropdown.as_deref() == Some(&id_toggle) {
                                this.open_dropdown = None;
                            } else {
                                this.open_dropdown = Some(id_toggle.clone());
                            }
                            cx.notify();
                        }))
                    })
                    .child(div().truncate().child(display_text))
                    .child(div().text_xs().text_color(rgb(TEXT_MUTED)).child("▾")),
            )
            .when(is_open && editable, |el| {
                el.child(
                    div()
                        .id(SharedString::from(format!("dd-menu-{}", id_owned)))
                        .absolute()
                        .top(px(if label.is_empty() { 34.0 } else { 54.0 }))
                        .left_0()
                        .right_0()
                        .max_h(px(220.0))
                        .overflow_y_scroll()
                        .bg(rgb(BG_CARD))
                        .border_1()
                        .border_color(rgb(BORDER_COL))
                        .rounded(px(6.0))
                        .shadow_lg()
                        .p(px(4.0))
                        .flex()
                        .flex_col()
                        .gap(px(2.0))
                        .children(options.into_iter().enumerate().map(|(idx, (val, lbl))| {
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
                                .bg(rgb(if is_sel { PRIMARY_LIGHT } else { BG_CARD }))
                                .text_color(rgb(if is_sel { PRIMARY_COL } else { TEXT_MAIN }))
                                .hover(|s| s.bg(rgb(BG_SUBTLE)))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.open_dropdown = None;
                                    cb(this, val_c.clone(), window, cx);
                                    cx.notify();
                                }))
                                .child(lbl)
                        })),
                )
            })
            .into_any_element()
    }

    pub fn render_section_action_bar(
        &self,
        section_key: &str,
        title: &str,
        can_edit_role: bool,
        is_locked: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_editing = self.editing_section.as_deref() == Some(section_key);
        let sec_owned = section_key.to_string();
        let sec_save = section_key.to_string();

        div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .pb(px(8.0))
            .border_b_1()
            .border_color(rgb(BORDER_COL))
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
                            .text_color(rgb(TEXT_MAIN))
                            .child(title.to_string()),
                    )
                    .when(is_locked, |el| {
                        el.child(badge("Đã ký khóa", DANGER_BG, DANGER_COL))
                    })
                    .when(!is_locked && !can_edit_role, |el| {
                        el.child(badge("Chỉ xem", BG_SUBTLE, TEXT_MUTED))
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(8.0))
                    .when(!is_locked && can_edit_role && !is_editing, |el| {
                        let s = sec_owned.clone();
                        el.child(
                            Button::new(SharedString::from(format!("btn-edit-{}", sec_owned)))
                                .label("Chỉnh sửa")
                                .small()
                                .secondary()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.enter_edit_section(&s, window, cx);
                                })),
                        )
                    })
                    .when(!is_locked && can_edit_role && is_editing, |el| {
                        let s_save = sec_save.clone();
                        el.child(
                            Button::new(SharedString::from(format!("btn-cancel-{}", sec_owned)))
                                .label("Hủy")
                                .small()
                                .secondary()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.editing_section = None;
                                    if let Some(id) = this.selected_record_id {
                                        this.load_record_into_form(id, window, cx);
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new(SharedString::from(format!("btn-save-{}", sec_save)))
                                .label("Lưu phần này")
                                .small()
                                .primary()
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.save_section(&s_save, window, cx);
                                })),
                        )
                    }),
            )
            .into_any_element()
    }
}

pub trait WhenExt: Sized {
    fn when(self, cond: bool, f: impl FnOnce(Self) -> Self) -> Self {
        if cond { f(self) } else { self }
    }
}
impl<T> WhenExt for T {}
