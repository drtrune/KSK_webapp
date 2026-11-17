use gpui::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder as _, px, rgb,
};
use gpui_component::{
    Sizable,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    input::Input,
};

use crate::models::VALID_ROLES;

use super::{
    KskGpuiApp,
    widgets::{
        bg_card, bg_subtle, border_col, DANGER_BG, DANGER_COL, PRIMARY_COL, primary_light,
        SUCCESS_BG, SUCCESS_COL, text_main, text_muted, WARN_BG, WARN_COL, badge, card_box,
        field_label, role_label_vi,
    },
};

impl KskGpuiApp {
    pub fn render_export_tab(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let rows = self.export_rows.clone();
        let total = rows.len();
        let locked_cnt = rows.iter().filter(|r| r.status == 2).count();
        let selected_cnt = self.export_selected_ids.len();
        let all_selected = total > 0 && rows.iter().all(|r| self.export_selected_ids.contains(&r.id));

        let st_val = self
            .exp_status_filter
            .map(|s| s.to_string())
            .unwrap_or_default();
        let exp_mode_val = self.exp_exam_mode.clone();
        let exp_cont_val = self
            .exp_contract_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let mut exp_cont_opts: Vec<(String, String)> = vec![("".into(), "Tất cả hợp đồng".into())];
        for c in &self.contracts_list {
            exp_cont_opts.push((
                c.id.to_string(),
                format!("{} - {}", c.contract_code, c.company_name),
            ));
        }

        div()
            .id("export-tab-scroll")
            .flex_1()
            .h_full()
            .overflow_y_scroll()
            .p(px(16.0))
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
                                    .text_color(rgb(text_main()))
                                    .child("Xuất dữ liệu CSV chuẩn 123 cột (QĐ 3176/BYT)"),
                            )
                            .child(
                                Button::new("btn-export-csv-exec")
                                    .label(if selected_cnt > 0 {
                                        format!("Xuất {} hồ sơ đã chọn (CSV)", selected_cnt)
                                    } else {
                                        format!("Xuất tất cả {} hồ sơ (CSV)", total)
                                    })
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.execute_csv_export(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .items_end()
                            .gap(px(10.0))
                            .child(
                                div()
                                    .w(px(140.0))
                                    .child(self.render_date_picker_entity(
                                        "exp-from",
                                        "Từ ngày",
                                        self.exp_from_date.clone(),
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .w(px(140.0))
                                    .child(self.render_date_picker_entity(
                                        "exp-to",
                                        "Đến ngày",
                                        self.exp_to_date.clone(),
                                        cx,
                                    )),
                            )
                            .child(
                                div()
                                    .w(px(200.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tìm tên / Mã BN / CCCD", false))
                                    .child(Input::new(&self.exp_q).small().cleanable(true)),
                            )
                            .child(
                                div()
                                    .w(px(180.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Đơn vị / Công ty", false))
                                    .child(Input::new(&self.exp_workplace).small().cleanable(true)),
                            )
                            .child(self.render_select_dropdown(
                                "exp-status",
                                "Trạng thái",
                                &st_val,
                                vec![
                                    ("".to_string(), "Tất cả".to_string()),
                                    ("0".to_string(), "Đang khám".to_string()),
                                    ("1".to_string(), "Hoàn tất".to_string()),
                                    ("2".to_string(), "Đã ký khóa".to_string()),
                                ],
                                true,
                                |this, val, _, _| {
                                    this.exp_status_filter = val.parse::<i64>().ok();
                                },
                                cx,
                            ))
                            .child(self.render_select_dropdown(
                                "exp-exam-mode",
                                "Loại khám",
                                &exp_mode_val,
                                vec![
                                    ("".to_string(), "Tất cả".to_string()),
                                    ("le".to_string(), "Khám lẻ".to_string()),
                                    ("doan".to_string(), "Khám đoàn".to_string()),
                                ],
                                true,
                                |this, val, _, cx| {
                                    this.exp_exam_mode = val;
                                    if this.exp_exam_mode != "doan" {
                                        this.exp_contract_id = None;
                                    }
                                    cx.notify();
                                },
                                cx,
                            ))
                            .when(exp_mode_val == "doan", |el| {
                                el.child(self.render_select_dropdown(
                                    "exp-contract",
                                    "Hợp đồng đoàn",
                                    &exp_cont_val,
                                    exp_cont_opts,
                                    true,
                                    |this, val, _, _| {
                                        this.exp_contract_id = val.parse::<i64>().ok();
                                    },
                                    cx,
                                ))
                            })
                            .child(
                                Button::new("btn-exp-filter")
                                    .label("Lọc dữ liệu")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.refresh_export_rows(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                // KPI bar
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.0))
                    .child(kpi_card("Tổng hồ sơ lọc", &total.to_string(), PRIMARY_COL))
                    .child(kpi_card("Đã ký khóa", &locked_cnt.to_string(), SUCCESS_COL))
                    .child(kpi_card(
                        "Chưa ký khóa",
                        &(total.saturating_sub(locked_cnt)).to_string(),
                        WARN_COL,
                    ))
                    .child(kpi_card("Đang tích chọn", &selected_cnt.to_string(), text_main())),
            )
            .child(
                card_box()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .pb(px(8.0))
                            .border_b_1()
                            .border_color(rgb(border_col()))
                            .child(
                                Checkbox::new("exp-select-all")
                                    .label("Chọn tất cả")
                                    .checked(all_selected)
                                    .on_change(cx.listener(move |this, val, _, cx| {
                                        if *val {
                                            for r in &this.export_rows {
                                                this.export_selected_ids.insert(r.id);
                                            }
                                        } else {
                                            this.export_selected_ids.clear();
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .when(rows.is_empty(), |el| {
                        el.child(
                            div()
                                .p(px(16.0))
                                .text_sm()
                                .text_color(rgb(text_muted()))
                                .child("Không có dữ liệu phù hợp bộ lọc."),
                        )
                    })
                    .children(rows.into_iter().map(|r| {
                        let checked = self.export_selected_ids.contains(&r.id);
                        let rid = r.id;
                        let (st_lbl, st_bg, st_fg) = match r.status {
                            2 => ("Đã ký", DANGER_BG, DANGER_COL),
                            1 => ("Hoàn tất", SUCCESS_BG, SUCCESS_COL),
                            _ => ("Đang khám", WARN_BG, WARN_COL),
                        };
                        div()
                            .py(px(6.0))
                            .px(px(8.0))
                            .rounded(px(6.0))
                            .bg(rgb(if checked { primary_light() } else { bg_subtle() }))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                Checkbox::new(SharedString::from(format!("exp-row-{}", rid)))
                                    .checked(checked)
                                    .on_change(cx.listener(move |this, val, _, cx| {
                                        if *val {
                                            this.export_selected_ids.insert(rid);
                                        } else {
                                            this.export_selected_ids.remove(&rid);
                                        }
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .w(px(120.0))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .child(r.ma_y_te.clone()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child(r.ho_ten.clone()),
                            )
                            .child(
                                div()
                                    .w(px(110.0))
                                    .text_xs()
                                    .text_color(rgb(text_muted()))
                                    .child(r.ngay_vao.clone()),
                            )
                            .child(
                                div()
                                    .w(px(180.0))
                                    .text_xs()
                                    .text_color(rgb(text_muted()))
                                    .truncate()
                                    .child(super::payload_field(&r, "NOI_LAM_VIEC_HOC_TAP")),
                            )
                            .child(
                                div()
                                    .w(px(70.0))
                                    .text_xs()
                                    .child({
                                        let pl = super::payload_field(&r, "PHAN_LOAI_SK");
                                        if pl.is_empty() { "-".to_string() } else { pl }
                                    }),
                            )
                            .child(badge(st_lbl, st_bg, st_fg))
                    })),
            )
            .into_any_element()
    }

    pub fn render_admin_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let sub = self.admin_sub_tab.clone();
        let tabs = [
            ("config", "Cấu hình DB & Cập nhật"),
            ("users", "Quản lý Người dùng"),
            ("catalogs", "Danh mục (ICD-10 / Hành chính)"),
            ("disease_catalog", "Phân loại Bệnh tật (Cây)"),
        ];

        let content = match sub.as_str() {
            "users" => self.render_admin_users(window, cx),
            "catalogs" => self.render_catalogs_manager(false, window, cx),
            "disease_catalog" => self.render_disease_tree_manager(window, cx),
            _ => self.render_admin_config(window, cx),
        };

        div()
            .id("admin-tab-scroll")
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
                    .gap(px(8.0))
                    .children(tabs.iter().map(|(k, lbl)| {
                        let is_act = sub == *k;
                        let key_s = k.to_string();
                        Button::new(SharedString::from(format!("adm-sub-{}", k)))
                            .label(*lbl)
                            .small()
                            .when(is_act, |b| b.primary())
                            .when(!is_act, |b| b.secondary())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.admin_sub_tab = key_s.clone();
                                if key_s == "users" {
                                    this.refresh_users(cx);
                                } else if key_s == "catalogs" {
                                    if matches!(
                                        this.adm_cat_type.as_str(),
                                        "CHUYEN_KHOA" | "BENH_TAT" | "TIEU_CHUAN_PL"
                                    ) {
                                        this.adm_cat_type = "TINH_TP".into();
                                    }
                                    this.refresh_catalogs(cx);
                                } else if key_s == "disease_catalog" {
                                    if !matches!(
                                        this.adm_cat_type.as_str(),
                                        "CHUYEN_KHOA" | "BENH_TAT" | "TIEU_CHUAN_PL"
                                    ) {
                                        this.adm_cat_type = "BENH_TAT".into();
                                    }
                                }
                                cx.notify();
                            }))
                    })),
            )
            .child(content)
            .into_any_element()
    }

    fn render_admin_config(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
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
                            .child("Cấu hình Cơ sở dữ liệu (Ổ đĩa chung LAN / SMB) & Đơn vị"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên CSKCB", false))
                                    .child(Input::new(&self.adm_cskcb).small()),
                            )
                            .child(
                                div()
                                    .w(px(200.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mã CSKCB (5 số)", false))
                                    .child(Input::new(&self.adm_ma_cskcb).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Đường dẫn file SQLite (.db trên ổ mạng)", false))
                                    .child(Input::new(&self.adm_db_path).small()),
                            )
                            .child(
                                Button::new("btn-adm-browse-db")
                                    .label("Chọn file .db")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(p) = rfd::FileDialog::new()
                                            .add_filter("SQLite DB", &["db", "sqlite"])
                                            .save_file()
                                        {
                                            let s = p.to_string_lossy().to_string();
                                            this.adm_db_path.update(cx, |inp, cx| {
                                                inp.set_value(s, window, cx);
                                            });
                                        }
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Thư mục chứa bản cập nhật trên ổ chung", false))
                                    .child(Input::new(&self.adm_update_dir).small()),
                            )
                            .child(
                                Button::new("btn-adm-browse-upd")
                                    .label("Chọn thư mục")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                            let s = p.to_string_lossy().to_string();
                                            this.adm_update_dir.update(cx, |inp, cx| {
                                                inp.set_value(s, window, cx);
                                            });
                                        }
                                    })),
                            ),
                    )
                    .child(
                        Checkbox::new("adm-auto-update")
                            .label("Tự động kiểm tra cập nhật khi mở ứng dụng")
                            .checked(self.adm_auto_check_update)
                            .on_change(cx.listener(|this, val, _, cx| {
                                this.adm_auto_check_update = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .pt(px(8.0))
                            .border_t_1()
                            .border_color(rgb(border_col()))
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(text_main()))
                                    .child("Tự động sao lưu Database về máy (Auto-Backup)"),
                            )
                            .child(
                                Checkbox::new("adm-auto-backup")
                                    .label("Bật tự động tải/sao lưu bản sao database về máy định kỳ")
                                    .checked(self.adm_auto_backup)
                                    .on_change(cx.listener(|this, val, _, cx| {
                                        this.adm_auto_backup = *val;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_end()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex()
                                            .flex_col()
                                            .gap(px(4.0))
                                            .child(field_label("Thư mục lưu bản sao lưu trên máy", false))
                                            .child(Input::new(&self.adm_backup_dir).small()),
                                    )
                                    .child(
                                        Button::new("btn-adm-browse-bkp")
                                            .label("Chọn thư mục")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                                    let s = p.to_string_lossy().to_string();
                                                    this.adm_backup_dir.update(cx, |inp, cx| {
                                                        inp.set_value(s, window, cx);
                                                    });
                                                }
                                            })),
                                    )
                                    .child(
                                        div()
                                            .w(px(110.0))
                                            .flex()
                                            .flex_col()
                                            .gap(px(4.0))
                                            .child(field_label("Chu kỳ (phút)", false))
                                            .child(Input::new(&self.adm_backup_interval).small()),
                                    )
                                    .child(
                                        Button::new("btn-adm-backup-now")
                                            .label("Backup DB ngay")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.trigger_manual_backup(cx);
                                            })),
                                    ),
                            )
                            .when_some(self.last_backup_status.clone(), |el, st| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(SUCCESS_COL))
                                        .child(st),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .child(
                                Button::new("btn-adm-save-cfg")
                                    .label("Lưu cấu hình & Kết nối lại DB")
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_admin_config(window, cx);
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
                            .child("Tạo Cơ sở dữ liệu mới & Chuyển sang DB mới"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(text_muted()))
                            .child("Khởi tạo một file SQLite mới hoàn toàn với Master Key, Recovery Key và tài khoản Admin mới, sau đó tự động chuyển kết nối sang DB này."),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_end()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Đường dẫn file DB mới (.db)", true))
                                    .child(Input::new(&self.adm_new_db_path).small()),
                            )
                            .child(
                                Button::new("btn-adm-new-db-browse")
                                    .label("Chọn nơi lưu .db")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(p) = rfd::FileDialog::new()
                                            .add_filter("SQLite DB", &["db", "sqlite"])
                                            .set_file_name("ksk_moi.db")
                                            .save_file()
                                        {
                                            let s = p.to_string_lossy().to_string();
                                            this.adm_new_db_path.update(cx, |inp, cx| {
                                                inp.set_value(s, window, cx);
                                            });
                                        }
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mật khẩu Admin cho DB mới", true))
                                    .child(Input::new(&self.adm_new_db_admin_pw).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Nhập lại mật khẩu Admin", true))
                                    .child(Input::new(&self.adm_new_db_admin_pw_confirm).small()),
                            ),
                    )
                    .when_some(self.adm_new_db_recovery_key.clone(), |el, rk| {
                        let rk_copy = rk.clone();
                        el.child(
                            div()
                                .p(px(10.0))
                                .rounded(px(6.0))
                                .bg(rgb(WARN_BG))
                                .border_1()
                                .border_color(rgb(WARN_COL))
                                .flex()
                                .flex_col()
                                .gap(px(6.0))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(WARN_COL))
                                        .child("Recovery Key của DB mới (hãy sao chép và cất giữ an toàn):"),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(8.0))
                                        .child(
                                            div()
                                                .flex_1()
                                                .p(px(6.0))
                                                .rounded(px(4.0))
                                                .bg(rgb(bg_subtle()))
                                                .text_sm()
                                                .font_weight(gpui::FontWeight::BOLD)
                                                .child(rk),
                                        )
                                        .child(
                                            Button::new("btn-adm-copy-new-db-rk")
                                                .label("Sao chép")
                                                .small()
                                                .secondary()
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    cx.write_to_clipboard(
                                                        gpui::ClipboardItem::new_string(
                                                            rk_copy.clone(),
                                                        ),
                                                    );
                                                    this.show_toast(
                                                        "Đã sao chép Recovery Key vào clipboard!",
                                                        false,
                                                        cx,
                                                    );
                                                })),
                                        ),
                                ),
                        )
                    })
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .child(
                                Button::new("btn-adm-create-new-db")
                                    .label("Tạo DB mới & Chuyển sang DB mới")
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.create_and_switch_new_db(window, cx);
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
                            .child(format!(
                                "Phát hành bản cập nhật (Phiên bản hiện tại: v{})",
                                crate::updater::CURRENT_VERSION
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Ghi chú thay đổi (Changelog)", false))
                            .child(Input::new(&self.adm_changelog).small()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .child(
                                Button::new("btn-adm-publish-upd")
                                    .label("Đóng gói & Phát hành lên thư mục chung")
                                    .small()
                                    .secondary()
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.publish_update(cx);
                                    })),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// Hiển thị giao diện quản lý, tạo mới, chỉnh sửa và reset mật khẩu tài khoản nhân viên.
    fn render_admin_users(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let users = self.users_list.clone();
        let edit_card = if self.adm_edit_user_id.is_some() {
            Some(self.render_edit_user_card(cx))
        } else {
            None
        };
        div()
            .flex()
            .flex_col()
            .gap(px(14.0))
            .children(edit_card)
            .child(
                card_box()
                    .child(
                        div()
                            .text_base()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("Tạo tài khoản nhân viên mới (Multi-Role)"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Tên đăng nhập", true))
                                    .child(Input::new(&self.adm_new_user).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Mật khẩu", true))
                                    .child(Input::new(&self.adm_new_pw).small()),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(field_label("Họ tên / Chức danh", true))
                                    .child(Input::new(&self.adm_new_fullname).small()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .flex_wrap()
                            .gap(px(8.0))
                            .children(VALID_ROLES.iter().map(|r| {
                                let checked = self.adm_new_roles.contains(&r.to_string());
                                let role_s = r.to_string();
                                Checkbox::new(SharedString::from(format!("new-role-{}", r)))
                                    .label(role_label_vi(r))
                                    .checked(checked)
                                    .on_change(cx.listener(move |this, val, _, cx| {
                                        if *val {
                                            if !this.adm_new_roles.contains(&role_s) {
                                                this.adm_new_roles.push(role_s.clone());
                                            }
                                        } else {
                                            this.adm_new_roles.retain(|x| x != &role_s);
                                        }
                                        cx.notify();
                                    }))
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .child(
                                Button::new("btn-adm-create-user")
                                    .label("Tạo tài khoản")
                                    .small()
                                    .primary()
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.create_admin_user(window, cx);
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
                            .child(format!("Danh sách tài khoản ({})", users.len())),
                    )
                    .children(users.into_iter().map(|u| {
                        let uid = u.id;
                        let is_active = u.is_locked == 0;
                        div()
                            .p(px(10.0))
                            .rounded(px(6.0))
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
                                    .gap(px(2.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child(format!("{} ({})", u.full_name, u.username)),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(text_muted()))
                                            .child(format!("Quyền: {}", u.roles().join(", "))),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(if is_active {
                                        badge("Hoạt động", SUCCESS_BG, SUCCESS_COL)
                                    } else {
                                        badge("Đã khóa", DANGER_BG, DANGER_COL)
                                    })
                                    .child(
                                        Button::new(SharedString::from(format!("btn-edit-u-{}", uid)))
                                            .label("Sửa / Reset MK")
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.start_edit_user(uid, window, cx);
                                            })),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("btn-toggle-u-{}", uid)))
                                            .label(if is_active { "Khóa" } else { "Mở khóa" })
                                            .small()
                                            .secondary()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.toggle_user_active(uid, is_active, cx);
                                            })),
                                    ),
                            )
                    })),
            )
            .into_any_element()
    }

    /// Hiển thị thẻ chỉnh sửa thông tin, phân quyền và reset mật khẩu cho tài khoản đang chọn.
    fn render_edit_user_card(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let username = self.adm_edit_username.clone();
        card_box()
            .border_color(rgb(PRIMARY_COL))
            .child(
                div()
                    .text_base()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child(format!("Chỉnh sửa tài khoản & Reset mật khẩu: {}", username)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(12.0))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Họ tên / Chức danh", true))
                            .child(Input::new(&self.adm_edit_fullname).small()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(field_label("Mật khẩu mới (để trống nếu giữ nguyên)", false))
                            .child(Input::new(&self.adm_edit_pw).small()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(8.0))
                    .children(VALID_ROLES.iter().map(|r| {
                        let checked = self.adm_edit_roles.contains(&r.to_string());
                        let role_s = r.to_string();
                        Checkbox::new(SharedString::from(format!("edit-role-{}", r)))
                            .label(role_label_vi(r))
                            .checked(checked)
                            .on_change(cx.listener(move |this, val, _, cx| {
                                if *val {
                                    if !this.adm_edit_roles.contains(&role_s) {
                                        this.adm_edit_roles.push(role_s.clone());
                                    }
                                } else {
                                    this.adm_edit_roles.retain(|x| x != &role_s);
                                }
                                cx.notify();
                            }))
                    })),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(px(8.0))
                    .child(
                        Button::new("btn-adm-cancel-edit-user")
                            .label("Hủy")
                            .small()
                            .secondary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cancel_edit_user(window, cx);
                            })),
                    )
                    .child(
                        Button::new("btn-adm-save-edit-user")
                            .label("Lưu thay đổi")
                            .small()
                            .primary()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_edit_user(window, cx);
                            })),
                    ),
            )
            .into_any_element()
    }

}

fn kpi_card(title: &str, value: &str, color_hex: u32) -> AnyElement {
    div()
        .flex_1()
        .bg(rgb(bg_card()))
        .border_1()
        .border_color(rgb(border_col()))
        .rounded(px(8.0))
        .p(px(12.0))
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            div()
                .text_xs()
                .text_color(rgb(text_muted()))
                .child(title.to_string()),
        )
        .child(
            div()
                .text_xl()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(color_hex))
                .child(value.to_string()),
        )
        .into_any_element()
}
