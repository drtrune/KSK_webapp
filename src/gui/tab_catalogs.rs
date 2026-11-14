use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, Styled,
    Window, div, prelude::FluentBuilder as _, px, rgb,
};
use gpui_component::{
    Sizable,
    button::{Button, ButtonVariants},
    input::Input,
    scroll::ScrollableElement as _,
};

use super::{
    KskGpuiApp,
    widgets::{
        BG_CARD, BG_SUBTLE, BORDER_COL, DANGER_BG, DANGER_COL, PRIMARY_COL, PRIMARY_LIGHT,
        SUCCESS_BG, SUCCESS_COL, TEXT_MAIN, TEXT_MUTED, WARN_BG, WARN_COL, badge, card_box,
        field_label,
    },
};

pub fn admin_catalog_types() -> Vec<(String, String)> {
    vec![
        ("TINH_TP".into(), "Tỉnh / Thành phố".into()),
        ("XA_PHUONG".into(), "Phường / Xã".into()),
        ("DAN_TOC".into(), "Dân tộc".into()),
        ("NGHE_NGHIEP".into(), "Nghề nghiệp".into()),
        ("NHOM_MAU".into(), "Nhóm máu".into()),
        ("DOI_TUONG".into(), "Đối tượng khám".into()),
        ("NGUON_CHI_TRA".into(), "Nguồn chi trả".into()),
        ("LOAI_KCB".into(), "Loại hình khám".into()),
        ("NOI_CAP".into(), "Nơi cấp CCCD".into()),
        ("ICD10".into(), "Mã bệnh ICD-10".into()),
    ]
}

pub fn disease_catalog_types() -> Vec<(String, String)> {
    vec![
        ("CHUYEN_KHOA".into(), "1. Chuyên khoa (CHUYEN_KHOA)".into()),
        ("BENH_TAT".into(), "2. Mặt bệnh (BENH_TAT)".into()),
        ("TIEU_CHUAN_PL".into(), "3. Tiêu chuẩn phân loại (TIEU_CHUAN_PL)".into()),
    ]
}

impl KskGpuiApp {
    pub fn render_catalogs_manager(
        &mut self,
        disease_mode: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let type_opts = if disease_mode {
            disease_catalog_types()
        } else {
            admin_catalog_types()
        };

        let cur_type = if disease_mode {
            if matches!(
                self.adm_cat_type.as_str(),
                "CHUYEN_KHOA" | "BENH_TAT" | "TIEU_CHUAN_PL"
            ) {
                self.adm_cat_type.clone()
            } else {
                "BENH_TAT".to_string()
            }
        } else if matches!(
            self.adm_cat_type.as_str(),
            "CHUYEN_KHOA" | "BENH_TAT" | "TIEU_CHUAN_PL"
        ) {
            "TINH_TP".to_string()
        } else {
            self.adm_cat_type.clone()
        };

        let items = self.catalogs.clone();
        let is_editing = self.adm_cat_edit_id.is_some();

        let title = if disease_mode {
            "Module Quản lý Danh mục Phân loại Bệnh tật (Chuyên khoa → Mặt bệnh → Tiêu chuẩn)"
        } else {
            "Quản lý Danh mục Hành chính & Y tế (seed_catalogs.tsv)"
        };

        div()
            .id(if disease_mode {
                "disease-cat-scroll"
            } else {
                "admin-cat-scroll"
            })
            .flex_1()
            .h_full()
            .overflow_y_scrollbar()
            .p(if disease_mode { px(16.0) } else { px(0.0) })
            .flex()
            .flex_col()
            .gap(px(14.0))
            .child(
                card_box()
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
                                    .text_base()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(TEXT_MAIN))
                                    .child(title),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(8.0))
                                    .child(
                                        Button::new("btn-cat-import-file")
                                            .label("Import TSV / CSV")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.import_catalogs_from_file(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("btn-cat-reload-seed")
                                            .label("Nạp lại từ seed_catalogs.tsv")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.reload_seed_catalogs(cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        // Filter bar
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_end()
                            .gap(px(10.0))
                            .child(self.render_select_dropdown(
                                if disease_mode {
                                    "dis-cat-type"
                                } else {
                                    "adm-cat-type"
                                },
                                "Loại danh mục",
                                &cur_type,
                                type_opts,
                                true,
                                |this, val, _, cx| {
                                    this.adm_cat_type = val;
                                    this.adm_cat_edit_id = None;
                                    this.refresh_catalogs(cx);
                                },
                                cx,
                            ))
                            .child(
                                div()
                                    .w(px(170.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Lọc mã cha (Tỉnh / CK / Bệnh)", false))
                                    .child(Input::new(&self.adm_cat_parent_filter).small().cleanable(true)),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(180.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tìm mã / tên", false))
                                    .child(Input::new(&self.adm_cat_search).small().cleanable(true)),
                            )
                            .child(
                                Button::new("btn-cat-search")
                                    .label("Lọc")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh_catalogs(cx);
                                    })),
                            ),
                    )
                    .child(
                        // Add / Edit form
                        div()
                            .p(px(10.0))
                            .rounded(px(6.0))
                            .bg(rgb(BG_SUBTLE))
                            .border_1()
                            .border_color(rgb(if is_editing { PRIMARY_COL } else { BORDER_COL }))
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_end()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(140.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã (code)", true))
                                    .child(Input::new(&self.adm_cat_code).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(200.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên hiển thị", true))
                                    .child(Input::new(&self.adm_cat_name).small()),
                            )
                            .child(
                                div()
                                    .w(px(160.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã cha / Điểm", false))
                                    .child(Input::new(&self.adm_cat_parent).small()),
                            )
                            .child(
                                Button::new("btn-cat-save-item")
                                    .label(if is_editing { "Cập nhật" } else { "+ Thêm mới" })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.adm_cat_type = cur_type.clone();
                                        this.save_catalog_item(window, cx);
                                    })),
                            )
                            .when(is_editing, |el| {
                                el.child(
                                    Button::new("btn-cat-cancel-edit")
                                        .label("Hủy sửa")
                                        .small()
                                        .secondary()
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.cancel_edit_catalog_item(window, cx);
                                        })),
                                )
                            }),
                    ),
            )
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
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(TEXT_MAIN))
                                    .child(format!("Danh sách hiển thị ({} mục, tối đa 200 dòng/lượt)", items.len())),
                            ),
                    )
                    .when(items.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(16.0))
                                .text_sm()
                                .text_color(rgb(TEXT_MUTED))
                                .child("Không có mục danh mục nào khớp điều kiện lọc."),
                        )
                    })
                    .children(items.into_iter().map(|c| {
                        let cid = c.id;
                        let is_active = c.is_active != 0;
                        let is_def = c.is_default != 0;
                        let c_for_edit = c.clone();
                        div()
                            .py(px(8.0))
                            .px(px(10.0))
                            .rounded(px(6.0))
                            .bg(rgb(if is_def { PRIMARY_LIGHT } else { BG_SUBTLE }))
                            .border_1()
                            .border_color(rgb(if is_def { PRIMARY_COL } else { BORDER_COL }))
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
                                    .flex_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(rgb(PRIMARY_COL))
                                            .child(c.code.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(if is_active { TEXT_MAIN } else { TEXT_MUTED }))
                                            .child(c.name.clone()),
                                    )
                                    .when(!c.parent_code.is_empty(), |el| {
                                        el.child(badge(format!("Cha: {}", c.parent_code), BG_CARD, TEXT_MUTED))
                                    })
                                    .when(is_def, |el| {
                                        el.child(badge("★ Mặc định", SUCCESS_BG, SUCCESS_COL))
                                    })
                                    .when(!is_active, |el| {
                                        el.child(badge("Vô hiệu", DANGER_BG, DANGER_COL))
                                    })
                                    .when(is_active && !is_def, |el| {
                                        el.child(badge("Đang dùng", WARN_BG, WARN_COL))
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(6.0))
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cat-edit-{}", cid)))
                                            .label("Sửa")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.start_edit_catalog_item(&c_for_edit, window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cat-def-{}", cid)))
                                            .label(if is_def { "Đã mặc định" } else { "Đặt mặc định" })
                                            .small()
                                            .when(is_def, |b| b.primary())
                                            .when(!is_def, |b| b.secondary())
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.set_catalog_default(cid, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cat-act-{}", cid)))
                                            .label(if is_active { "Vô hiệu" } else { "Kích hoạt" })
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.toggle_catalog_active(cid, !is_active, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-cat-del-{}", cid)))
                                            .label("Xóa")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.delete_catalog_item(cid, cx);
                                            })),
                                    ),
                            )
                    })),
            )
            .into_any_element()
    }
}
