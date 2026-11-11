use crate::config::AppConfig;
use crate::db::{get_db_info, hash_password, init_db, seed_catalogs_from_asset};
use crate::models::*;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Datelike, Local};
use quick_xml::{
    events::{BytesText, Event},
    Reader, Writer,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    fs,
    io::{Cursor, Read, Write},
    sync::Arc,
    time::Duration,
};
use tokio::sync::RwLock;
use uuid::Uuid;

const EMBEDDED_TEMPLATE_DOCX: &[u8] = include_bytes!("../asset/template18.docx");

pub struct AppState {
    pub db: RwLock<SqlitePool>,
    pub config: RwLock<AppConfig>,
    pub http_client: reqwest::Client,
}

pub type SharedState = Arc<AppState>;

pub fn api_error(status: StatusCode, msg: impl Into<String>) -> Response {
    let body = Json(json!({
        "status": status.as_u16(),
        "error": msg.into()
    }));
    (status, body).into_response()
}

pub async fn authenticate(
    pool: &SqlitePool,
    headers: &HeaderMap,
    token_param: Option<&str>,
) -> Result<User, Response> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string())
        .or_else(|| token_param.map(|s| s.trim().to_string()))
        .ok_or_else(|| api_error(StatusCode::UNAUTHORIZED, "Chưa đăng nhập hoặc thiếu token xác thực"))?;

    let user: Option<User> = sqlx::query_as(
        r#"
        SELECT u.id, u.username, u.password_hash, u.full_name, u.role, u.is_locked, u.created_at
        FROM sessions s
        JOIN users u ON u.id = s.user_id
        WHERE s.token = ?
        "#,
    )
    .bind(&token)
    .fetch_optional(pool)
    .await
    .map_err(|e| api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi truy vấn phiên: {}", e)))?;

    match user {
        Some(u) if u.is_locked == 0 => Ok(u),
        Some(_) => Err(api_error(
            StatusCode::FORBIDDEN,
            "Tài khoản của bạn đã bị khóa bởi Quản trị viên",
        )),
        None => Err(api_error(
            StatusCode::UNAUTHORIZED,
            "Phiên đăng nhập không hợp lệ hoặc đã hết hạn",
        )),
    }
}

// ============================================================================
// 1. AUTHENTICATION & USER MANAGEMENT (MULTI-ROLE RBAC)
// ============================================================================

pub async fn login_handler(
    State(state): State<SharedState>,
    Json(req): Json<LoginRequest>,
) -> Response {
    let username = req.username.trim();
    let p_hash = hash_password(req.password.trim());
    let db = state.db.read().await;

    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password_hash, full_name, role, is_locked, created_at FROM users WHERE username = ?",
    )
    .bind(username)
    .fetch_optional(&*db)
    .await
    .unwrap_or(None);

    let Some(user) = user else {
        return api_error(StatusCode::UNAUTHORIZED, "Tên đăng nhập hoặc mật khẩu không đúng");
    };

    if user.password_hash != p_hash {
        return api_error(StatusCode::UNAUTHORIZED, "Tên đăng nhập hoặc mật khẩu không đúng");
    }

    if user.is_locked != 0 {
        return api_error(
            StatusCode::FORBIDDEN,
            "Tài khoản này đã bị khóa. Vui lòng liên hệ Quản trị viên.",
        );
    }

    let token = Uuid::new_v4().to_string();
    if let Err(e) = sqlx::query("INSERT INTO sessions (token, user_id) VALUES (?, ?)")
        .bind(&token)
        .bind(user.id)
        .execute(&*db)
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Không thể tạo phiên đăng nhập: {}", e),
        );
    }

    let roles = user.roles();
    Json(LoginResponse { token, user, roles }).into_response()
}

pub async fn me_handler(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    let cfg = state.config.read().await;
    let roles = user.roles();

    Json(json!({
        "user": user,
        "roles": roles,
        "facility": {
            "ma_cskcb": cfg.ma_cskcb,
            "ma_gtin_cskcb": cfg.ma_gtin_cskcb,
            "ten_cskcb": cfg.ten_cskcb,
            "dia_danh": cfg.dia_danh,
        }
    }))
    .into_response()
}

pub async fn list_users_handler(State(state): State<SharedState>, headers: HeaderMap) -> Response {
    let db = state.db.read().await;
    let current_user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !current_user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên (admin) mới có quyền xem danh sách tài khoản");
    }

    let users: Vec<User> = sqlx::query_as(
        "SELECT id, username, password_hash, full_name, role, is_locked, created_at FROM users ORDER BY id ASC",
    )
    .fetch_all(&*db)
    .await
    .unwrap_or_default();

    let list: Vec<Value> = users
        .into_iter()
        .map(|u| {
            let roles = u.roles();
            json!({
                "id": u.id,
                "username": u.username,
                "full_name": u.full_name,
                "role": u.role,
                "roles": roles,
                "is_locked": u.is_locked,
                "created_at": u.created_at
            })
        })
        .collect();

    Json(list).into_response()
}

pub async fn create_user_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<CreateUserRequest>,
) -> Response {
    let db = state.db.read().await;
    let current_user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !current_user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền tạo tài khoản");
    }

    let username = req.username.trim().to_lowercase();
    let full_name = req.full_name.trim();

    if username.is_empty() || req.password.trim().is_empty() || full_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Vui lòng nhập đầy đủ tên đăng nhập, mật khẩu và họ tên");
    }

    let valid_selected: Vec<&str> = req
        .roles
        .iter()
        .map(|s| s.trim())
        .filter(|s| VALID_ROLES.contains(s))
        .collect();

    if valid_selected.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Vui lòng chọn ít nhất một quyền (Role) hợp lệ");
    }

    let role_csv = valid_selected.join(",");
    let p_hash = hash_password(req.password.trim());
    let res = sqlx::query(
        "INSERT INTO users (username, password_hash, full_name, role, is_locked) VALUES (?, ?, ?, ?, 0)",
    )
    .bind(&username)
    .bind(&p_hash)
    .bind(full_name)
    .bind(&role_csv)
    .execute(&*db)
    .await;

    match res {
        Ok(r) => Json(json!({
            "message": "Tạo tài khoản thành công",
            "id": r.last_insert_rowid()
        }))
        .into_response(),
        Err(e) => api_error(
            StatusCode::BAD_REQUEST,
            format!("Không thể tạo tài khoản (có thể trùng username): {}", e),
        ),
    }
}

pub async fn update_user_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
    Json(req): Json<UpdateUserRequest>,
) -> Response {
    let db = state.db.read().await;
    let current_user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !current_user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền cập nhật tài khoản");
    }

    let valid_selected: Vec<&str> = req
        .roles
        .iter()
        .map(|s| s.trim())
        .filter(|s| VALID_ROLES.contains(s))
        .collect();

    if valid_selected.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Vui lòng chọn ít nhất một quyền (Role) hợp lệ");
    }
    let role_csv = valid_selected.join(",");

    if let Some(pw) = &req.password {
        if !pw.trim().is_empty() {
            let p_hash = hash_password(pw.trim());
            let _ = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
                .bind(p_hash)
                .bind(user_id)
                .execute(&*db)
                .await;
        }
    }

    if let Some(locked) = req.is_locked {
        let _ = sqlx::query("UPDATE users SET is_locked = ? WHERE id = ?")
            .bind(locked)
            .bind(user_id)
            .execute(&*db)
            .await;
        if locked != 0 {
            let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
                .bind(user_id)
                .execute(&*db)
                .await;
        }
    }

    let res = sqlx::query("UPDATE users SET full_name = ?, role = ? WHERE id = ?")
        .bind(req.full_name.trim())
        .bind(&role_csv)
        .bind(user_id)
        .execute(&*db)
        .await;

    match res {
        Ok(_) => Json(json!({ "message": "Cập nhật tài khoản thành công" })).into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi cập nhật: {}", e)),
    }
}

pub async fn toggle_lock_user_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(user_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let current_user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !current_user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền khóa/mở khóa tài khoản");
    }
    if current_user.id == user_id {
        return api_error(StatusCode::BAD_REQUEST, "Không thể tự khóa tài khoản của chính mình");
    }

    let user: Option<User> = sqlx::query_as(
        "SELECT id, username, password_hash, full_name, role, is_locked, created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_optional(&*db)
    .await
    .unwrap_or(None);

    let Some(u) = user else {
        return api_error(StatusCode::NOT_FOUND, "Không tìm thấy tài khoản");
    };

    let new_status = if u.is_locked == 0 { 1 } else { 0 };
    let _ = sqlx::query("UPDATE users SET is_locked = ? WHERE id = ?")
        .bind(new_status)
        .bind(user_id)
        .execute(&*db)
        .await;

    if new_status == 1 {
        let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(&*db)
            .await;
    }

    Json(json!({
        "message": if new_status == 1 { "Đã khóa tài khoản" } else { "Đã mở khóa tài khoản" },
        "is_locked": new_status
    }))
    .into_response()
}

pub async fn change_my_password_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let old_p_hash = hash_password(req.old_password.trim());
    if user.password_hash != old_p_hash {
        return api_error(StatusCode::BAD_REQUEST, "Mật khẩu hiện tại không đúng");
    }

    let new_pw = req.new_password.trim();
    if new_pw.len() < 6 {
        return api_error(StatusCode::BAD_REQUEST, "Mật khẩu mới phải có ít nhất 6 ký tự");
    }

    let new_p_hash = hash_password(new_pw);
    if let Err(e) = sqlx::query("UPDATE users SET password_hash = ? WHERE id = ?")
        .bind(&new_p_hash)
        .bind(user.id)
        .execute(&*db)
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Không thể cập nhật mật khẩu: {}", e),
        );
    }

    if req.revoke_other_sessions.unwrap_or(false) {
        let cur_token = headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let _ = sqlx::query("DELETE FROM sessions WHERE user_id = ? AND token != ?")
            .bind(user.id)
            .bind(cur_token)
            .execute(&*db)
            .await;
    }

    Json(json!({
        "message": "Đổi mật khẩu thành công! Vui lòng sử dụng mật khẩu mới cho các lần đăng nhập tiếp theo."
    }))
    .into_response()
}

pub async fn update_my_profile_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<UpdateProfileRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    // Chỉ có tài khoản có quyền admin mới được phép tự đổi họ tên/chức danh
    let is_admin = user
        .role
        .split(',')
        .map(|r| r.trim())
        .any(|r| r.eq_ignore_ascii_case("admin"));

    if !is_admin {
        return api_error(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên (Admin) mới có quyền thay đổi họ và tên / chức danh người dùng.",
        );
    }

    let full_name = req.full_name.trim();
    if full_name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Họ tên / Chức danh không được để trống");
    }

    if let Err(e) = sqlx::query("UPDATE users SET full_name = ? WHERE id = ?")
        .bind(full_name)
        .bind(user.id)
        .execute(&*db)
        .await
    {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi cập nhật họ tên / chức danh: {}", e),
        );
    }

    Json(json!({
        "message": "Cập nhật họ tên / chức danh thành công!",
        "full_name": full_name
    }))
    .into_response()
}

// ============================================================================
// 2. TÍCH HỢP API HIS
// ============================================================================

pub async fn fetch_his_patient_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<HisQuery>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let ma_tn = query.ma_tn.trim().to_string();
    if ma_tn.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Vui lòng nhập Mã tiếp nhận (ma_tn)");
    }

    let cfg = state.config.read().await.clone();

    if cfg.his_use_mock {
        return execute_mock_his(&ma_tn);
    }

    let req_body_str = cfg.his_payload_template.replace("{{ma_tn}}", &ma_tn);

    let method = if cfg.his_http_method.eq_ignore_ascii_case("GET") {
        reqwest::Method::GET
    } else {
        reqwest::Method::POST
    };

    let mut req_builder = state.http_client.request(method, &cfg.his_api_url);

    if !cfg.his_api_key.is_empty() {
        let auth_hdr = if cfg.his_auth_header.is_empty() { "x-api-key" } else { &cfg.his_auth_header };
        req_builder = req_builder.header(auth_hdr, &cfg.his_api_key);
        if auth_hdr.eq_ignore_ascii_case("authorization") && !cfg.his_api_key.starts_with("Bearer ") {
            req_builder = req_builder.header(header::AUTHORIZATION, format!("Bearer {}", cfg.his_api_key));
        }
    }

    if cfg.his_http_method.eq_ignore_ascii_case("POST") {
        req_builder = req_builder
            .header("Content-Type", "application/json")
            .body(req_body_str);
    }

    let his_resp = req_builder
        .timeout(Duration::from_secs(cfg.his_timeout_seconds))
        .send()
        .await;

    match his_resp {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let err_txt = resp.text().await.unwrap_or_default();
                return api_error(
                    StatusCode::BAD_GATEWAY,
                    format!("API HIS trả về mã lỗi {}: {}", status, err_txt),
                );
            }
            match resp.json::<HisResponse>().await {
                Ok(parsed) => Json(parsed).into_response(),
                Err(e) => api_error(
                    StatusCode::BAD_GATEWAY,
                    format!("Dữ liệu JSON từ HIS không đúng chuẩn: {}", e),
                ),
            }
        }
        Err(e) => api_error(
            StatusCode::BAD_GATEWAY,
            format!("Không thể kết nối đến API HIS ({}): {}", cfg.his_api_url, e),
        ),
    }
}

pub fn execute_mock_his(ma_tn: &str) -> Response {
    let today = Local::now().format("%d/%m/%Y").to_string();
    let data = match ma_tn {
        "TN.2611.0004018" => HisPatientData {
            ma_tn: "TN.2611.0004018".to_string(),
            ma_y_te: "23005512".to_string(),
            ho_ten: "NGUYỄN VĂN A".to_string(),
            gioi_tinh: 1,
            ngay_sinh: "31/01/2000".to_string(),
            so_cccd: "026099997785".to_string(),
            ngaycap_cccd: "13/10/2021".to_string(),
            noicap_cccd: "Cục Cảnh sát QLHC về TTXH".to_string(),
            ma_dan_toc: "01".to_string(),
            ma_nghe_nghiep: "00".to_string(),
            so_dien_thoai: "0977889123".to_string(),
            dia_chi: "Số 123 Đường B".to_string(),
            matinh_cu_tru: "11221".to_string(),
            maxa_cu_tru: "13338".to_string(),
            ngay_vao: "09/11/2026".to_string(),
        },
        "TN.2611.0004019" => HisPatientData {
            ma_tn: "TN.2611.0004019".to_string(),
            ma_y_te: "23005519".to_string(),
            ho_ten: "TRẦN THỊ BÍCH NGỌC".to_string(),
            gioi_tinh: 2,
            ngay_sinh: "15/08/1995".to_string(),
            so_cccd: "052195004321".to_string(),
            ngaycap_cccd: "20/05/2022".to_string(),
            noicap_cccd: "Cục Cảnh sát QLHC về TTXH".to_string(),
            ma_dan_toc: "01".to_string(),
            ma_nghe_nghiep: "00".to_string(),
            so_dien_thoai: "0912345678".to_string(),
            dia_chi: "Số 45 Nguyễn Tất Thành".to_string(),
            matinh_cu_tru: "11221".to_string(),
            maxa_cu_tru: "13339".to_string(),
            ngay_vao: today,
        },
        other => HisPatientData {
            ma_tn: other.to_string(),
            ma_y_te: format!("2600{}", &other.chars().filter(|c| c.is_ascii_digit()).collect::<String>()),
            ho_ten: "LÊ HOÀNG NAM".to_string(),
            gioi_tinh: 1,
            ngay_sinh: "10/05/1992".to_string(),
            so_cccd: "052092008899".to_string(),
            ngaycap_cccd: "12/12/2021".to_string(),
            noicap_cccd: "Cục Cảnh sát QLHC VTTXH".to_string(),
            ma_dan_toc: "01".to_string(),
            ma_nghe_nghiep: "00".to_string(),
            so_dien_thoai: "0988776655".to_string(),
            dia_chi: "Khu phố 3, Phường An Bình".to_string(),
            matinh_cu_tru: "11221".to_string(),
            maxa_cu_tru: "13338".to_string(),
            ngay_vao: today,
        },
    };

    Json(HisResponse {
        status: 200,
        message: "success".to_string(),
        data: Some(data),
    })
    .into_response()
}

pub async fn mock_his_api_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<HisRequest>,
) -> Response {
    let cfg = state.config.read().await;
    let provided_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .or_else(|| {
            headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
        })
        .unwrap_or("");

    if !cfg.his_api_key.is_empty() && provided_key != cfg.his_api_key {
        return (
            StatusCode::UNAUTHORIZED,
            Json(HisResponse {
                status: 401,
                message: "Invalid API Key".to_string(),
                data: None,
            }),
        )
            .into_response();
    }

    execute_mock_his(req.ma_tiep_nhan.trim())
}

pub async fn test_his_connection_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<TestHisRequest>,
) -> Response {
    let db = state.db.read().await;
    let current_user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !current_user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới được test kết nối HIS");
    }

    let cfg = state.config.read().await.clone();
    let api_url = req.api_url.unwrap_or(cfg.his_api_url);
    let method_str = req.http_method.unwrap_or(cfg.his_http_method);
    let auth_hdr = req.auth_header.unwrap_or(cfg.his_auth_header);
    let api_key = req.api_key.unwrap_or(cfg.his_api_key);
    let payload_tmpl = req.payload_template.unwrap_or(cfg.his_payload_template);
    let ma_tn = req.ma_tn.trim();

    let method = if method_str.eq_ignore_ascii_case("GET") {
        reqwest::Method::GET
    } else {
        reqwest::Method::POST
    };

    let mut req_builder = state.http_client.request(method, &api_url);
    if !api_key.is_empty() {
        req_builder = req_builder.header(&auth_hdr, &api_key);
    }
    let body_str = payload_tmpl.replace("{{ma_tn}}", ma_tn);
    if method_str.eq_ignore_ascii_case("POST") {
        req_builder = req_builder
            .header("Content-Type", "application/json")
            .body(body_str.clone());
    }

    let start = std::time::Instant::now();
    let res = req_builder.timeout(Duration::from_secs(8)).send().await;
    let elapsed_ms = start.elapsed().as_millis();

    match res {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            Json(json!({
                "status": status,
                "elapsed_ms": elapsed_ms,
                "request": {
                    "url": api_url,
                    "method": method_str,
                    "body": body_str
                },
                "response_text": text
            }))
            .into_response()
        }
        Err(e) => api_error(
            StatusCode::BAD_GATEWAY,
            format!("Lỗi kết nối ({}ms): {}", elapsed_ms, e),
        ),
    }
}

// ============================================================================
// 3. QUẢN LÝ HỒ SƠ KHÁM SỨC KHỎE
// ============================================================================

fn normalize_filter_date(d: &str) -> String {
    let s = d.trim();
    if s.is_ascii() && s.len() == 10 && &s[2..3] == "/" && &s[5..6] == "/" {
        format!("{}-{}-{}", &s[6..10], &s[3..5], &s[0..2])
    } else {
        s.to_string()
    }
}

fn json_val_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.trim().to_string(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
        _ => String::new(),
    }
}

async fn query_filtered_records(
    db: &SqlitePool,
    from_date: Option<&str>,
    to_date: Option<&str>,
    status: Option<i64>,
    q: Option<&str>,
    workplace: Option<&str>,
    classification: Option<&str>,
    selected_ids: Option<&str>,
    limit: Option<i64>,
    order_asc: bool,
) -> Vec<KskRecordRow> {
    let q_str = q.unwrap_or_default().trim().to_string();
    let from_str = normalize_filter_date(from_date.unwrap_or_default());
    let to_str = normalize_filter_date(to_date.unwrap_or_default());
    let workplace_str = workplace.unwrap_or_default().trim().to_lowercase();
    let class_str = classification.unwrap_or_default().trim().to_string();

    let mut parsed_ids: Option<Vec<i64>> = None;
    if let Some(raw_ids) = selected_ids {
        let trimmed = raw_ids.trim();
        if !trimmed.is_empty() {
            let ids: Vec<i64> = trimmed
                .split(',')
                .filter_map(|part| part.trim().parse::<i64>().ok())
                .collect();
            if ids.is_empty() {
                return Vec::new();
            }
            parsed_ids = Some(ids);
        }
    }

    let mut sql = String::from("SELECT * FROM ksk_records WHERE 1=1");
    let mut conditions: Vec<String> = Vec::new();

    if status.is_some() {
        conditions.push("status = ?".to_string());
    }

    if !from_str.is_empty() {
        conditions.push("(CASE WHEN length(ngay_vao) = 10 AND substr(ngay_vao, 3, 1) = '/' AND substr(ngay_vao, 6, 1) = '/' THEN (substr(ngay_vao, 7, 4) || '-' || substr(ngay_vao, 4, 2) || '-' || substr(ngay_vao, 1, 2)) ELSE substr(created_at, 1, 10) END) >= ?".to_string());
    }

    if !to_str.is_empty() {
        conditions.push("(CASE WHEN length(ngay_vao) = 10 AND substr(ngay_vao, 3, 1) = '/' AND substr(ngay_vao, 6, 1) = '/' THEN (substr(ngay_vao, 7, 4) || '-' || substr(ngay_vao, 4, 2) || '-' || substr(ngay_vao, 1, 2)) ELSE substr(created_at, 1, 10) END) <= ?".to_string());
    }

    if !q_str.is_empty() {
        conditions.push("(ma_tiep_nhan LIKE ? OR ho_ten LIKE ? OR so_cccd LIKE ? OR ma_y_te LIKE ?)".to_string());
    }

    if let Some(ref ids) = parsed_ids {
        let placeholders = vec!["?"; ids.len()].join(", ");
        conditions.push(format!("id IN ({})", placeholders));
    }

    for cond in conditions {
        sql.push_str(" AND ");
        sql.push_str(&cond);
    }

    if order_asc {
        sql.push_str(" ORDER BY id ASC");
    } else {
        sql.push_str(" ORDER BY updated_at DESC, id DESC");
    }

    // Nếu không cần lọc JSON trên Rust và có limit thì đẩy LIMIT xuống SQL
    let has_rust_filter = !workplace_str.is_empty() || !class_str.is_empty();
    if !has_rust_filter {
        if let Some(lim) = limit {
            let safe_lim = lim.clamp(1, 5000);
            sql.push_str(&format!(" LIMIT {}", safe_lim));
        }
    }

    let mut query = sqlx::query_as::<_, KskRecordRow>(&sql);

    if let Some(st) = status {
        query = query.bind(st);
    }
    if !from_str.is_empty() {
        query = query.bind(from_str);
    }
    if !to_str.is_empty() {
        query = query.bind(to_str);
    }
    if !q_str.is_empty() {
        let like_q = format!("%{}%", q_str);
        query = query
            .bind(like_q.clone())
            .bind(like_q.clone())
            .bind(like_q.clone())
            .bind(like_q);
    }
    if let Some(ref ids) = parsed_ids {
        for id_val in ids {
            query = query.bind(*id_val);
        }
    }

    let mut rows: Vec<KskRecordRow> = query.fetch_all(db).await.unwrap_or_default();

    if has_rust_filter {
        rows.retain(|r| {
            let map: Map<String, Value> = serde_json::from_str(&r.payload_json).unwrap_or_default();
            if !workplace_str.is_empty() {
                let wp = map
                    .get("NOI_LAM_VIEC_HOC_TAP")
                    .map(json_val_to_string)
                    .unwrap_or_default()
                    .to_lowercase();
                if !wp.contains(&workplace_str) {
                    return false;
                }
            }
            if !class_str.is_empty() {
                let pl = map
                    .get("PHAN_LOAI_SK")
                    .map(json_val_to_string)
                    .unwrap_or_default();
                if pl != class_str {
                    return false;
                }
            }
            true
        });
        if let Some(lim) = limit {
            let safe_lim = lim.clamp(1, 5000) as usize;
            rows.truncate(safe_lim);
        }
    }

    rows
}

pub async fn list_records_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(filter): Query<RecordFilterQuery>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let rows = query_filtered_records(
        &*db,
        filter.from_date.as_deref(),
        filter.to_date.as_deref(),
        filter.status,
        filter.q.as_deref(),
        filter.workplace.as_deref(),
        filter.classification.as_deref(),
        None,
        Some(filter.limit.unwrap_or(200)),
        false,
    )
    .await;

    let dtos: Vec<KskRecordDto> = rows.into_iter().map(KskRecordDto::from).collect();
    Json(dtos).into_response()
}

pub async fn get_record_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(record_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let row: Option<KskRecordRow> = sqlx::query_as("SELECT * FROM ksk_records WHERE id = ?")
        .bind(record_id)
        .fetch_optional(&*db)
        .await
        .unwrap_or(None);

    match row {
        Some(r) => Json(KskRecordDto::from(r)).into_response(),
        None => api_error(StatusCode::NOT_FOUND, "Không tìm thấy hồ sơ KSK"),
    }
}

pub async fn get_record_by_ma_tn_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(ma_tn): Path<String>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let row: Option<KskRecordRow> =
        sqlx::query_as("SELECT * FROM ksk_records WHERE ma_tiep_nhan = ?")
            .bind(ma_tn.trim())
            .fetch_optional(&*db)
            .await
            .unwrap_or(None);

    match row {
        Some(r) => Json(KskRecordDto::from(r)).into_response(),
        None => api_error(StatusCode::NOT_FOUND, "Chưa có hồ sơ với Mã tiếp nhận này"),
    }
}

pub async fn save_record_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<SaveRecordRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    let cfg = state.config.read().await.clone();

    let existing: Option<KskRecordRow> = if let Some(id) = req.id {
        sqlx::query_as("SELECT * FROM ksk_records WHERE id = ?")
            .bind(id)
            .fetch_optional(&*db)
            .await
            .unwrap_or(None)
    } else if let Some(ma_tn) = &req.ma_tiep_nhan {
        if !ma_tn.trim().is_empty() {
            sqlx::query_as("SELECT * FROM ksk_records WHERE ma_tiep_nhan = ?")
                .bind(ma_tn.trim())
                .fetch_optional(&*db)
                .await
                .unwrap_or(None)
        } else {
            None
        }
    } else {
        None
    };

    if let Some(ref rec) = existing {
        if rec.status == 1 {
            return api_error(
                StatusCode::FORBIDDEN,
                "Hồ sơ này đã được Ký / Khóa (status = 1). Chỉ Admin mới có quyền Mở khóa để chỉnh sửa.",
            );
        }

        // 1. Phân quyền theo Section nếu client chỉ định
        if let Some(sec) = req.section.as_deref() {
            if sec == "clinical_all" {
                let has_clinical = user.has_role("admin")
                    || user.has_role("bs_noi")
                    || user.has_role("bs_ngoai")
                    || user.has_role("bs_dalieu")
                    || user.has_role("bs_san")
                    || user.has_role("bs_mat")
                    || user.has_role("bs_tmh")
                    || user.has_role("bs_rhm");
                if !has_clinical {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        "Tài khoản của bạn không có quyền phụ trách bất kỳ chuyên khoa lâm sàng nào.",
                    );
                }
            } else if let Some(req_role) = get_section_required_role(sec) {
                if !user.has_role(req_role) {
                    return api_error(
                        StatusCode::FORBIDDEN,
                        format!(
                            "Bạn không có quyền chỉnh sửa mục [{}]. Vui lòng liên hệ đúng bác sĩ/nhân viên phụ trách.",
                            get_section_label(sec)
                        ),
                    );
                }
            }
        }

        // 2. Kiểm tra sâu diff từng trường dữ liệu: Tuyệt đối không cho phép sửa đổi trường ngoài quyền hạn
        let existing_map: Map<String, Value> = serde_json::from_str(&rec.payload_json).unwrap_or_default();
        let target_section_fields = req.section.as_deref().map(get_section_fields);

        for (k, v) in &req.payload {
            // Bỏ qua các trường hệ thống không thuộc bệnh án
            if k == "id" || k == "MA_CSKCB" || k == "MA_GTIN_CSKCB" || k == "status" || k == "created_at" || k == "updated_at" || k == "updated_by" {
                continue;
            }
            if k == "MA_TIEP_NHAN" {
                let new_tn = v.as_str().unwrap_or("").trim();
                if new_tn == rec.ma_tiep_nhan {
                    continue;
                }
            }

            // Nếu người dùng gửi theo section cụ thể, các trường ngoài section đó không áp dụng
            if let Some(sec_fields) = target_section_fields {
                if !sec_fields.is_empty() && !sec_fields.contains(&k.as_str()) {
                    continue;
                }
            }

            let old_val = existing_map.get(k).and_then(|x| x.as_str()).unwrap_or("").trim();
            let new_val_str = match v {
                Value::String(s) => s.trim().to_string(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                _ => String::new(),
            };
            let new_val = new_val_str.trim();

            // Giá trị không thay đổi thì bỏ qua
            if old_val == new_val {
                continue;
            }

            // Giá trị có thay đổi: Bắt buộc user phải có quyền sửa trường này
            if !can_user_roles_edit_field(&user.role, k) {
                let sec_name = field_section_name(k);
                return api_error(
                    StatusCode::FORBIDDEN,
                    format!(
                        "Từ chối quyền chỉnh sửa: Tài khoản của bạn không có quyền sửa trường '{}' thuộc mục [{}].",
                        k, sec_name
                    ),
                );
            }
        }
    } else if !user.has_role("admin") && !user.has_role("reception") {
        return api_error(
            StatusCode::FORBIDDEN,
            "Chỉ nhân viên Lễ tân hoặc Quản trị viên mới được phép tiếp nhận tạo mới hồ sơ KSK.",
        );
    }

    let mut merged = merge_payload_by_roles(
        existing.as_ref().map(|r| r.payload_json.as_str()),
        &req.payload,
        &user.role,
        &cfg.ma_cskcb,
        &cfg.ma_gtin_cskcb,
        req.section.as_deref(),
    );

    // Tự động điền tên bác sĩ chuyên khoa: Chỉ gán khi chuyên khoa đó có dữ liệu được nhập/chỉnh sửa
    let doctor_mappings = [
        ("bs_theluc", "BS_THE_LUC", "CHIEU_CAO"),
        ("bs_noi", "BS_NOI", "NOI_KHOA_TUAN_HOAN"),
        ("bs_ngoai", "BS_NGOAI", "KET_QUA_KHAM_NGOAI_KHOA"),
        ("bs_dalieu", "BS_DA_LIEU", "KET_QUA_KHAM_DA_LIEU"),
        ("bs_san", "BS_SAN", "KET_QUA_KHAM_SAN_PHU_KHOA"),
        ("bs_mat", "BS_MAT", "KHONG_KINH_MAT_PHAI"),
        ("bs_tmh", "BS_TMH", "TAI_TRAI_NOI_THUONG"),
        ("bs_rhm", "BS_RHM", "HAM_TREN"),
        ("bs_cls", "BS_CLS", "CLS_HUYET_HOC"),
        ("bs_ketluan", "BS_KET_LUAN", "KET_LUAN_BENH"),
    ];

    for (role_tag, field_name, sample_field) in doctor_mappings {
        // Chỉ tự động điền tên Bác sĩ khi chuyên khoa đó thực sự có dữ liệu
        let has_content = merged.get(sample_field).and_then(|v| v.as_str()).map(|s| !s.trim().is_empty()).unwrap_or(false)
            || (role_tag == "bs_ketluan" && merged.get("PHAN_LOAI_SK").and_then(|v| v.as_str()).map(|s| !s.trim().is_empty()).unwrap_or(false));
        if has_content {
            if user.role.split(',').any(|r| r.trim() == role_tag) {
                merged.insert(field_name.to_string(), Value::String(user.full_name.clone()));
            } else if user.has_role(role_tag) {
                let cur = merged.get(field_name).and_then(|v| v.as_str()).unwrap_or("").trim();
                if cur.is_empty() {
                    merged.insert(field_name.to_string(), Value::String(user.full_name.clone()));
                }
            }
        }
    }

    let ma_tn = req
        .ma_tiep_nhan
        .as_deref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            merged
                .get("MA_TIEP_NHAN")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
        .or_else(|| existing.as_ref().map(|r| r.ma_tiep_nhan.clone()))
        .unwrap_or_default();

    if ma_tn.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Mã tiếp nhận không được để trống");
    }

    merged.insert("MA_TIEP_NHAN".to_string(), Value::String(ma_tn.clone()));

    fn get_val_str(m: &Map<String, Value>, key: &str) -> String {
        m.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    }

    let ma_y_te = get_val_str(&merged, "MA_Y_TE");
    let ho_ten = get_val_str(&merged, "HO_TEN").to_uppercase();
    merged.insert("HO_TEN".to_string(), Value::String(ho_ten.clone()));
    let gioi_tinh = {
        let g = get_val_str(&merged, "GIOI_TINH");
        if g.is_empty() { "1".to_string() } else { g }
    };
    let ngay_sinh = get_val_str(&merged, "NGAY_SINH");
    let so_cccd = get_val_str(&merged, "SO_CCCD");
    let ngay_vao = {
        let nv = get_val_str(&merged, "NGAY_VAO");
        if nv.is_empty() {
            let today = Local::now().format("%d/%m/%Y").to_string();
            merged.insert("NGAY_VAO".to_string(), Value::String(today.clone()));
            today
        } else {
            nv
        }
    };

    let payload_json = serde_json::to_string(&merged).unwrap_or_else(|_| "{}".to_string());

    let record_id = if let Some(rec) = existing {
        if let Err(e) = sqlx::query(
            r#"
            UPDATE ksk_records
            SET ma_tiep_nhan = ?, ma_y_te = ?, ho_ten = ?, gioi_tinh = ?, ngay_sinh = ?,
                so_cccd = ?, ngay_vao = ?, payload_json = ?, updated_by = ?,
                updated_at = datetime('now', 'localtime')
            WHERE id = ?
            "#,
        )
        .bind(&ma_tn)
        .bind(&ma_y_te)
        .bind(&ho_ten)
        .bind(&gioi_tinh)
        .bind(&ngay_sinh)
        .bind(&so_cccd)
        .bind(&ngay_vao)
        .bind(&payload_json)
        .bind(&user.full_name)
        .bind(rec.id)
        .execute(&*db)
        .await
        {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lỗi cập nhật hồ sơ KSK: {}", e),
            );
        }
        rec.id
    } else {
        match sqlx::query(
            r#"
            INSERT INTO ksk_records (
                ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh,
                so_cccd, ngay_vao, status, payload_json, updated_by
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
            "#,
        )
        .bind(&ma_tn)
        .bind(&ma_y_te)
        .bind(&ho_ten)
        .bind(&gioi_tinh)
        .bind(&ngay_sinh)
        .bind(&so_cccd)
        .bind(&ngay_vao)
        .bind(&payload_json)
        .bind(&user.full_name)
        .execute(&*db)
        .await
        {
            Ok(res) => res.last_insert_rowid(),
            Err(e) => {
                return api_error(
                    StatusCode::BAD_REQUEST,
                    format!("Không thể tạo hồ sơ mới (kiểm tra trùng Mã tiếp nhận): {}", e),
                )
            }
        }
    };

    let saved: Option<KskRecordRow> = sqlx::query_as("SELECT * FROM ksk_records WHERE id = ?")
        .bind(record_id)
        .fetch_optional(&*db)
        .await
        .unwrap_or(None);

    match saved {
        Some(r) => Json(KskRecordDto::from(r)).into_response(),
        None => api_error(StatusCode::INTERNAL_SERVER_ERROR, "Không thể đọc lại hồ sơ sau khi lưu"),
    }
}

pub async fn lock_record_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(record_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    if !user.has_role("bs_ketluan") && !user.has_role("admin") {
        return api_error(
            StatusCode::FORBIDDEN,
            "Chỉ Bác sĩ Kết luận (bs_ketluan) hoặc Quản trị viên (admin) mới có quyền Ký / Khóa hồ sơ.",
        );
    }

    let res = sqlx::query(
        "UPDATE ksk_records SET status = 1, updated_by = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
    )
    .bind(&user.full_name)
    .bind(record_id)
    .execute(&*db)
    .await;

    match res {
        Ok(_) => Json(json!({
            "message": "Đã Ký và Khóa hồ sơ thành công (status = 1)",
            "status": 1
        }))
        .into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi khóa hồ sơ: {}", e)),
    }
}

pub async fn unlock_record_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(record_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    if !user.has_role("admin") {
        return api_error(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên (admin) mới có quyền Mở khóa (status = 0) hồ sơ đã khóa.",
        );
    }

    let res = sqlx::query(
        "UPDATE ksk_records SET status = 0, updated_by = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
    )
    .bind(&user.full_name)
    .bind(record_id)
    .execute(&*db)
    .await;

    match res {
        Ok(_) => Json(json!({
            "message": "Đã Mở khóa hồ sơ thành công (status = 0)",
            "status": 0
        }))
        .into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi mở khóa hồ sơ: {}", e)),
    }
}

// ============================================================================
// 4. XUẤT FILE WORD (.DOCX) - GIỮ DẤU "....." KHI CHƯA KHÁM ĐỂ ĐIỀN TAY
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct DocxExportQuery {
    pub token: Option<String>,
    pub mode: Option<String>, // "reception" (chỉ điền hành chính & tiền sử, giữ nguyên ..... các mục khám để ghi tay)
}

async fn lookup_catalog_name(pool: &SqlitePool, cat_type: &str, code: &str) -> String {
    let c = code.trim();
    if c.is_empty() {
        return String::new();
    }
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT name FROM catalogs WHERE cat_type = ? AND (code = ? OR code = ?) LIMIT 1",
    )
    .bind(cat_type)
    .bind(c)
    .bind(c.trim_start_matches('0'))
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.map(|r| r.0).unwrap_or_else(|| c.to_string())
}

pub async fn export_record_docx_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(record_id): Path<i64>,
    Query(query): Query<DocxExportQuery>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, query.token.as_deref()).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let row: Option<KskRecordRow> = sqlx::query_as("SELECT * FROM ksk_records WHERE id = ?")
        .bind(record_id)
        .fetch_optional(&*db)
        .await
        .unwrap_or(None);

    let Some(record) = row else {
        return api_error(StatusCode::NOT_FOUND, "Không tìm thấy hồ sơ KSK");
    };

    let cfg = state.config.read().await.clone();
    let payload: Map<String, Value> =
        serde_json::from_str(&record.payload_json).unwrap_or_default();
    let get = |k: &str| -> String {
        payload
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };

    let is_reception_print = query.mode.as_deref() == Some("reception");

    let dan_toc_name = lookup_catalog_name(&*db, "DAN_TOC", &get("MA_DAN_TOC")).await;
    let doi_tuong_name = lookup_catalog_name(&*db, "DOI_TUONG", &get("DOI_TUONG")).await;
    let nguon_chi_tra_name =
        lookup_catalog_name(&*db, "NGUON_CHI_TRA", &get("NGUON_CHI_TRA")).await;
    let tinh_name = lookup_catalog_name(&*db, "TINH_TP", &get("MATINH_CU_TRU")).await;
    let xa_name = lookup_catalog_name(&*db, "XA_PHUONG", &get("MAXA_CU_TRU")).await;
    let nghe_nghiep_name =
        lookup_catalog_name(&*db, "NGHE_NGHIEP", &get("MA_NGHE_NGHIEP")).await;

    let ngay_sinh = get("NGAY_SINH");
    let tuoi_str = ngay_sinh
        .split('/')
        .nth(2)
        .and_then(|y| y.parse::<i32>().ok())
        .map(|birth_year| {
            let cur_year = Local::now().year();
            if cur_year >= birth_year {
                (cur_year - birth_year).to_string()
            } else {
                String::new()
            }
        })
        .unwrap_or_default();

    let mut replacements: HashMap<usize, String> = HashMap::new();
    let so_phieu = {
        let sp = get("SO_PHIEU");
        if sp.is_empty() {
            format!("{:04}", record.id)
        } else {
            sp
        }
    };

    replacements.insert(
        3,
        format!("Số: {}/GKSK-{}/26", so_phieu, cfg.ma_cskcb),
    );
    replacements.insert(
        6,
        format!("1. Họ và tên (viết chữ in hoa): {}", get("HO_TEN")),
    );
    let gt = get("GIOI_TINH");
    replacements.insert(
        7,
        format!(
            "2. Giới tính: {} Nam   {} Nữ",
            if gt == "1" { "☑" } else { "☐" },
            if gt == "2" { "☑" } else { "☐" }
        ),
    );
    replacements.insert(
        8,
        format!(
            "3. Ngày tháng năm sinh: {}   (Tuổi: {})",
            if ngay_sinh.is_empty() { "……/……/…………".to_string() } else { ngay_sinh },
            if tuoi_str.is_empty() { "……".to_string() } else { tuoi_str }
        ),
    );
    replacements.insert(
        9,
        format!("4. Số CCCD/Hộ chiếu/Mã định danh: {}", get("SO_CCCD")),
    );
    let ngaycap_cccd = get("NGAYCAP_CCCD");
    let noicap_cccd = get("NOICAP_CCCD");
    replacements.insert(
        10,
        format!(
            "5. Cấp ngày: {}   Nơi cấp: {}",
            if ngaycap_cccd.is_empty() { "……/……/…………" } else { &ngaycap_cccd },
            if noicap_cccd.is_empty() { "…………………………………………" } else { &noicap_cccd }
        ),
    );
    let nhom_mau = get("NHOM_MAU");
    replacements.insert(
        11,
        format!("6. Dân tộc: {}     7. Đối tượng: {}", dan_toc_name, doi_tuong_name),
    );
    replacements.insert(
        12,
        format!(
            "8. Nguồn chi trả: {}     9. Nhóm máu: {} (nếu có)",
            nguon_chi_tra_name,
            if nhom_mau.is_empty() { "……" } else { &nhom_mau }
        ),
    );
    replacements.insert(
        13,
        format!(
            "10. Nơi ở hiện tại: Tỉnh/thành: {}; Phường/xã: {}; Số nhà/thôn/xóm: {}; ĐT: {}",
            tinh_name,
            xa_name,
            get("DIA_CHI"),
            get("DIEN_THOAI")
        ),
    );
    replacements.insert(14, format!("11. Nghề nghiệp: {}", nghe_nghiep_name));
    replacements.insert(
        15,
        format!("12. Nơi làm việc, học tập: {}", get("NOI_LAM_VIEC_HOC_TAP")),
    );
    replacements.insert(16, format!("13. Lý do khám sức khỏe: {}", get("LY_DO_VV")));

    // Tiền sử gia đình
    let tsgd_co = get("TSGD_MAC_BENH") == "1";
    replacements.insert(
        20,
        format!(
            "a) {} Không       b) {} Có; Nếu “có”, đề nghị ghi cụ thể tên bệnh: {}",
            if !tsgd_co { "☑" } else { "☐" },
            if tsgd_co { "☑" } else { "☐" },
            if tsgd_co { get("TSGD_MA_BENH") } else { "…………………………………………………………".to_string() }
        ),
    );

    // Tiền sử bản thân (22 mục)
    let tsbt_pairs: &[(usize, usize, &str)] = &[
        (33, 34, "TSBT_BENH_TRONG_5_NAM_QUA"),
        (37, 38, "TSBT_DAI_THAO_DUONG"),
        (41, 42, "TSBT_BENH_THAN_KINH"),
        (45, 46, "TSBT_BENH_TAM_THAN"),
        (49, 50, "TSBT_BENH_MAT"),
        (53, 54, "TSBT_MAT_Y_THUC"),
        (57, 58, "TSBT_BENH_TAI"),
        (61, 62, "TSBT_NGAT"),
        (65, 66, "TSBT_BENH_TIM"),
        (69, 70, "TSBT_BENH_TIEU_HOA"),
        (73, 74, "TSBT_PHAU_THUAT_TIM"),
        (77, 78, "TSBT_ROI_LOAN_GIAC_NGU"),
        (81, 82, "TSBT_TANG_HUYET_AP"),
        (85, 86, "TSBT_TAI_BIEN"),
        (89, 90, "TSBT_KHO_THO"),
        (93, 94, "TSBT_BENH_COT_SONG"),
        (97, 98, "TSBT_BENH_PHOI"),
        (101, 102, "TSBT_RUOU_THUONG_XUYEN"),
        (105, 106, "TSBT_BENH_THAN"),
        (109, 110, "TSBT_MA_TUY"),
        (113, 114, "TSBT_NGHIEN_RUOU"),
        (117, 118, "TSBT_BENH_KHAC"),
    ];
    for (p_yes, p_no, field_name) in tsbt_pairs {
        let is_yes = get(field_name) == "1";
        replacements.insert(*p_yes, if is_yes { "☑" } else { "☐" }.to_string());
        replacements.insert(*p_no, if !is_yes { "☑" } else { "☐" }.to_string());
    }
    if get("TSBT_BENH_KHAC") == "1" && !get("TSBT_MA_BENH_KHAC").is_empty() {
        replacements.insert(
            116,
            format!("Bệnh khác (ghi rõ): {}", get("TSBT_MA_BENH_KHAC")),
        );
    }

    // Câu hỏi khác
    let dt_co = get("TSBT_DANG_DIEU_TRI_BENH") == "1";
    replacements.insert(121, format!("{} Không", if !dt_co { "☑" } else { "☐" }));
    replacements.insert(
        122,
        format!(
            "{} Có (Nếu có, xin hãy liệt kê tên bệnh, các thuốc đang dùng và liều lượng): {}",
            if dt_co { "☑" } else { "☐" },
            if dt_co {
                format!("{} {}", get("TSBT_MA_BENH"), get("TSBT_TEN_THUOC_LIEU_LUONG"))
            } else {
                "………………………………………………………………………………………………………………………………".to_string()
            }
        ),
    );

    let ts_co = gt == "2" && get("TSBT_THAI_SAN") == "1";
    replacements.insert(124, format!("{} Không", if !ts_co { "☑" } else { "☐" }));
    replacements.insert(
        125,
        format!(
            "{} Có (Nếu có, xin hãy liệt kê tên bệnh, các thuốc đang dùng và liều lượng): {}",
            if ts_co { "☑" } else { "☐" },
            if ts_co {
                format!("{} {}", get("TSBT_MA_BENH_THAI_SAN"), get("TSBT_TEN_THUOC_THAI_SAN"))
            } else {
                "………………………………………………………………………………………………………………………………".to_string()
            }
        ),
    );

    let nv = get("NGAY_VAO");
    let parts: Vec<&str> = nv.split('/').collect();
    let (d_str, m_str, y_str) = if parts.len() == 3 {
        (parts[0], parts[1], parts[2])
    } else {
        ("……", "……", "2026")
    };
    replacements.insert(
        128,
        format!("{}, ngày {} tháng {} năm {}", cfg.dia_danh, d_str, m_str, y_str),
    );

    // NẾU LÀ IN PHIẾU TIẾP NHẬN (KHÁCH ĐI KHÁM) -> GIỮ NGUYÊN DẤU "....." CỦA CÁC PHẦN KHÁM
    if !is_reception_print {
        replacements.insert(
            312,
            format!("{}, ngày {} tháng {} năm {}", cfg.dia_danh, d_str, m_str, y_str),
        );

        let chieu_cao = get("CHIEU_CAO");
        let can_nang = get("CAN_NANG");
        let bmi = get("CHI_SO_BMI");
        if !chieu_cao.is_empty() || !can_nang.is_empty() {
            replacements.insert(
                137,
                format!(
                    "- Chiều cao: {} cm; - Cân nặng: {} kg; - Chỉ số BMI: {}",
                    if chieu_cao.is_empty() { "…………" } else { &chieu_cao },
                    if can_nang.is_empty() { "…………" } else { &can_nang },
                    if bmi.is_empty() { "…………" } else { &bmi }
                ),
            );
        }
        let mach = get("MACH");
        let huyet_ap = get("HUYET_AP");
        if !mach.is_empty() || !huyet_ap.is_empty() {
            replacements.insert(
                138,
                format!(
                    "- Mạch: {} lần/phút; - Huyết áp: {} mmHg",
                    if mach.is_empty() { "…………" } else { &mach },
                    if huyet_ap.is_empty() { "…………/…………" } else { &huyet_ap }
                ),
            );
        }
        if !get("KHAM_THE_LUC_PL").is_empty() {
            replacements.insert(139, format!("- Phân loại thể lực: Loại {}", get("KHAM_THE_LUC_PL")));
        }

        let fmt_pl = |val: String| -> String {
            if val.is_empty() { "Phân loại: ……".to_string() } else { format!("Phân loại: Loại {}", val) }
        };

        if !get("NOI_KHOA_TUAN_HOAN").is_empty() { replacements.insert(153, get("NOI_KHOA_TUAN_HOAN")); }
        if !get("NOI_KHOA_TUAN_HOAN_PL").is_empty() { replacements.insert(158, fmt_pl(get("NOI_KHOA_TUAN_HOAN_PL"))); }
        if !get("NOI_KHOA_HO_HAP").is_empty() { replacements.insert(162, get("NOI_KHOA_HO_HAP")); }
        if !get("NOI_KHOA_HO_HAP_PL").is_empty() { replacements.insert(166, fmt_pl(get("NOI_KHOA_HO_HAP_PL"))); }
        if !get("NOI_KHOA_TIEU_HOA").is_empty() { replacements.insert(170, get("NOI_KHOA_TIEU_HOA")); }
        if !get("NOI_KHOA_TIEU_HOA_PL").is_empty() { replacements.insert(174, fmt_pl(get("NOI_KHOA_TIEU_HOA_PL"))); }
        if !get("NOI_KHOA_THAN_TN_SD").is_empty() { replacements.insert(179, get("NOI_KHOA_THAN_TN_SD")); }
        if !get("NOI_KHOA_THAN_TIETNIEU_PL").is_empty() { replacements.insert(183, fmt_pl(get("NOI_KHOA_THAN_TIETNIEU_PL"))); }
        if !get("NOI_KHOA_NOI_TIET").is_empty() { replacements.insert(187, get("NOI_KHOA_NOI_TIET")); }
        if !get("NOI_KHOA_NOI_TIET_PL").is_empty() { replacements.insert(191, fmt_pl(get("NOI_KHOA_NOI_TIET_PL"))); }
        if !get("NOI_KHOA_CO_XUONG_KHOP").is_empty() { replacements.insert(195, get("NOI_KHOA_CO_XUONG_KHOP")); }
        if !get("NOI_KHOA_CO_XUONG_KHOP_PL").is_empty() { replacements.insert(199, fmt_pl(get("NOI_KHOA_CO_XUONG_KHOP_PL"))); }
        if !get("NOI_KHOA_THAN_KINH").is_empty() { replacements.insert(203, get("NOI_KHOA_THAN_KINH")); }
        if !get("NOI_KHOA_THAN_KINH_PL").is_empty() { replacements.insert(207, fmt_pl(get("NOI_KHOA_THAN_KINH_PL"))); }
        if !get("NOI_KHOA_TAM_THAN").is_empty() { replacements.insert(211, get("NOI_KHOA_TAM_THAN")); }
        if !get("NOI_KHOA_TAM_THAN_PL").is_empty() { replacements.insert(215, fmt_pl(get("NOI_KHOA_TAM_THAN_PL"))); }
        if !get("BS_NOI").is_empty() { replacements.insert(154, get("BS_NOI")); }

        if !get("KET_QUA_KHAM_NGOAI_KHOA").is_empty() { replacements.insert(218, format!("Ngoại khoa: {}", get("KET_QUA_KHAM_NGOAI_KHOA"))); }
        if !get("KHAM_NGOAI_KHOA_PL").is_empty() { replacements.insert(219, fmt_pl(get("KHAM_NGOAI_KHOA_PL"))); }
        if !get("BS_NGOAI").is_empty() { replacements.insert(220, get("BS_NGOAI")); }

        if !get("KET_QUA_KHAM_DA_LIEU").is_empty() { replacements.insert(222, format!("Da liễu: {}", get("KET_QUA_KHAM_DA_LIEU"))); }
        if !get("KHAM_DA_LIEU_PL").is_empty() { replacements.insert(223, fmt_pl(get("KHAM_DA_LIEU_PL"))); }
        if !get("BS_DA_LIEU").is_empty() { replacements.insert(224, get("BS_DA_LIEU")); }

        if gt == "2" {
            if !get("KET_QUA_KHAM_SAN_PHU_KHOA").is_empty() { replacements.insert(226, format!("Sản phụ khoa: {}", get("KET_QUA_KHAM_SAN_PHU_KHOA"))); }
            if !get("KHAM_SAN_PHU_KHOA_PL").is_empty() { replacements.insert(227, fmt_pl(get("KHAM_SAN_PHU_KHOA_PL"))); }
            if !get("BS_SAN").is_empty() { replacements.insert(228, get("BS_SAN")); }
        }

        let kk_mp = get("KHONG_KINH_MAT_PHAI");
        let kk_mt = get("KHONG_KINH_MAT_TRAI");
        if !kk_mp.is_empty() || !kk_mt.is_empty() {
            replacements.insert(
                231,
                format!(
                    "Kết quả khám thị lực:  Không kính: Mắt phải: {}   Mắt trái: {}",
                    if kk_mp.is_empty() { "……" } else { &kk_mp },
                    if kk_mt.is_empty() { "……" } else { &kk_mt }
                ),
            );
        }
        let ck_mp = get("CO_KINH_MAT_PHAI");
        let ck_mt = get("CO_KINH_MAT_TRAI");
        if !ck_mp.is_empty() || !ck_mt.is_empty() {
            replacements.insert(
                232,
                format!(
                    "Có kính: Mắt phải: {}   Mắt trái: {}",
                    if ck_mp.is_empty() { "……" } else { &ck_mp },
                    if ck_mt.is_empty() { "……" } else { &ck_mt }
                ),
            );
        }
        if !get("BENH_KHAC_MAT").is_empty() { replacements.insert(238, format!("Các bệnh về mắt (nếu có): {}", get("BENH_KHAC_MAT"))); }
        if !get("KHAM_MAT_PL").is_empty() { replacements.insert(240, fmt_pl(get("KHAM_MAT_PL"))); }
        if !get("BS_MAT").is_empty() { replacements.insert(237, get("BS_MAT")); }

        if !get("TAI_TRAI_NOI_THUONG").is_empty() || !get("TAI_TRAI_NOI_THAM").is_empty() {
            replacements.insert(245, format!("Tai trái: Nói thường: {} m; Nói thầm: {} m", get("TAI_TRAI_NOI_THUONG"), get("TAI_TRAI_NOI_THAM")));
        }
        if !get("TAI_PHAI_NOI_THUONG").is_empty() || !get("TAI_PHAI_NOI_THAM").is_empty() {
            replacements.insert(246, format!("Tai phải: Nói thường: {} m; Nói thầm: {} m", get("TAI_PHAI_NOI_THUONG"), get("TAI_PHAI_NOI_THAM")));
        }
        if !get("BENH_KHAC_TAI_MUI_HONG").is_empty() { replacements.insert(248, format!("Các bệnh về tai mũi họng (nếu có): {}", get("BENH_KHAC_TAI_MUI_HONG"))); }
        if !get("KHAM_TAI_MUI_HONG_PL").is_empty() { replacements.insert(250, fmt_pl(get("KHAM_TAI_MUI_HONG_PL"))); }
        if !get("BS_TMH").is_empty() { replacements.insert(247, get("BS_TMH")); }

        if !get("HAM_TREN").is_empty() { replacements.insert(255, format!("Hàm trên: {}", get("HAM_TREN"))); }
        if !get("HAM_DUOI").is_empty() { replacements.insert(256, format!("Hàm dưới: {}", get("HAM_DUOI"))); }
        if !get("BENH_KHAC_RANG_HAM_MAT").is_empty() { replacements.insert(258, format!("Các bệnh về răng hàm mặt (nếu có): {}", get("BENH_KHAC_RANG_HAM_MAT"))); }
        if !get("KHAM_RANG_HAM_MAT_PL").is_empty() { replacements.insert(260, fmt_pl(get("KHAM_RANG_HAM_MAT_PL"))); }
        if !get("BS_RHM").is_empty() { replacements.insert(257, get("BS_RHM")); }

        if !get("CLS_HUYET_HOC").is_empty() { replacements.insert(268, format!("- Huyết học: {}", get("CLS_HUYET_HOC"))); }
        if !get("CLS_DUONG_MAU").is_empty() { replacements.insert(269, format!("- Sinh hóa máu: Đường máu: {}", get("CLS_DUONG_MAU"))); }
        if !get("CLS_URE").is_empty() || !get("CLS_CREATININ").is_empty() {
            replacements.insert(270, format!("Urê: {}     Creatinin: {}", get("CLS_URE"), get("CLS_CREATININ")));
        }
        if !get("CLS_ASAT_GOT").is_empty() || !get("CLS_ALAT_GPT").is_empty() {
            replacements.insert(271, format!("ASAT(GOT): {}     ALAT(GPT): {}", get("CLS_ASAT_GOT"), get("CLS_ALAT_GPT")));
        }
        if !get("CLS_NUOC_TIEU").is_empty() { replacements.insert(275, format!("- Kết quả: {}", get("CLS_NUOC_TIEU"))); }
        if !get("CLS_XQ_TIM_PHOI").is_empty() { replacements.insert(279, get("CLS_XQ_TIM_PHOI")); }
        if !get("BS_CLS").is_empty() { replacements.insert(272, get("BS_CLS")); }

        if !get("KET_LUAN_BENH").is_empty() || !get("CAC_BENH_TAT_NEU_CO").is_empty() {
            replacements.insert(
                305,
                format!(
                    "1. Tình trạng sức khỏe; mắc các bệnh, tật (nếu có): {} (ICD-10: {})",
                    get("KET_LUAN_BENH"), get("CAC_BENH_TAT_NEU_CO")
                ),
            );
        }
        if !get("PHAN_LOAI_SK").is_empty() {
            replacements.insert(309, format!("2.   Phân loại sức khỏe (theo quy định của Bộ Y tế): Loại {}", get("PHAN_LOAI_SK")));
        }
        if !get("BS_KET_LUAN").is_empty() {
            replacements.insert(315, get("BS_KET_LUAN"));
        }
    }

    let template_bytes =
        fs::read(&cfg.template_docx_path).unwrap_or_else(|_| EMBEDDED_TEMPLATE_DOCX.to_vec());

    match build_filled_docx(&template_bytes, &replacements) {
        Ok(docx_bytes) => {
            let safe_ma_tn = record
                .ma_tiep_nhan
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' { c } else { '_' })
                .collect::<String>();
            let prefix = if is_reception_print { "Phieu_TiepNhan" } else { "Phieu_KSK" };
            let filename = format!("{}_{}.docx", prefix, safe_ma_tn);
            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                            .to_string(),
                    ),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                docx_bytes,
            )
                .into_response()
        }
        Err(e) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi tạo file Word từ template18.docx: {}", e),
        ),
    }
}

fn build_filled_docx(
    template_bytes: &[u8],
    replacements: &HashMap<usize, String>,
) -> Result<Vec<u8>, String> {
    let reader = Cursor::new(template_bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| e.to_string())?;

    let mut out_buf = Cursor::new(Vec::new());
    {
        let mut zip_writer = zip::ZipWriter::new(&mut out_buf);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().to_string();
            let mut content = Vec::new();
            file.read_to_end(&mut content).map_err(|e| e.to_string())?;

            zip_writer
                .start_file(&name, options)
                .map_err(|e| e.to_string())?;

            if name == "word/document.xml" {
                let xml_str = String::from_utf8(content).map_err(|e| e.to_string())?;
                let modified_xml = replace_docx_paragraphs(&xml_str, replacements)?;
                zip_writer
                    .write_all(modified_xml.as_bytes())
                    .map_err(|e| e.to_string())?;
            } else {
                zip_writer.write_all(&content).map_err(|e| e.to_string())?;
            }
        }
        zip_writer.finish().map_err(|e| e.to_string())?;
    }

    Ok(out_buf.into_inner())
}

fn replace_docx_paragraphs(
    xml_str: &str,
    replacements: &HashMap<usize, String>,
) -> Result<String, String> {
    let mut reader = Reader::from_str(xml_str);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    let mut current_p_idx: Option<usize> = None;
    let mut p_counter: usize = 0;
    let mut inside_wt = false;
    let mut wt_index_in_p: usize = 0;
    let mut has_replaced_in_current_p = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                if name_bytes == b"w:p" || name_bytes.ends_with(b":p") {
                    current_p_idx = Some(p_counter);
                    p_counter += 1;
                    wt_index_in_p = 0;
                    has_replaced_in_current_p = false;
                } else if name_bytes == b"w:t" || name_bytes.ends_with(b":t") {
                    inside_wt = true;
                }
                writer
                    .write_event(Event::Start(e))
                    .map_err(|err| err.to_string())?;
            }
            Ok(Event::End(e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                if name_bytes == b"w:p" || name_bytes.ends_with(b":p") {
                    if let Some(p_idx) = current_p_idx {
                        if !has_replaced_in_current_p {
                            if let Some(new_val) = replacements.get(&p_idx) {
                                let run_xml = format!(
                                    r#"<w:r><w:t xml:space="preserve">{}</w:t></w:r>"#,
                                    quick_xml::escape::escape(new_val)
                                );
                                writer
                                    .get_mut()
                                    .write_all(run_xml.as_bytes())
                                    .map_err(|err| err.to_string())?;
                            }
                        }
                    }
                    current_p_idx = None;
                } else if name_bytes == b"w:t" || name_bytes.ends_with(b":t") {
                    inside_wt = false;
                    wt_index_in_p += 1;
                }
                writer
                    .write_event(Event::End(e))
                    .map_err(|err| err.to_string())?;
            }
            Ok(Event::Text(e)) => {
                if inside_wt {
                    if let Some(p_idx) = current_p_idx {
                        if let Some(new_val) = replacements.get(&p_idx) {
                            if wt_index_in_p == 0 {
                                writer
                                    .write_event(Event::Text(BytesText::new(new_val)))
                                    .map_err(|err| err.to_string())?;
                                has_replaced_in_current_p = true;
                            }
                            continue;
                        }
                    }
                }
                writer
                    .write_event(Event::Text(e))
                    .map_err(|err| err.to_string())?;
            }
            Ok(Event::Eof) => break,
            Ok(other) => {
                writer
                    .write_event(other)
                    .map_err(|err| err.to_string())?;
            }
            Err(e) => return Err(format!("XML parse error: {}", e)),
        }
    }

    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| e.to_string())
}

// ============================================================================
// 5. TRÍCH XUẤT CSV CHUẨN QĐ 3176 & QĐ 1804 (123 CHỈ TIÊU)
// ============================================================================

fn extract_map_str(map: &Map<String, Value>, key: &str) -> String {
    map.get(key).map(json_val_to_string).unwrap_or_default()
}

fn build_cls_summary(map: &Map<String, Value>) -> (String, String, String, String, String, String) {
    let cls_items: [(&str, &str, &str, &str); 10] = [
        ("CLS_HH", "Huyết học", "CLS_HUYET_HOC", ""),
        ("CLS_GLU", "Đường máu", "CLS_DUONG_MAU", "mmol/L"),
        ("CLS_URE", "Urê máu", "CLS_URE", "mmol/L"),
        ("CLS_CRE", "Creatinin", "CLS_CREATININ", "µmol/L"),
        ("CLS_GOT", "ASAT (GOT)", "CLS_ASAT_GOT", "U/L"),
        ("CLS_GPT", "ALAT (GPT)", "CLS_ALAT_GPT", "U/L"),
        ("CLS_NT", "Nước tiểu", "CLS_NUOC_TIEU", ""),
        ("CLS_XQ", "X-quang tim phổi", "CLS_XQ_TIM_PHOI", ""),
        ("CLS_KHAC", "CLS khác", "CLS_KHAC_MO_TA", ""),
        ("CLS_DK", "CLS định kỳ", "CLS_DINH_KY_KET_QUA", ""),
    ];

    let mut codes = Vec::new();
    let mut names = Vec::new();
    let mut vals = Vec::new();
    let mut units = Vec::new();
    let mut descs = Vec::new();
    let mut abnormal = false;

    for (code, label, field, unit) in cls_items {
        let mut val = extract_map_str(map, field);
        if val.is_empty() && field == "CLS_DINH_KY_KET_QUA" {
            val = extract_map_str(map, "CLS_DINH_KY_KQ");
        }
        if !val.is_empty() {
            codes.push(code);
            names.push(label);
            vals.push(val.clone());
            if !unit.is_empty() {
                units.push(unit);
            }
            descs.push(format!("{}: {}", label, val));
            let lower = val.to_lowercase();
            if lower.contains("bất thường") || lower.contains("tăng") || lower.contains("giảm") || lower.contains("viêm") || lower.contains("dương") {
                abnormal = true;
            }
        }
    }

    if codes.is_empty() {
        return (
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        );
    }

    let conclusion = if abnormal {
        "Bất thường".to_string()
    } else {
        "Bình thường".to_string()
    };

    (
        codes.join("; "),
        names.join("; "),
        vals.join("; "),
        units.join("; "),
        descs.join("; "),
        conclusion,
    )
}

pub async fn export_csv_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<CsvExportFilterQuery>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, query.token.as_deref()).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    let cfg = state.config.read().await.clone();

    let rows = query_filtered_records(
        &*db,
        query.from_date.as_deref(),
        query.to_date.as_deref(),
        query.status,
        query.q.as_deref(),
        query.workplace.as_deref(),
        query.classification.as_deref(),
        query.selected_ids.as_deref(),
        None,
        true,
    )
    .await;

    let mut output: Vec<u8> = vec![0xEF, 0xBB, 0xBF];
    {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(true)
            .from_writer(&mut output);

        if let Err(e) = wtr.write_record(CSV_EXPORT_COLUMNS) {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lỗi ghi header CSV: {}", e),
            );
        }

        for row in rows {
            let map: Map<String, Value> =
                serde_json::from_str(&row.payload_json).unwrap_or_default();
            let cls_summary = build_cls_summary(&map);

            let record_values: Vec<String> = CSV_EXPORT_COLUMNS
                .iter()
                .map(|&col| match col {
                    "HO_TEN" => {
                        let v = extract_map_str(&map, "HO_TEN");
                        let raw = if v.is_empty() { row.ho_ten.clone() } else { v };
                        raw.to_uppercase()
                    }
                    "GIOI_TINH" => {
                        let v = extract_map_str(&map, "GIOI_TINH");
                        let raw = if v.is_empty() { row.gioi_tinh.clone() } else { v };
                        if raw.is_empty() { "1".to_string() } else { raw }
                    }
                    "NGAY_SINH" => {
                        let v = extract_map_str(&map, "NGAY_SINH");
                        if v.is_empty() { row.ngay_sinh.clone() } else { v }
                    }
                    "MA_DAN_TOC" => {
                        let v = extract_map_str(&map, "MA_DAN_TOC");
                        if v.is_empty() { "01".to_string() } else { v }
                    }
                    "SO_CCCD" => {
                        let v = extract_map_str(&map, "SO_CCCD");
                        if v.is_empty() { row.so_cccd.clone() } else { v }
                    }
                    "MA_NGHE_NGHIEP" => {
                        let v = extract_map_str(&map, "MA_NGHE_NGHIEP");
                        if v.is_empty() { "00".to_string() } else { v }
                    }
                    "LY_DO_VV" => {
                        let v = extract_map_str(&map, "LY_DO_VV");
                        if v.is_empty() { "Khám sức khỏe".to_string() } else { v }
                    }
                    "MA_LK" => {
                        let v = extract_map_str(&map, "MA_LK");
                        if v.is_empty() { row.ma_tiep_nhan.clone() } else { v }
                    }
                    "MA_CSKCB" => {
                        let v = extract_map_str(&map, "MA_CSKCB");
                        if v.is_empty() { cfg.ma_cskcb.clone() } else { v }
                    }
                    "MA_GTIN_CSKCB" => {
                        let v = extract_map_str(&map, "MA_GTIN_CSKCB");
                        if v.is_empty() { cfg.ma_gtin_cskcb.clone() } else { v }
                    }
                    "DOI_TUONG" => {
                        let v = extract_map_str(&map, "DOI_TUONG");
                        if v.is_empty() { "13".to_string() } else { v }
                    }
                    "NGUON_CHI_TRA" => {
                        let v = extract_map_str(&map, "NGUON_CHI_TRA");
                        if v.is_empty() { "9".to_string() } else { v }
                    }
                    "MA_LOAI_KCB" => {
                        let v = extract_map_str(&map, "MA_LOAI_KCB");
                        if v.is_empty() { "15".to_string() } else { v }
                    }
                    "NGAY_VAO" => {
                        let v = extract_map_str(&map, "NGAY_VAO");
                        if v.is_empty() { row.ngay_vao.clone() } else { v }
                    }
                    "NOI_KHOA_THAN_TN_SD_PL" => {
                        let v = extract_map_str(&map, "NOI_KHOA_THAN_TN_SD_PL");
                        if v.is_empty() {
                            extract_map_str(&map, "NOI_KHOA_THAN_TIETNIEU_PL")
                        } else {
                            v
                        }
                    }
                    "CKDT_NOI_KHOA_TUAN_HOAN"
                    | "CKDT_NOI_KHOA_HO_HAP"
                    | "CKDT_NOI_KHOA_TIEU_HOA"
                    | "CKDT_NOI_KHOA_THAN_TN_SD"
                    | "CKDT_NOI_KHOA_NOI_TIET"
                    | "CKDT_NOI_KHOA_CO_XUONG_KHOP"
                    | "CKDT_NOI_KHOA_THAN_KINH"
                    | "CKDT_NOI_KHOA_TAM_THAN" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_NOI") } else { v }
                    }
                    "CKDT_KHAM_NGOAI_KHOA" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_NGOAI") } else { v }
                    }
                    "CKDT_KHAM_DA_LIEU" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_DA_LIEU") } else { v }
                    }
                    "CKDT_KHAM_SAN_PHU_KHOA" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_SAN") } else { v }
                    }
                    "CKDT_KHAM_MAT" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_MAT") } else { v }
                    }
                    "CKDT_KHAM_TAI_MUI_HONG" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_TMH") } else { v }
                    }
                    "CKDT_KHAM_RANG_HAM_MAT" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_RHM") } else { v }
                    }
                    "MA_DICH_VU" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cls_summary.0.clone() } else { v }
                    }
                    "MA_CHI_SO" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cls_summary.1.clone() } else { v }
                    }
                    "GIA_TRI" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cls_summary.2.clone() } else { v }
                    }
                    "DON_VI_DO" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cls_summary.3.clone() } else { v }
                    }
                    "MO_TA" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cls_summary.4.clone() } else { v }
                    }
                    "KET_LUAN" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cls_summary.5.clone() } else { v }
                    }
                    "CKDT_KET_QUA_CLS" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_CLS") } else { v }
                    }
                    "CKS_NGUOI_KET_LUAN" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { extract_map_str(&map, "BS_KET_LUAN") } else { v }
                    }
                    "CKS_BENH_VIEN" => {
                        let v = extract_map_str(&map, col);
                        if v.is_empty() { cfg.ten_cskcb.clone() } else { v }
                    }
                    _ => extract_map_str(&map, col),
                })
                .collect();

            if let Err(e) = wtr.write_record(&record_values) {
                return api_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Lỗi ghi dòng CSV: {}", e),
                );
            }
        }

        let _ = wtr.flush();
    }

    let filename = format!("KSK_QD3176_{}.csv", Local::now().format("%Y%m%d_%H%M%S"));
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        output,
    )
        .into_response()
}

// ============================================================================
// 6. QUẢN LÝ DANH MỤC
// ============================================================================

pub async fn get_catalog_versions_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    // Đảm bảo các cat_type có sẵn trong catalog_meta nếu chưa có
    let rows: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT c.cat_type, COALESCE(m.updated_at, '2026-01-01 00:00:00') as updated_at
        FROM (SELECT DISTINCT cat_type FROM catalogs) c
        LEFT JOIN catalog_meta m ON c.cat_type = m.cat_type
        "#
    )
    .fetch_all(&*db)
    .await
    .unwrap_or_default();

    let mut map = serde_json::Map::new();
    for (cat_type, updated_at) in rows {
        map.insert(cat_type, Value::String(updated_at));
    }

    Json(Value::Object(map)).into_response()
}

pub async fn list_catalogs_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Query(query): Query<CatalogQuery>,
) -> Response {
    let db = state.db.read().await;
    let _user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };

    let cat_type = query.cat_type.unwrap_or_default().trim().to_string();
    let parent_code = query.parent_code.unwrap_or_default().trim().to_string();
    let q = query.q.unwrap_or_default().trim().to_string();
    let limit = match query.limit {
        Some(l) if l <= 0 => 100_000,
        Some(l) => l.clamp(1, 100_000),
        None => 1000,
    };

    let items: Vec<CatalogItem> = if !cat_type.is_empty() && !parent_code.is_empty() {
        sqlx::query_as(
            "SELECT id, cat_type, code, name, parent_code, is_active, is_default FROM catalogs WHERE cat_type = ? AND parent_code = ? ORDER BY code ASC LIMIT ?",
        )
        .bind(&cat_type)
        .bind(&parent_code)
        .bind(limit)
        .fetch_all(&*db)
        .await
        .unwrap_or_default()
    } else if !cat_type.is_empty() && !q.is_empty() {
        let like_q = format!("%{}%", q);
        sqlx::query_as(
            "SELECT id, cat_type, code, name, parent_code, is_active, is_default FROM catalogs WHERE cat_type = ? AND (code LIKE ? OR name LIKE ?) ORDER BY code ASC LIMIT ?",
        )
        .bind(&cat_type)
        .bind(&like_q)
        .bind(&like_q)
        .bind(limit)
        .fetch_all(&*db)
        .await
        .unwrap_or_default()
    } else if !cat_type.is_empty() {
        sqlx::query_as(
            "SELECT id, cat_type, code, name, parent_code, is_active, is_default FROM catalogs WHERE cat_type = ? ORDER BY code ASC LIMIT ?",
        )
        .bind(&cat_type)
        .bind(limit)
        .fetch_all(&*db)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as(
            "SELECT id, cat_type, code, name, parent_code, is_active, is_default FROM catalogs WHERE cat_type != 'ICD10' AND cat_type != 'TIEU_CHUAN_PL' ORDER BY cat_type ASC, code ASC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&*db)
        .await
        .unwrap_or_default()
    };

    Json(items).into_response()
}

pub async fn bulk_import_catalogs_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<BulkCatalogImportRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(
            StatusCode::FORBIDDEN,
            "Chỉ Quản trị viên (admin) mới có quyền Import danh mục",
        );
    }

    let cat_type = req.cat_type.trim().to_uppercase();
    if cat_type.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Loại danh mục (cat_type) không được để trống");
    }

    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lỗi khởi tạo transaction: {}", e),
            )
        }
    };

    if req.replace_all {
        let _ = sqlx::query("DELETE FROM catalogs WHERE cat_type = ?")
            .bind(&cat_type)
            .execute(&mut *tx)
            .await;
    }

    let mut imported_count = 0usize;
    for raw_line in req.raw_text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = if line.contains('|') {
            line.split('|').collect()
        } else if line.contains('\t') {
            line.split('\t').collect()
        } else {
            line.split(';').collect()
        };

        if parts.len() >= 2 {
            let code = parts[0].trim();
            let name = parts[1].trim();
            let parent_code = if parts.len() >= 3 { parts[2].trim() } else { "" };

            if !code.is_empty() && !name.is_empty() {
                let res = sqlx::query(
                    r#"
                    INSERT INTO catalogs (cat_type, code, name, parent_code)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(cat_type, code, parent_code)
                    DO UPDATE SET name = excluded.name
                    "#,
                )
                .bind(&cat_type)
                .bind(code)
                .bind(name)
                .bind(parent_code)
                .execute(&mut *tx)
                .await;

                if res.is_ok() {
                    imported_count += 1;
                }
            }
        }
    }

    if let Err(e) = tx.commit().await {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi lưu danh mục vào DB: {}", e),
        );
    }

    let _ = sqlx::query(
        "INSERT INTO catalog_meta (cat_type, updated_at) VALUES (?, datetime('now', 'localtime')) ON CONFLICT(cat_type) DO UPDATE SET updated_at = datetime('now', 'localtime')"
    )
    .bind(&cat_type)
    .execute(&*db)
    .await;

    Json(json!({
        "message": format!("Đã import thành công {} dòng vào danh mục {}", imported_count, cat_type),
        "imported_count": imported_count
    }))
    .into_response()
}

pub async fn delete_catalog_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(cat_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền xóa danh mục");
    }

    let _ = sqlx::query("DELETE FROM catalogs WHERE id = ?")
        .bind(cat_id)
        .execute(&*db)
        .await;

    let _ = sqlx::query(
        "UPDATE catalog_meta SET updated_at = datetime('now', 'localtime') WHERE cat_type IN (SELECT cat_type FROM catalogs WHERE id = ?)"
    )
    .bind(cat_id)
    .execute(&*db)
    .await;

    Json(json!({ "message": "Đã xóa mục danh mục" })).into_response()
}

pub async fn bulk_delete_catalogs_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<BulkDeleteCatalogRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền xóa danh mục");
    }

    if req.ids.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Danh sách ID xóa không được để trống");
    }

    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lỗi transaction: {}", e),
            )
        }
    };

    let mut deleted_count = 0usize;
    for id in req.ids {
        let res = sqlx::query("DELETE FROM catalogs WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await;
        if let Ok(r) = res {
            deleted_count += r.rows_affected() as usize;
        }
    }

    if let Err(e) = tx.commit().await {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi xóa danh mục: {}", e),
        );
    }

    Json(json!({
        "message": format!("Đã xóa thành công {} mục danh mục", deleted_count),
        "deleted_count": deleted_count
    }))
    .into_response()
}

pub async fn update_catalog_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(cat_id): Path<i64>,
    Json(req): Json<UpdateCatalogRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền sửa danh mục");
    }

    let code = req.code.trim();
    let name = req.name.trim();
    let parent_code = req.parent_code.trim();

    if code.is_empty() || name.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Mã và Tên danh mục không được để trống");
    }

    let res = sqlx::query(
        "UPDATE catalogs SET code = ?, name = ?, parent_code = ? WHERE id = ?"
    )
    .bind(code)
    .bind(name)
    .bind(parent_code)
    .bind(cat_id)
    .execute(&*db)
    .await;

    let _ = sqlx::query(
        "UPDATE catalog_meta SET updated_at = datetime('now', 'localtime') WHERE cat_type IN (SELECT cat_type FROM catalogs WHERE id = ?)"
    )
    .bind(cat_id)
    .execute(&*db)
    .await;

    match res {
        Ok(r) => {
            if r.rows_affected() == 0 {
                api_error(StatusCode::NOT_FOUND, "Không tìm thấy mục danh mục để cập nhật")
            } else {
                Json(json!({ "message": "Đã cập nhật mục danh mục thành công" })).into_response()
            }
        }
        Err(e) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi cập nhật danh mục: {}", e),
        ),
    }
}

pub async fn bulk_status_catalogs_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<BulkStatusCatalogRequest>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền cập nhật trạng thái danh mục");
    }

    if req.ids.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Danh sách ID không được để trống");
    }

    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => {
            return api_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Lỗi transaction: {}", e),
            )
        }
    };

    let new_status = if req.is_active == 0 { 0i64 } else { 1i64 };
    let mut updated_count = 0usize;
    for id in &req.ids {
        let res = sqlx::query("UPDATE catalogs SET is_active = ? WHERE id = ?")
            .bind(new_status)
            .bind(id)
            .execute(&mut *tx)
            .await;
        if let Ok(r) = res {
            updated_count += r.rows_affected() as usize;
        }
    }

    // Cập nhật timestamp cho các cat_type bị ảnh hưởng
    for id in &req.ids {
        let _ = sqlx::query("UPDATE catalog_meta SET updated_at = datetime('now', 'localtime') WHERE cat_type IN (SELECT cat_type FROM catalogs WHERE id = ?)")
            .bind(id)
            .execute(&mut *tx)
            .await;
    }

    if let Err(e) = tx.commit().await {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi cập nhật trạng thái danh mục: {}", e),
        );
    }

    let action_str = if new_status == 0 { "tạm ngưng" } else { "kích hoạt" };
    Json(json!({
        "message": format!("Đã {} thành công {} mục danh mục", action_str, updated_count),
        "updated_count": updated_count
    }))
    .into_response()
}

pub async fn set_default_catalog_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(cat_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền thiết lập mặc định");
    }

    let item: Option<(String,)> = sqlx::query_as("SELECT cat_type FROM catalogs WHERE id = ?")
        .bind(cat_id)
        .fetch_optional(&*db)
        .await
        .unwrap_or(None);

    let cat_type = match item {
        Some((ct,)) => ct,
        None => return api_error(StatusCode::NOT_FOUND, "Không tìm thấy mục danh mục"),
    };

    let mut tx = match db.begin().await {
        Ok(t) => t,
        Err(e) => return api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi transaction: {}", e)),
    };

    // Reset default cho các mục cùng cat_type
    let _ = sqlx::query("UPDATE catalogs SET is_default = 0 WHERE cat_type = ?")
        .bind(&cat_type)
        .execute(&mut *tx)
        .await;

    // Đặt mục này làm mặc định và kích hoạt
    let res = sqlx::query("UPDATE catalogs SET is_default = 1, is_active = 1 WHERE id = ?")
        .bind(cat_id)
        .execute(&mut *tx)
        .await;

    // Cập nhật timestamp meta
    let _ = sqlx::query("UPDATE catalog_meta SET updated_at = datetime('now', 'localtime') WHERE cat_type = ?")
        .bind(&cat_type)
        .execute(&mut *tx)
        .await;

    if let Err(e) = tx.commit().await {
        return api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi lưu mặc định: {}", e));
    }

    match res {
        Ok(_) => Json(json!({ "message": "Đã đặt mục làm mặc định thành công" })).into_response(),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi: {}", e)),
    }
}

pub async fn toggle_active_catalog_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(cat_id): Path<i64>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới có quyền đổi trạng thái");
    }

    let res = sqlx::query(
        "UPDATE catalogs SET is_active = CASE WHEN is_active = 1 THEN 0 ELSE 1 END WHERE id = ?"
    )
    .bind(cat_id)
    .execute(&*db)
    .await;

    let _ = sqlx::query(
        "UPDATE catalog_meta SET updated_at = datetime('now', 'localtime') WHERE cat_type IN (SELECT cat_type FROM catalogs WHERE id = ?)"
    )
    .bind(cat_id)
    .execute(&*db)
    .await;

    match res {
        Ok(r) => {
            if r.rows_affected() == 0 {
                api_error(StatusCode::NOT_FOUND, "Không tìm thấy mục danh mục")
            } else {
                Json(json!({ "message": "Đã thay đổi trạng thái danh mục thành công" })).into_response()
            }
        }
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, format!("Lỗi: {}", e)),
    }
}

// ============================================================================
// 7. ADMIN SETTINGS & DATABASE DYNAMIC MANAGEMENT
// ============================================================================

pub async fn get_admin_settings_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới được vào Cấu hình Hệ thống");
    }

    let cfg = state.config.read().await.clone();
    let db_info = get_db_info(&*db, &cfg.database_path).await;

    Json(json!({
        "config": cfg,
        "db_info": db_info
    }))
    .into_response()
}

pub async fn update_admin_settings_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(new_cfg): Json<AppConfig>,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới được lưu Cấu hình Hệ thống");
    }

    if let Err(e) = new_cfg.save() {
        return api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Không thể lưu config.toml: {}", e),
        );
    }

    let mut cfg_lock = state.config.write().await;
    *cfg_lock = new_cfg.clone();

    Json(json!({
        "message": "Đã lưu Cấu hình Hệ thống thành công",
        "config": new_cfg
    }))
    .into_response()
}

pub async fn switch_database_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(req): Json<SwitchDbRequest>,
) -> Response {
    let current_db = state.db.read().await;
    let user = match authenticate(&*current_db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới được đổi cơ sở dữ liệu");
    }
    drop(current_db);

    let target_path = req.db_path.trim().to_string();
    if target_path.is_empty() {
        return api_error(StatusCode::BAD_REQUEST, "Đường dẫn file database không được để trống");
    }

    let p = std::path::Path::new(&target_path);
    if !req.create_if_missing && !p.exists() {
        return api_error(StatusCode::NOT_FOUND, format!("File database '{}' không tồn tại", target_path));
    }

    match init_db(&target_path).await {
        Ok(new_pool) => {
            {
                let mut db_lock = state.db.write().await;
                *db_lock = new_pool.clone();
            }
            {
                let mut cfg_lock = state.config.write().await;
                cfg_lock.database_path = target_path.clone();
                let _ = cfg_lock.save();
            }
            let info = get_db_info(&new_pool, &target_path).await;
            Json(json!({
                "message": format!("Đã kết nối và nạp thành công database: {}", target_path),
                "db_info": info
            }))
            .into_response()
        }
        Err(e) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Không thể khởi tạo hoặc nạp file database: {}", e),
        ),
    }
}

pub async fn reload_asset_catalogs_handler(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Response {
    let db = state.db.read().await;
    let user = match authenticate(&*db, &headers, None).await {
        Ok(u) => u,
        Err(resp) => return resp,
    };
    if !user.has_role("admin") {
        return api_error(StatusCode::FORBIDDEN, "Chỉ Quản trị viên mới được nạp lại danh mục asset");
    }

    match seed_catalogs_from_asset(&*db).await {
        Ok(count) => Json(json!({
            "message": format!("Đã nạp thành công {} mục danh mục chuẩn từ asset vào CSDL!", count),
            "count": count
        }))
        .into_response(),
        Err(e) => api_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Lỗi khi nạp danh mục từ asset: {}", e),
        ),
    }
}
