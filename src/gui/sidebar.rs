use super::KskRecordSummary;
use dioxus::prelude::*;

#[component]
pub fn Sidebar(
    open: Signal<bool>,
    can_create: bool,
    from_date: Signal<String>,
    to_date: Signal<String>,
    status_filter: Signal<Option<i64>>,
    search_q: Signal<String>,
    records: Signal<Vec<KskRecordSummary>>,
    selected_id: Signal<Option<i64>>,
    on_refresh: EventHandler<()>,
    on_new_record: EventHandler<()>,
    on_select_record: EventHandler<i64>,
) -> Element {
    let is_open = *open.read();
    if !is_open {
        return rsx! {
            div { class: "sidebar collapsed",
                button {
                    class: "btn",
                    title: "Mở danh sách bệnh nhân",
                    onclick: move |_| open.set(true),
                    "▶"
                }
            }
        };
    }

    let recs = records.read().clone();
    let sel_id = *selected_id.read();
    let st_val = match *status_filter.read() {
        None => "".to_string(),
        Some(v) => v.to_string(),
    };

    rsx! {
        div { class: "sidebar",
            div { style: "display: flex; justify-content: space-between; align-items: center;",
                span { style: "font-weight: 700; font-size: 14px; color: #0f766e;", "📋 Danh sách khám" }
                div { style: "display: flex; gap: 4px;",
                    button { class: "btn", title: "Tải lại", onclick: move |_| on_refresh.call(()), "🔄" }
                    button { class: "btn", title: "Thu gọn", onclick: move |_| open.set(false), "◀" }
                }
            }

            if can_create {
                button {
                    class: "btn btn-primary",
                    style: "width: 100%; justify-content: center;",
                    onclick: move |_| on_new_record.call(()),
                    "➕ Tiếp nhận hồ sơ mới"
                }
            }

            div { style: "background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 7px; padding: 8px; display: flex; flex-direction: column; gap: 6px;",
                div { style: "display: flex; gap: 6px; align-items: center;",
                    input {
                        r#type: "text",
                        placeholder: "Từ dd/MM/yyyy",
                        value: "{from_date}",
                        oninput: move |e| {
                            from_date.set(e.value());
                            on_refresh.call(());
                        }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Đến dd/MM/yyyy",
                        value: "{to_date}",
                        oninput: move |e| {
                            to_date.set(e.value());
                            on_refresh.call(());
                        }
                    }
                }
                div { style: "display: flex; gap: 6px; align-items: center;",
                    select {
                        value: "{st_val}",
                        onchange: move |e| {
                            let val = e.value();
                            status_filter.set(val.parse::<i64>().ok());
                            on_refresh.call(());
                        },
                        option { value: "", selected: st_val.is_empty(), "Tất cả trạng thái" }
                        option { value: "0", selected: st_val == "0", "Đang khám" }
                        option { value: "1", selected: st_val == "1", "Đã kết luận" }
                    }
                    button {
                        class: "btn",
                        style: "white-space: nowrap;",
                        onclick: move |_| {
                            let today = chrono::Local::now().format("%d/%m/%Y").to_string();
                            from_date.set(today.clone());
                            to_date.set(today);
                            on_refresh.call(());
                        },
                        "Hôm nay"
                    }
                }
                input {
                    r#type: "text",
                    placeholder: "🔍 Quét mã TN / Tìm tên / CCCD...",
                    value: "{search_q}",
                    oninput: move |e| {
                        let q = e.value();
                        search_q.set(q.clone());
                        on_refresh.call(());
                        // Tự động mở hồ sơ khi quét mã vạch khớp chính xác mã tiếp nhận
                        let qt = q.trim().to_lowercase();
                        if !qt.is_empty() {
                            if let Some(found) = records.read().iter().find(|r| r.ma_tiep_nhan.trim().to_lowercase() == qt) {
                                on_select_record.call(found.id);
                            }
                        }
                    },
                    onkeypress: move |e| {
                        if e.key() == Key::Enter {
                            if let Some(first) = records.read().first() {
                                on_select_record.call(first.id);
                            }
                        }
                    }
                }
            }

            div { style: "font-size: 11.5px; color: #64748b;", "Tìm thấy {recs.len()} hồ sơ" }

            div { class: "sidebar-list",
                for rec in recs {
                    {
                        let is_sel = sel_id == Some(rec.id);
                        let rid = rec.id;
                        let card_cls = if is_sel { "patient-card selected" } else { "patient-card" };
                        let display_name = if rec.ho_ten.is_empty() { "(Chưa có tên)".to_string() } else { rec.ho_ten.clone() };
                        rsx! {
                            div {
                                key: "{rid}",
                                class: "{card_cls}",
                                onclick: move |_| on_select_record.call(rid),
                                div { style: "display: flex; justify-content: space-between; align-items: center; gap: 6px;",
                                    span { style: "font-weight: 700; font-size: 13px; color: #0f172a;", "{display_name}" }
                                    if rec.status == 1 {
                                        span { class: "badge badge-green", "✓ Đã KL" }
                                    } else {
                                        span { class: "badge badge-amber", "⏳ Đang khám" }
                                    }
                                }
                                div { style: "font-size: 11.5px; color: #0284c7; margin-top: 2px;",
                                    "{rec.ma_tiep_nhan}"
                                    if !rec.ngay_vao.is_empty() {
                                        span { style: "color: #64748b;", " • {rec.ngay_vao}" }
                                    }
                                }
                                if !rec.phan_loai_sk.is_empty() {
                                    div { style: "font-size: 11px; color: #0d9488; font-weight: 600; margin-top: 2px;",
                                        "Phân loại: Loại {rec.phan_loai_sk}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
