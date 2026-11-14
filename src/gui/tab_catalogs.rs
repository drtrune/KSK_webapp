use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder as _, px, rgb,
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
        bg_card, bg_subtle, border_col, DANGER_BG, DANGER_COL, PRIMARY_COL, primary_light,
        SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, badge, card_box,
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
                                    .text_color(rgb(text_main()))
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
                            .bg(rgb(bg_subtle()))
                            .border_1()
                            .border_color(rgb(if is_editing { PRIMARY_COL } else { border_col() }))
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
                                    .text_color(rgb(text_main()))
                                    .child(format!("Danh sách hiển thị ({} mục, tối đa 200 dòng/lượt)", items.len())),
                            ),
                    )
                    .when(items.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(16.0))
                                .text_sm()
                                .text_color(rgb(text_muted()))
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
                            .bg(rgb(if is_def { primary_light() } else { bg_subtle() }))
                            .border_1()
                            .border_color(rgb(if is_def { PRIMARY_COL } else { border_col() }))
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
                                            .text_color(rgb(if is_active { text_main() } else { text_muted() }))
                                            .child(c.name.clone()),
                                    )
                                    .when(!c.parent_code.is_empty(), |el| {
                                        el.child(badge(format!("Cha: {}", c.parent_code), bg_card(), text_muted()))
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

    fn query_all_catalog_items(&self, cat_type: &str) -> Vec<crate::models::CatalogItem> {
        let state = self.state.clone();
        let ct = cat_type.to_string();
        self.run_sync(async move {
            let pool = state.db.read().await;
            sqlx::query_as::<_, crate::models::CatalogItem>(
                "SELECT id, cat_type, code, name, parent_code, is_active, is_default \
                 FROM catalogs WHERE cat_type = ? ORDER BY is_active DESC, id ASC",
            )
            .bind(&ct)
            .fetch_all(&*pool)
            .await
            .unwrap_or_default()
        })
    }

    pub fn render_disease_tree_manager(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let specs = self.query_all_catalog_items("CHUYEN_KHOA");
        let diseases = self.query_all_catalog_items("BENH_TAT");
        let standards = self.query_all_catalog_items("TIEU_CHUAN_PL");

        let search = self.adm_cat_search.read(cx).value().trim().to_lowercase();
        let expand_all = self.adm_tree_expand_all || !search.is_empty();
        let editing_id = self.adm_cat_edit_id;
        let cur_type = self.adm_cat_type.clone();

        let type_label = match cur_type.as_str() {
            "CHUYEN_KHOA" => "Chuyên khoa",
            "TIEU_CHUAN_PL" => "Tiêu chuẩn phân loại",
            _ => "Mặt bệnh",
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
                            .flex_wrap()
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
                                            .text_base()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .child("Cây Phân loại Bệnh tật (Chuyên khoa → Mặt bệnh → Tiêu chuẩn)"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted()))
                                            .child(format!(
                                                "{} chuyên khoa • {} mặt bệnh • {} tiêu chuẩn",
                                                specs.len(),
                                                diseases.len(),
                                                standards.len()
                                            )),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .flex_wrap()
                                    .gap(px(6.0))
                                    .child(
                                        Button::new("tree-exp-all")
                                            .label("Mở tất cả")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.adm_tree_expand_all = true;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("tree-col-all")
                                            .label("Thu gọn")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.adm_tree_expand_all = false;
                                                this.adm_tree_expanded.clear();
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("tree-add-spec")
                                            .label("+ Chuyên khoa")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.adm_cat_edit_id = None;
                                                this.adm_cat_type = "CHUYEN_KHOA".into();
                                                this.adm_cat_code.update(cx, |s, cx| s.set_value("", window, cx));
                                                this.adm_cat_name.update(cx, |s, cx| s.set_value("", window, cx));
                                                this.adm_cat_parent.update(cx, |s, cx| s.set_value("", window, cx));
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("tree-import")
                                            .label("Import TSV/CSV")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.import_catalogs_from_file(cx);
                                            })),
                                    )
                                    .child(
                                        Button::new("tree-reload")
                                            .label("Nạp lại seed_catalogs.tsv")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.reload_seed_catalogs(cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(8.0))
                            .child(div().flex_1().child(Input::new(&self.adm_cat_search).small().cleanable(true))),
                    ),
            )
            .child(
                // Form Thêm / Sửa node trong cây
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
                                    .child(if editing_id.is_some() {
                                        format!("Chỉnh sửa {} đang chọn", type_label)
                                    } else {
                                        format!("Thêm mới {}", type_label)
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .children(
                                        [
                                            ("CHUYEN_KHOA", "Chuyên khoa"),
                                            ("BENH_TAT", "Mặt bệnh"),
                                            ("TIEU_CHUAN_PL", "Tiêu chuẩn"),
                                        ]
                                        .into_iter()
                                        .map(|(ct, lbl)| {
                                            let active = cur_type == ct;
                                            let ct_s = ct.to_string();
                                            Button::new(SharedString::from(format!("tree-type-{}", ct)))
                                                .label(lbl)
                                                .xsmall()
                                                .when(active, |b| b.primary())
                                                .when(!active, |b| b.secondary())
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.adm_cat_type = ct_s.clone();
                                                    cx.notify();
                                                }))
                                        }),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_end()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .w(px(150.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã", true))
                                    .child(Input::new(&self.adm_cat_code).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(220.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên hiển thị (ghi kèm (Loại X) nếu là tiêu chuẩn)", true))
                                    .child(Input::new(&self.adm_cat_name).small()),
                            )
                            .child(
                                div()
                                    .w(px(170.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã cha", false))
                                    .child(Input::new(&self.adm_cat_parent).small()),
                            )
                            .child(
                                Button::new("tree-btn-save")
                                    .label(if editing_id.is_some() { "Lưu sửa" } else { "+ Thêm" })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_catalog_item(window, cx);
                                    })),
                            )
                            .when(editing_id.is_some(), |el| {
                                el.child(
                                    Button::new("tree-btn-cancel")
                                        .label("Huỷ")
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
                // Cây 3 tầng: Chuyên khoa -> Mặt bệnh -> Tiêu chuẩn
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .children(specs.into_iter().filter_map(|sp| {
                        let sp_diseases: Vec<_> = diseases
                            .iter()
                            .filter(|d| d.parent_code == sp.code)
                            .cloned()
                            .collect();

                        let sp_match = search.is_empty()
                            || sp.name.to_lowercase().contains(&search)
                            || sp.code.to_lowercase().contains(&search);

                        let filtered_diseases: Vec<_> = if sp_match {
                            sp_diseases.clone()
                        } else {
                            sp_diseases
                                .into_iter()
                                .filter(|d| {
                                    let d_match = d.name.to_lowercase().contains(&search)
                                        || d.code.to_lowercase().contains(&search);
                                    let has_std = standards.iter().any(|st| {
                                        st.parent_code == d.code
                                            && (st.name.to_lowercase().contains(&search)
                                                || st.code.to_lowercase().contains(&search))
                                    });
                                    d_match || has_std
                                })
                                .collect()
                        };

                        if !search.is_empty() && !sp_match && filtered_diseases.is_empty() {
                            return None;
                        }

                        let sp_key = format!("S:{}", sp.code);
                        let is_sp_open = expand_all || self.adm_tree_expanded.contains(&sp_key);
                        let sp_code = sp.code.clone();
                        let sp_item = sp.clone();
                        let sp_id = sp.id;
                        let sp_active = sp.is_active == 1;
                        let dis_count = filtered_diseases.len();

                        Some(
                            div()
                                .rounded(px(8.0))
                                .border_1()
                                .border_color(rgb(border_col()))
                                .bg(rgb(bg_card()))
                                .overflow_hidden()
                                .child(
                                    div()
                                        .px(px(12.0))
                                        .py(px(8.0))
                                        .bg(rgb(bg_subtle()))
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .id(SharedString::from(format!("tree-sp-tog-{}", sp.id)))
                                                .flex_1()
                                                .cursor_pointer()
                                                .flex()
                                                .flex_row()
                                                .items_center()
                                                .gap(px(8.0))
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    if this.adm_tree_expanded.contains(&sp_key) {
                                                        this.adm_tree_expanded.remove(&sp_key);
                                                    } else {
                                                        this.adm_tree_expanded.insert(sp_key.clone());
                                                    }
                                                    cx.notify();
                                                }))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(text_muted()))
                                                        .child(if is_sp_open { "▼" } else { "▶" }),
                                                )
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_weight(gpui::FontWeight::BOLD)
                                                        .text_color(rgb(text_main()))
                                                        .child(format!("[{}] {}", sp.code, sp.name)),
                                                )
                                                .child(badge(
                                                    format!("{} bệnh", dis_count),
                                                    primary_light(),
                                                    PRIMARY_COL,
                                                ))
                                                .when(!sp_active, |el| {
                                                    el.child(badge("Đã vô hiệu", DANGER_BG, DANGER_COL))
                                                }),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_row()
                                                .gap(px(4.0))
                                                .child(
                                                    Button::new(SharedString::from(format!("tsp-add-{}", sp_id)))
                                                        .label("+ Bệnh")
                                                        .xsmall()
                                                        .secondary()
                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                            this.adm_cat_edit_id = None;
                                                            this.adm_cat_type = "BENH_TAT".into();
                                                            this.adm_cat_code.update(cx, |s, cx| s.set_value("", window, cx));
                                                            this.adm_cat_name.update(cx, |s, cx| s.set_value("", window, cx));
                                                            this.adm_cat_parent.update(cx, |s, cx| s.set_value(sp_code.clone(), window, cx));
                                                            cx.notify();
                                                        })),
                                                )
                                                .child(
                                                    Button::new(SharedString::from(format!("tsp-ed-{}", sp_id)))
                                                        .label("Sửa")
                                                        .xsmall()
                                                        .secondary()
                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                            this.adm_cat_type = "CHUYEN_KHOA".into();
                                                            this.start_edit_catalog_item(&sp_item, window, cx);
                                                        })),
                                                )
                                                .child(
                                                    Button::new(SharedString::from(format!("tsp-act-{}", sp_id)))
                                                        .label(if sp_active { "Ẩn" } else { "Hiện" })
                                                        .xsmall()
                                                        .secondary()
                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                            this.toggle_catalog_active(sp_id, !sp_active, cx);
                                                        })),
                                                ),
                                        ),
                                )
                                .when(is_sp_open, |el| {
                                    el.child(
                                        div()
                                            .pl(px(24.0))
                                            .pr(px(10.0))
                                            .py(px(8.0))
                                            .border_t_1()
                                            .border_color(rgb(border_col()))
                                            .flex()
                                            .flex_col()
                                            .gap(px(6.0))
                                            .children(filtered_diseases.into_iter().map(|d| {
                                                let d_stds: Vec<_> = standards
                                                    .iter()
                                                    .filter(|st| st.parent_code == d.code)
                                                    .filter(|st| {
                                                        search.is_empty()
                                                            || d.name.to_lowercase().contains(&search)
                                                            || d.code.to_lowercase().contains(&search)
                                                            || st.name.to_lowercase().contains(&search)
                                                            || st.code.to_lowercase().contains(&search)
                                                    })
                                                    .cloned()
                                                    .collect();
                                                let d_key = format!("D:{}", d.code);
                                                let is_d_open = expand_all || self.adm_tree_expanded.contains(&d_key);
                                                let d_code = d.code.clone();
                                                let d_item = d.clone();
                                                let d_id = d.id;
                                                let d_active = d.is_active == 1;
                                                let std_count = d_stds.len();

                                                div()
                                                    .rounded(px(6.0))
                                                    .border_1()
                                                    .border_color(rgb(border_col()))
                                                    .bg(rgb(bg_subtle()))
                                                    .child(
                                                        div()
                                                            .px(px(10.0))
                                                            .py(px(6.0))
                                                            .flex()
                                                            .flex_row()
                                                            .items_center()
                                                            .justify_between()
                                                            .gap(px(8.0))
                                                            .child(
                                                                div()
                                                                    .id(SharedString::from(format!("tree-d-tog-{}", d.id)))
                                                                    .flex_1()
                                                                    .cursor_pointer()
                                                                    .flex()
                                                                    .flex_row()
                                                                    .items_center()
                                                                    .gap(px(6.0))
                                                                    .on_click(cx.listener(move |this, _, _, cx| {
                                                                        if this.adm_tree_expanded.contains(&d_key) {
                                                                            this.adm_tree_expanded.remove(&d_key);
                                                                        } else {
                                                                            this.adm_tree_expanded.insert(d_key.clone());
                                                                        }
                                                                        cx.notify();
                                                                    }))
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .text_color(rgb(text_muted()))
                                                                            .child(if is_d_open { "▼" } else { "▶" }),
                                                                    )
                                                                    .child(
                                                                        div()
                                                                            .text_xs()
                                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                                            .text_color(rgb(text_main()))
                                                                            .child(format!("[{}] {}", d.code, d.name)),
                                                                    )
                                                                    .child(badge(
                                                                        format!("{} mức", std_count),
                                                                        bg_card(),
                                                                        text_muted(),
                                                                    ))
                                                                    .when(!d_active, |el| {
                                                                        el.child(badge("Ẩn", DANGER_BG, DANGER_COL))
                                                                    }),
                                                            )
                                                            .child(
                                                                div()
                                                                    .flex()
                                                                    .flex_row()
                                                                    .gap(px(4.0))
                                                                    .child(
                                                                        Button::new(SharedString::from(format!("td-add-{}", d_id)))
                                                                            .label("+ Tiêu chuẩn")
                                                                            .xsmall()
                                                                            .secondary()
                                                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                                                this.adm_cat_edit_id = None;
                                                                                this.adm_cat_type = "TIEU_CHUAN_PL".into();
                                                                                this.adm_cat_code.update(cx, |s, cx| s.set_value("", window, cx));
                                                                                this.adm_cat_name.update(cx, |s, cx| s.set_value("", window, cx));
                                                                                this.adm_cat_parent.update(cx, |s, cx| s.set_value(d_code.clone(), window, cx));
                                                                                cx.notify();
                                                                            })),
                                                                    )
                                                                    .child(
                                                                        Button::new(SharedString::from(format!("td-ed-{}", d_id)))
                                                                            .label("Sửa")
                                                                            .xsmall()
                                                                            .secondary()
                                                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                                                this.adm_cat_type = "BENH_TAT".into();
                                                                                this.start_edit_catalog_item(&d_item, window, cx);
                                                                            })),
                                                                    )
                                                                    .child(
                                                                        Button::new(SharedString::from(format!("td-act-{}", d_id)))
                                                                            .label(if d_active { "Ẩn" } else { "Hiện" })
                                                                            .xsmall()
                                                                            .secondary()
                                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                                this.toggle_catalog_active(d_id, !d_active, cx);
                                                                            })),
                                                                    )
                                                                    .child(
                                                                        Button::new(SharedString::from(format!("td-del-{}", d_id)))
                                                                            .label("Xoá")
                                                                            .xsmall()
                                                                            .secondary()
                                                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                                                this.delete_catalog_item(d_id, cx);
                                                                            })),
                                                                    ),
                                                            ),
                                                    )
                                                    .when(is_d_open, |el| {
                                                        el.child(
                                                            div()
                                                                .pl(px(22.0))
                                                                .pr(px(8.0))
                                                                .py(px(6.0))
                                                                .border_t_1()
                                                                .border_color(rgb(border_col()))
                                                                .flex()
                                                                .flex_col()
                                                                .gap(px(4.0))
                                                                .children(d_stds.into_iter().map(|st| {
                                                                    let st_id = st.id;
                                                                    let st_active = st.is_active == 1;
                                                                    let st_item = st.clone();
                                                                    div()
                                                                        .px(px(8.0))
                                                                        .py(px(4.0))
                                                                        .rounded(px(4.0))
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
                                                                                .text_color(rgb(if st_active { text_main() } else { text_muted() }))
                                                                                .child(format!("[{}] {}", st.code, st.name)),
                                                                        )
                                                                        .child(
                                                                            div()
                                                                                .flex()
                                                                                .flex_row()
                                                                                .gap(px(4.0))
                                                                                .when(!st_active, |b| {
                                                                                    b.child(badge("Ẩn", DANGER_BG, DANGER_COL))
                                                                                })
                                                                                .child(
                                                                                    Button::new(SharedString::from(format!("tst-ed-{}", st_id)))
                                                                                        .label("Sửa")
                                                                                        .xsmall()
                                                                                        .secondary()
                                                                                        .on_click(cx.listener(move |this, _, window, cx| {
                                                                                            this.adm_cat_type = "TIEU_CHUAN_PL".into();
                                                                                            this.start_edit_catalog_item(&st_item, window, cx);
                                                                                        })),
                                                                                )
                                                                                .child(
                                                                                    Button::new(SharedString::from(format!("tst-act-{}", st_id)))
                                                                                        .label(if st_active { "Ẩn" } else { "Hiện" })
                                                                                        .xsmall()
                                                                                        .secondary()
                                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                                            this.toggle_catalog_active(st_id, !st_active, cx);
                                                                                        })),
                                                                                )
                                                                                .child(
                                                                                    Button::new(SharedString::from(format!("tst-del-{}", st_id)))
                                                                                        .label("Xoá")
                                                                                        .xsmall()
                                                                                        .secondary()
                                                                                        .on_click(cx.listener(move |this, _, _, cx| {
                                                                                            this.delete_catalog_item(st_id, cx);
                                                                                        })),
                                                                                ),
                                                                        )
                                                                })),
                                                        )
                                                    })
                                            })),
                                    )
                                }),
                        )
                    })),
            )
            .into_any_element()
    }
}
