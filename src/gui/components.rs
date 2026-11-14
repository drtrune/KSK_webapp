use crate::models::CatalogItem;
use dioxus::prelude::*;
use serde_json::{json, Map, Value};

pub fn map_get(map: &Map<String, Value>, key: &str) -> String {
    map.get(key)
        .and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(if *b { "1".to_string() } else { "0".to_string() }),
            _ => None,
        })
        .unwrap_or_default()
}

pub fn form_set_key(form: &mut Signal<Map<String, Value>>, key: &str, val: String) {
    form.write().insert(key.to_string(), json!(val));
}

pub const PL_OPTIONS: &[(&str, &str)] = &[
    ("", "-- Chưa phân loại --"),
    ("1", "Loại 1 (Rất khỏe)"),
    ("2", "Loại 2 (Khỏe)"),
    ("3", "Loại 3 (Trung bình)"),
    ("4", "Loại 4 (Yếu)"),
    ("5", "Loại 5 (Rất yếu)"),
];

pub const YES_NO_OPTIONS: &[(&str, &str)] = &[("0", "Không"), ("1", "Có")];

#[component]
pub fn SectionHeader(
    title: String,
    can_edit_role: bool,
    is_new_record: bool,
    is_locked_record: bool,
    is_editing: bool,
    on_edit: EventHandler<()>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "section-header",
            span { class: "section-title", "{title}" }
            div { style: "display: flex; gap: 8px; align-items: center;",
                if is_locked_record {
                    span { class: "badge badge-amber", "🔒 Hồ sơ đã ký khóa" }
                } else if !can_edit_role {
                    span { class: "badge", style: "background: #f1f5f9; color: #64748b;", "👁 Chỉ xem (Không có quyền mục này)" }
                } else if is_new_record || is_editing {
                    if !is_new_record {
                        button { class: "btn", onclick: move |_| on_cancel.call(()), "Hủy" }
                    }
                    button { class: "btn btn-primary", onclick: move |_| on_save.call(()), "💾 Lưu mục này" }
                } else {
                    button { class: "btn btn-blue", onclick: move |_| on_edit.call(()), "✏️ Sửa" }
                }
            }
        }
    }
}

#[component]
pub fn TextField(
    label: String,
    field_key: String,
    form: Signal<Map<String, Value>>,
    enabled: bool,
    #[props(default = String::new())] hint: String,
) -> Element {
    let val = map_get(&form.read(), &field_key);
    let k = field_key.clone();
    rsx! {
        div { class: "field",
            label { "{label}" }
            input {
                r#type: "text",
                value: "{val}",
                placeholder: "{hint}",
                disabled: !enabled,
                oninput: move |evt| {
                    form_set_key(&mut form, &k, evt.value());
                }
            }
        }
    }
}

#[component]
pub fn TextAreaField(
    label: String,
    field_key: String,
    form: Signal<Map<String, Value>>,
    enabled: bool,
    #[props(default = 2)] rows: i64,
    #[props(default = String::new())] hint: String,
) -> Element {
    let val = map_get(&form.read(), &field_key);
    let k = field_key.clone();
    rsx! {
        div { class: "field",
            label { "{label}" }
            textarea {
                rows: "{rows}",
                value: "{val}",
                placeholder: "{hint}",
                disabled: !enabled,
                oninput: move |evt| {
                    form_set_key(&mut form, &k, evt.value());
                }
            }
        }
    }
}

#[component]
pub fn SelectField(
    label: String,
    field_key: String,
    form: Signal<Map<String, Value>>,
    options: Vec<(String, String)>,
    enabled: bool,
) -> Element {
    let val = map_get(&form.read(), &field_key);
    let k = field_key.clone();
    rsx! {
        div { class: "field",
            label { "{label}" }
            select {
                value: "{val}",
                disabled: !enabled,
                onchange: move |evt| {
                    form_set_key(&mut form, &k, evt.value());
                },
                for (code, name) in options {
                    option { value: "{code}", selected: val == code, "{name}" }
                }
            }
        }
    }
}

#[component]
pub fn CatalogSelect(
    label: String,
    field_key: String,
    form: Signal<Map<String, Value>>,
    items: Vec<CatalogItem>,
    #[props(default = String::new())] parent_filter: String,
    enabled: bool,
) -> Element {
    let mut search = use_signal(String::new);
    let current = map_get(&form.read(), &field_key);
    let q = search.read().trim().to_lowercase();

    let filtered: Vec<CatalogItem> = items
        .iter()
        .filter(|it| it.is_active != 0)
        .filter(|it| parent_filter.is_empty() || it.parent_code == parent_filter)
        .filter(|it| {
            q.is_empty()
                || it.name.to_lowercase().contains(&q)
                || it.code.to_lowercase().contains(&q)
                || it.code == current
        })
        .take(150)
        .cloned()
        .collect();

    let k = field_key.clone();
    rsx! {
        div { class: "field",
            label { "{label}" }
            div { style: "display: flex; gap: 4px;",
                if enabled && items.len() > 15 {
                    input {
                        r#type: "text",
                        style: "width: 38%;",
                        placeholder: "🔍 Lọc...",
                        value: "{search}",
                        oninput: move |e| search.set(e.value()),
                    }
                }
                select {
                    value: "{current}",
                    disabled: !enabled,
                    onchange: move |evt| {
                        form_set_key(&mut form, &k, evt.value());
                    },
                    option { value: "", "-- Chọn danh mục --" }
                    for it in filtered {
                        option {
                            value: "{it.code}",
                            selected: current == it.code,
                            "{it.name} ({it.code})"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn IcdMultiField(
    label: String,
    field_key: String,
    form: Signal<Map<String, Value>>,
    icd_items: Vec<CatalogItem>,
    enabled: bool,
) -> Element {
    let mut q = use_signal(String::new);
    let current = map_get(&form.read(), &field_key);
    let ql = q.read().trim().to_lowercase();

    let matches: Vec<CatalogItem> = if enabled && !ql.is_empty() {
        icd_items
            .iter()
            .filter(|it| it.is_active != 0)
            .filter(|it| it.code.to_lowercase().contains(&ql) || it.name.to_lowercase().contains(&ql))
            .take(12)
            .cloned()
            .collect()
    } else {
        Vec::new()
    };

    let k1 = field_key.clone();
    let k2 = field_key.clone();
    rsx! {
        div { class: "field", style: "position: relative;",
            label { "{label}" }
            div { style: "display: flex; gap: 6px;",
                input {
                    r#type: "text",
                    value: "{current}",
                    placeholder: "Mã ICD-10 (vd: I10; E11)",
                    disabled: !enabled,
                    oninput: move |evt| form_set_key(&mut form, &k1, evt.value()),
                }
                if enabled {
                    input {
                        r#type: "text",
                        style: "width: 150px;",
                        placeholder: "+ Tìm ICD-10...",
                        value: "{q}",
                        oninput: move |evt| q.set(evt.value()),
                    }
                }
            }
            if !matches.is_empty() {
                div {
                    style: "position: absolute; top: 100%; right: 0; width: 340px; max-height: 200px; overflow-y: auto; background: #fff; border: 1px solid #cbd5e1; border-radius: 6px; box-shadow: 0 6px 16px rgba(0,0,0,0.12); z-index: 30;",
                    for it in matches {
                        {
                            let k_item = k2.clone();
                            rsx! {
                                button {
                                    r#type: "button",
                                    style: "display: block; width: 100%; text-align: left; padding: 6px 10px; border: none; background: transparent; cursor: pointer; font-size: 12px; border-bottom: 1px solid #f1f5f9;",
                                    onclick: move |_| {
                                        let cur = map_get(&form.read(), &k_item);
                                        let mut parts: Vec<String> = cur
                                            .split(';')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        if !parts.contains(&it.code) {
                                            parts.push(it.code.clone());
                                            form_set_key(&mut form, &k_item, parts.join("; "));
                                        }
                                        q.set(String::new());
                                    },
                                    "{it.code} - {it.name}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

