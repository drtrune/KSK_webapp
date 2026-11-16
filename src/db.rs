use crate::config::AppConfig;
use crate::models::{DbInfo, HisPushRequest};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::fs;
use std::str::FromStr;
use std::time::Duration;

const SEED_CATALOGS_TSV: &str = include_str!("../asset/seed_catalogs.tsv");

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        if !parent.as_os_str().is_empty() {
            let _ = fs::create_dir_all(parent);
        }
    }

    let conn_str = format!("sqlite:{}", db_path);
    // Dùng JournalMode::Delete + Synchronous::Full + busy_timeout(15s) để an toàn khi nhiều máy trạm
    // cùng truy cập file SQLite trên ổ đĩa mạng chung (LAN / SMB) mà không bị lỗi -shm locking.
    let options = SqliteConnectOptions::from_str(&conn_str)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Delete)
        .synchronous(SqliteSynchronous::Full)
        .busy_timeout(Duration::from_secs(15));

    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect_with(options)
        .await?;

    // 1. Tạo bảng users
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            full_name TEXT NOT NULL,
            role TEXT NOT NULL,
            is_locked INTEGER NOT NULL DEFAULT 0,
            wrapped_mk TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        "#,
    )
    .execute(&pool)
    .await?;
    let _ = sqlx::query("ALTER TABLE users ADD COLUMN wrapped_mk TEXT NOT NULL DEFAULT ''")
        .execute(&pool)
        .await;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL DEFAULT ''
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 2. Tạo bảng sessions
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            token TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
        );
        "#,
    )
    .execute(&pool)
    .await?;

    // 3. Tạo bảng ksk_records
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ksk_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ma_tiep_nhan TEXT NOT NULL UNIQUE,
            ma_y_te TEXT NOT NULL DEFAULT '',
            ho_ten TEXT NOT NULL DEFAULT '',
            gioi_tinh TEXT NOT NULL DEFAULT '1',
            ngay_sinh TEXT NOT NULL DEFAULT '',
            so_cccd TEXT NOT NULL DEFAULT '',
            ngay_vao TEXT NOT NULL DEFAULT '',
            status INTEGER NOT NULL DEFAULT 0,
            exam_mode TEXT NOT NULL DEFAULT 'le',
            contract_id INTEGER,
            exam_templates TEXT NOT NULL DEFAULT 'TREN_18',
            payload_json TEXT NOT NULL DEFAULT '{}',
            updated_by TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        "#,
    )
    .execute(&pool)
    .await?;
    let _ = sqlx::query("ALTER TABLE ksk_records ADD COLUMN exam_mode TEXT NOT NULL DEFAULT 'le'")
        .execute(&pool)
        .await;
    let _ = sqlx::query("ALTER TABLE ksk_records ADD COLUMN contract_id INTEGER")
        .execute(&pool)
        .await;
    let _ = sqlx::query(
        "ALTER TABLE ksk_records ADD COLUMN exam_templates TEXT NOT NULL DEFAULT 'TREN_18'",
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_ksk_records_mode_contract ON ksk_records(exam_mode, contract_id)",
    )
    .execute(&pool)
    .await;

    // 3b. Tạo bảng companies & exam_contracts (Khám đoàn)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS companies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            tax_code TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL DEFAULT '',
            contact_person TEXT NOT NULL DEFAULT '',
            contact_phone TEXT NOT NULL DEFAULT '',
            note TEXT NOT NULL DEFAULT '',
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS exam_contracts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            company_id INTEGER NOT NULL,
            contract_code TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            exam_date TEXT NOT NULL DEFAULT '',
            expected_count INTEGER NOT NULL DEFAULT 0,
            status INTEGER NOT NULL DEFAULT 0,
            scan_file_path TEXT NOT NULL DEFAULT '',
            default_templates TEXT NOT NULL DEFAULT 'TREN_18',
            note TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT
        );
        "#,
    )
    .execute(&pool)
    .await?;
    let _ = sqlx::query(
        "ALTER TABLE exam_contracts ADD COLUMN scan_file_path TEXT NOT NULL DEFAULT ''",
    )
    .execute(&pool)
    .await;
    let _ = sqlx::query(
        "ALTER TABLE exam_contracts ADD COLUMN default_templates TEXT NOT NULL DEFAULT 'TREN_18'",
    )
    .execute(&pool)
    .await;

    // 4. Tạo bảng catalogs & catalog_meta (lưu version/timestamp cho caching)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS catalogs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cat_type TEXT NOT NULL,
            code TEXT NOT NULL,
            name TEXT NOT NULL,
            parent_code TEXT NOT NULL DEFAULT '',
            is_active INTEGER NOT NULL DEFAULT 1,
            is_default INTEGER NOT NULL DEFAULT 0,
            UNIQUE(cat_type, code, parent_code)
        );
        CREATE INDEX IF NOT EXISTS idx_catalogs_type_parent ON catalogs(cat_type, parent_code);
        CREATE INDEX IF NOT EXISTS idx_catalogs_code ON catalogs(cat_type, code);

        CREATE TABLE IF NOT EXISTS catalog_meta (
            cat_type TEXT PRIMARY KEY,
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        "#,
    )
    .execute(&pool)
    .await?;

    let _ = sqlx::query("ALTER TABLE catalogs ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1").execute(&pool).await;
    let _ = sqlx::query("ALTER TABLE catalogs ADD COLUMN is_default INTEGER NOT NULL DEFAULT 0").execute(&pool).await;

    // Seed dữ liệu tài khoản mặc định
    let (user_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    if user_count == 0 {
        let default_users = [
            ("admin", "admin123", "Quản trị viên Hệ thống", "admin"),
            ("letan", "123456", "CN. Nguyễn Thị Lễ Tân", "reception"),
            ("nvtiensu", "123456", "ĐD. Lê Thị Tiền Sử", "tiensu"),
            ("bstheluc", "123456", "ĐD. Trần Văn Thể Lực", "bs_theluc"),
            ("bsnoi", "123456", "BS.CKI Lê Văn Nội Khoa", "bs_noi"),
            ("bsngoai", "123456", "BS.CKI Phạm Văn Ngoại Khoa", "bs_ngoai"),
            ("bsdalieu", "123456", "BS. Võ Thị Da Liễu", "bs_dalieu"),
            ("bssan", "123456", "BS.CKI Nguyễn Thị Sản Khoa", "bs_san"),
            ("bsmat", "123456", "BS. Trần Thị Nhãn Khoa", "bs_mat"),
            ("bstmh", "123456", "BS.CKI Đặng Văn Tai Mũi Họng", "bs_tmh"),
            ("bsrhm", "123456", "BS. Hoàng Văn Răng Hàm Mặt", "bs_rhm"),
            ("bscls", "123456", "BS. Ngô Thị Cận Lâm Sàng", "bs_cls"),
            ("bsketluan", "123456", "BS.CKII Trương Văn Kết Luận", "bs_ketluan"),
        ];

        for (u, p, name, role) in default_users {
            let p_hash = hash_password(p);
            sqlx::query(
                "INSERT OR IGNORE INTO users (username, password_hash, full_name, role, is_locked) VALUES (?, ?, ?, ?, 0)",
            )
            .bind(u)
            .bind(p_hash)
            .bind(name)
            .bind(role)
            .execute(&pool)
            .await?;
        }
    }

    // Đảm bảo tài khoản nvtiensu luôn có sẵn cho DB đã khởi tạo
    let p_hash = hash_password("123456");
    let _ = sqlx::query(
        "INSERT OR IGNORE INTO users (username, password_hash, full_name, role, is_locked) VALUES ('nvtiensu', ?, 'ĐD. Lê Thị Tiền Sử', 'tiensu', 0)",
    )
    .bind(p_hash)
    .execute(&pool)
    .await;

    // Seed danh mục chuẩn từ SEED_CATALOGS_TSV CHỈ KHI bảng catalogs đang trống (tránh ghi đè/lock DB trên ổ đĩa chung)
    let (cat_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM catalogs")
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));
    if cat_count == 0 {
        seed_catalogs_from_asset(&pool).await?;
    }

    Ok(pool)
}

pub async fn seed_catalogs_from_asset(pool: &SqlitePool) -> Result<usize, sqlx::Error> {
    // Ưu tiên đọc nội dung mới nhất trực tiếp từ file trên đĩa nếu có (ví dụ asset/seed_catalogs.tsv),
    // nếu file không có trên đĩa (ví dụ khi chạy binary độc lập) thì fallback sang nội dung nhúng lúc biên dịch.
    let content = std::fs::read_to_string("asset/seed_catalogs.tsv")
        .or_else(|_| std::fs::read_to_string("../asset/seed_catalogs.tsv"))
        .unwrap_or_else(|_| SEED_CATALOGS_TSV.to_string());

    let mut tx = pool.begin().await?;

    // Xóa sạch bảng catalogs cũ để đảm bảo nạp chính xác 100% dữ liệu từ seed_catalogs.tsv
    sqlx::query("DELETE FROM catalogs").execute(&mut *tx).await?;

    let mut count = 0usize;
    for line in content.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let cat_type = parts[0].trim();
            let code = parts[1].trim();
            let name = parts[2].trim();
            let parent_code = if parts.len() >= 4 { parts[3].trim() } else { "" };
            if !cat_type.is_empty() && !code.is_empty() && !name.is_empty() {
                let res = sqlx::query(
                    r#"
                    INSERT INTO catalogs (cat_type, code, name, parent_code)
                    VALUES (?, ?, ?, ?)
                    ON CONFLICT(cat_type, code, parent_code)
                    DO UPDATE SET name = excluded.name
                    "#,
                )
                .bind(cat_type)
                .bind(code)
                .bind(name)
                .bind(parent_code)
                .execute(&mut *tx)
                .await;
                if res.is_ok() {
                    count += 1;
                }
            }
        }
    }
    tx.commit().await?;

    // Cập nhật timestamp cho toàn bộ cat_type trong catalog_meta
    let _ = sqlx::query(
        r#"
        INSERT INTO catalog_meta (cat_type, updated_at)
        SELECT DISTINCT cat_type, datetime('now', 'localtime') FROM catalogs
        ON CONFLICT(cat_type) DO UPDATE SET updated_at = datetime('now', 'localtime')
        "#,
    )
    .execute(pool)
    .await;

    // Đảm bảo thiết lập sẵn các giá trị mặc định ban đầu
    let _ = sqlx::query("UPDATE catalogs SET is_default = 1 WHERE cat_type = 'TINH_TP' AND code = '11221'").execute(pool).await;
    let _ = sqlx::query("UPDATE catalogs SET is_default = 1 WHERE cat_type = 'NGHE_NGHIEP' AND code = '00'").execute(pool).await;
    let _ = sqlx::query("UPDATE catalogs SET is_default = 1 WHERE cat_type = 'DOI_TUONG' AND code = '13'").execute(pool).await;
    let _ = sqlx::query("UPDATE catalogs SET is_default = 1 WHERE cat_type = 'NGUON_CHI_TRA' AND code = '9'").execute(pool).await;
    let _ = sqlx::query("UPDATE catalogs SET is_default = 1 WHERE cat_type = 'NOI_CAP' AND code = '00'").execute(pool).await;

    Ok(count)
}

#[allow(dead_code)]
pub async fn get_db_info(pool: &SqlitePool, current_db_path: &str) -> DbInfo {
    let file_size_bytes = fs::metadata(current_db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let (records_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ksk_records")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let (users_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let (catalogs_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM catalogs")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    let mut available_db_files = Vec::new();
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "db" || ext == "sqlite" || ext == "sqlite3" {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            available_db_files.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    available_db_files.sort();

    DbInfo {
        current_db_path: current_db_path.to_string(),
        file_size_bytes,
        records_count,
        users_count,
        catalogs_count,
        available_db_files,
    }
}

fn normalize_vi_match(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .replace("tỉnh ", "")
        .replace("thành phố ", "")
        .replace("tp. ", "")
        .replace("tp ", "")
        .replace("phường ", "")
        .replace("xã ", "")
        .replace("thị trấn ", "")
        .trim()
        .to_string()
}

async fn find_catalog_code_by_name(
    pool: &SqlitePool,
    cat_type: &str,
    name_query: &str,
    parent_code: Option<&str>,
) -> Option<String> {
    let q = name_query.trim();
    if q.is_empty() {
        return None;
    }

    let rows: Vec<(String, String)> = if let Some(pc) = parent_code {
        sqlx::query_as(
            "SELECT code, name FROM catalogs WHERE cat_type = ? AND parent_code = ? AND is_active = 1",
        )
        .bind(cat_type)
        .bind(pc)
        .fetch_all(pool)
        .await
        .unwrap_or_default()
    } else {
        sqlx::query_as("SELECT code, name FROM catalogs WHERE cat_type = ? AND is_active = 1")
            .bind(cat_type)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
    };

    let q_lower = q.to_lowercase();
    for (code, name) in &rows {
        if name.trim().to_lowercase() == q_lower {
            return Some(code.clone());
        }
    }

    let q_norm = normalize_vi_match(q);
    for (code, name) in &rows {
        if normalize_vi_match(name) == q_norm {
            return Some(code.clone());
        }
    }

    for (code, name) in &rows {
        let n_norm = normalize_vi_match(name);
        if !q_norm.is_empty() && (n_norm.contains(&q_norm) || q_norm.contains(&n_norm)) {
            return Some(code.clone());
        }
    }

    None
}

pub async fn has_master_key_setup(pool: &SqlitePool) -> bool {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_settings WHERE key = 'recovery_wrapped_mk'")
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
    row.map(|(v,)| !v.trim().is_empty()).unwrap_or(false)
}

/// Khởi tạo Master Key + Recovery Key lần đầu cho Admin và mã hóa dữ liệu cũ (nếu có).
pub async fn initialize_master_key_for_admin(
    pool: &SqlitePool,
    admin_password: &str,
) -> Result<([u8; 32], String), String> {
    if admin_password.trim().is_empty() {
        return Err("Mật khẩu Admin không được để trống".into());
    }
    let mk = crate::crypto::generate_master_key();
    let recovery_key = crate::crypto::generate_recovery_key();
    let admin_wrapped = crate::crypto::wrap_master_key(&mk, admin_password)?;
    let recovery_wrapped = crate::crypto::wrap_master_key(&mk, &recovery_key)?;
    let admin_hash = hash_password(admin_password);

    sqlx::query("UPDATE users SET password_hash = ?, wrapped_mk = ? WHERE username = 'admin'")
        .bind(&admin_hash)
        .bind(&admin_wrapped)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Bọc MK cho các tài khoản mặc định khác nếu đang dùng mật khẩu mặc định 123456
    let def_hash = hash_password("123456");
    if let Ok(def_wrapped) = crate::crypto::wrap_master_key(&mk, "123456") {
        let _ = sqlx::query(
            "UPDATE users SET wrapped_mk = ? WHERE username != 'admin' AND password_hash = ? AND wrapped_mk = ''",
        )
        .bind(&def_wrapped)
        .bind(&def_hash)
        .execute(pool)
        .await;
    }

    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES ('recovery_wrapped_mk', ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&recovery_wrapped)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    // Mã hóa các hồ sơ plaintext hiện có (nếu có)
    if let Ok(rows) = sqlx::query_as::<_, (i64, String, String, String, String)>(
        "SELECT id, ho_ten, ngay_sinh, so_cccd, payload_json FROM ksk_records",
    )
    .fetch_all(pool)
    .await
    {
        for (id, ht, ns, cccd, pj) in rows {
            if !pj.starts_with("ENC1:") || !ht.starts_with("ENC1:") {
                let (e_ht, e_ns, e_cccd, e_pj) = crate::crypto::encrypt_record_fields(
                    Some(&mk),
                    if ht.starts_with("ENC1:") { "" } else { &ht },
                    if ns.starts_with("ENC1:") { "" } else { &ns },
                    if cccd.starts_with("ENC1:") { "" } else { &cccd },
                    if pj.starts_with("ENC1:") { "{}" } else { &pj },
                );
                let _ = sqlx::query(
                    "UPDATE ksk_records SET ho_ten = ?, ngay_sinh = ?, so_cccd = ?, payload_json = ? WHERE id = ?",
                )
                .bind(&e_ht)
                .bind(&e_ns)
                .bind(&e_cccd)
                .bind(&e_pj)
                .bind(id)
                .execute(pool)
                .await;
            }
        }
    }

    Ok((mk, recovery_key))
}

/// Khôi phục tài khoản Admin bằng Recovery Key và đặt mật khẩu mới.
pub async fn recover_admin_with_key(
    pool: &SqlitePool,
    recovery_key: &str,
    new_admin_password: &str,
) -> Result<[u8; 32], String> {
    if new_admin_password.trim().len() < 4 {
        return Err("Mật khẩu mới phải có ít nhất 4 ký tự".into());
    }
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_settings WHERE key = 'recovery_wrapped_mk'")
            .fetch_optional(pool)
            .await
            .map_err(|e| e.to_string())?;
    let wrapped = row
        .map(|(v,)| v)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| "Hệ thống chưa thiết lập Recovery Key".to_string())?;

    let mk = crate::crypto::unwrap_master_key(&wrapped, recovery_key)?;
    let new_wrapped = crate::crypto::wrap_master_key(&mk, new_admin_password)?;
    let new_hash = hash_password(new_admin_password);

    sqlx::query("UPDATE users SET password_hash = ?, wrapped_mk = ?, is_locked = 0 WHERE username = 'admin'")
        .bind(&new_hash)
        .bind(&new_wrapped)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(mk)
}

/// Tự động backup database SQLite về thư mục local (tải bản sao về máy) và dọn các bản cũ.
pub async fn backup_database(
    pool: &SqlitePool,
    db_path: &str,
    backup_dir: &str,
    keep_count: usize,
) -> Result<String, String> {
    let dir = if backup_dir.trim().is_empty() {
        "backups"
    } else {
        backup_dir.trim()
    };
    fs::create_dir_all(dir).map_err(|e| format!("Không thể tạo thư mục backup: {}", e))?;

    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    let target_path = std::path::Path::new(dir).join(format!("ksk_backup_{}.db", ts));
    let target_str = target_path.to_string_lossy().to_string();
    let escaped = target_str.replace('\'', "''");

    let vacuum_sql = format!("VACUUM INTO '{}'", escaped);
    if let Err(e) = sqlx::query(&vacuum_sql).execute(pool).await {
        tracing::warn!("VACUUM INTO that bai ({}), thu copy truc tiep...", e);
        fs::copy(db_path, &target_path)
            .map_err(|ce| format!("Lỗi copy database backup: {}", ce))?;
    }

    // Dọn bớt các file backup cũ nếu vượt quá keep_count
    if keep_count > 0 {
        if let Ok(entries) = fs::read_dir(dir) {
            let mut files: Vec<(String, std::path::PathBuf)> = entries
                .flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with("ksk_backup_") && name.ends_with(".db") {
                        Some((name, e.path()))
                    } else {
                        None
                    }
                })
                .collect();
            files.sort_by(|a, b| b.0.cmp(&a.0));
            for (_, old_path) in files.into_iter().skip(keep_count) {
                let _ = fs::remove_file(old_path);
            }
        }
    }

    Ok(target_str)
}

/// Upsert thông tin hành chính từ HIS (hoặc AutoHotkey v2 UIA Bridge) vào bảng ksk_records
pub async fn upsert_patient_from_his(
    pool: &SqlitePool,
    cfg: &AppConfig,
    req: &HisPushRequest,
    master_key: Option<&[u8; 32]>,
) -> Result<(i64, Value), String> {
    let ma_tn = req.ma_tiep_nhan.trim().to_string();
    if ma_tn.is_empty() {
        return Err("Thiếu mã tiếp nhận (ma_tiep_nhan)".to_string());
    }

    // Kiểm tra hồ sơ đã tồn tại chưa để giữ nguyên dữ liệu khám lâm sàng/CLS nếu có
    let existing: Option<(i64, String, i64)> = sqlx::query_as(
        "SELECT id, payload_json, status FROM ksk_records WHERE ma_tiep_nhan = ?",
    )
    .bind(&ma_tn)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut payload_map: Map<String, Value> = if let Some((_, ref raw_json, _)) = existing {
        let dec_json = crate::crypto::decrypt_field(master_key, raw_json);
        serde_json::from_str(&dec_json).unwrap_or_default()
    } else {
        Map::new()
    };

    // Chuẩn hóa ngày vào (dd/MM/yyyy)
    let ngay_vao = if !req.ngay_vao.trim().is_empty() {
        req.ngay_vao.trim().chars().take(10).collect::<String>()
    } else {
        chrono::Local::now().format("%d/%m/%Y").to_string()
    };

    // Chuẩn hóa giới tính ("1" Nam, "2" Nữ)
    let gioi_tinh = match req.gioi_tinh.trim().to_lowercase().as_str() {
        "2" | "nữ" | "nu" | "female" | "f" => "2".to_string(),
        _ => "1".to_string(),
    };

    // Resolve Dân tộc, Nghề nghiệp, Tỉnh/TP, Xã/Phường từ catalogs
    let ma_dan_toc = find_catalog_code_by_name(pool, "DAN_TOC", &req.ten_dan_toc, None)
        .await
        .unwrap_or_else(|| "01".to_string());

    let ma_nghe_nghiep = find_catalog_code_by_name(pool, "NGHE_NGHIEP", &req.ten_nghe_nghiep, None)
        .await
        .unwrap_or_else(|| "00".to_string());

    let mut ma_tinh = "11221".to_string(); // Mặc định Gia Lai
    let mut ma_xa = String::new();
    let xp_tinh = req.ten_xa_phuong_tinh.trim();
    if !xp_tinh.is_empty() {
        let parts: Vec<&str> = xp_tinh.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 2 {
            let xa_part = parts[0];
            let tinh_part = parts[parts.len() - 1];
            if let Some(t_code) = find_catalog_code_by_name(pool, "TINH_TP", tinh_part, None).await
            {
                ma_tinh = t_code;
            }
            if let Some(x_code) =
                find_catalog_code_by_name(pool, "XA_PHUONG", xa_part, Some(&ma_tinh)).await
            {
                ma_xa = x_code;
            } else if let Some(x_code) =
                find_catalog_code_by_name(pool, "XA_PHUONG", xa_part, None).await
            {
                ma_xa = x_code;
            }
        } else if let Some(x_code) =
            find_catalog_code_by_name(pool, "XA_PHUONG", xp_tinh, Some(&ma_tinh)).await
        {
            ma_xa = x_code;
        }
    }

    let dia_chi = if !req.dia_chi.trim().is_empty() {
        req.dia_chi.trim().to_string()
    } else if !req.so_nha.trim().is_empty() && !xp_tinh.is_empty() {
        format!("{}, {}", req.so_nha.trim(), xp_tinh)
    } else {
        req.so_nha.trim().to_string()
    };

    let noicap_cccd = if !req.noicap_cccd.trim().is_empty() {
        req.noicap_cccd.trim().to_string()
    } else {
        "Cục Cảnh sát QLHCVTTXH".to_string()
    };

    let ngaycap_cccd = req.ngaycap_cccd.trim().chars().take(10).collect::<String>();

    let ho_ten = req.ho_ten.trim().to_uppercase();
    let ma_y_te = req.ma_y_te.trim().to_string();
    let ngay_sinh = req.ngay_sinh.trim().chars().take(10).collect::<String>();
    let so_cccd = req.so_cccd.trim().to_string();

    // Cập nhật các trường hành chính vào payload_map
    payload_map.insert("MA_TIEP_NHAN".to_string(), json!(ma_tn));
    payload_map.insert("MA_LK".to_string(), json!(ma_tn));
    payload_map.insert("MA_Y_TE".to_string(), json!(ma_y_te));
    payload_map.insert("HO_TEN".to_string(), json!(ho_ten));
    payload_map.insert("GIOI_TINH".to_string(), json!(gioi_tinh));
    payload_map.insert("NGAY_SINH".to_string(), json!(ngay_sinh));
    payload_map.insert("MA_DAN_TOC".to_string(), json!(ma_dan_toc));
    payload_map.insert("SO_CCCD".to_string(), json!(so_cccd));
    payload_map.insert("NGAYCAP_CCCD".to_string(), json!(ngaycap_cccd));
    payload_map.insert("NOICAP_CCCD".to_string(), json!(noicap_cccd));
    payload_map.insert("DIA_CHI".to_string(), json!(dia_chi));
    payload_map.insert("MATINH_CU_TRU".to_string(), json!(ma_tinh));
    if !ma_xa.is_empty() {
        payload_map.insert("MAXA_CU_TRU".to_string(), json!(ma_xa));
    }
    payload_map.insert("MA_NGHE_NGHIEP".to_string(), json!(ma_nghe_nghiep));
    payload_map.insert("DIEN_THOAI".to_string(), json!(req.so_dien_thoai.trim()));
    payload_map.insert("NGAY_VAO".to_string(), json!(ngay_vao));
    payload_map.insert("MA_CSKCB".to_string(), json!(cfg.ma_cskcb));
    payload_map.insert("MA_GTIN_CSKCB".to_string(), json!(cfg.ma_gtin_cskcb));
    if !req.ly_do_vv.trim().is_empty() {
        payload_map.insert("LY_DO_VV".to_string(), json!(req.ly_do_vv.trim()));
    }
    if !req.tien_su_benh_raw.trim().is_empty() {
        payload_map.insert("HIS_TIEN_SU_RAW".to_string(), json!(req.tien_su_benh_raw.trim()));
    }
    payload_map
        .entry("DOI_TUONG".to_string())
        .or_insert_with(|| json!("13"));
    payload_map
        .entry("NGUON_CHI_TRA".to_string())
        .or_insert_with(|| json!("9"));
    payload_map
        .entry("MA_LOAI_KCB".to_string())
        .or_insert_with(|| json!("01"));

    let payload_val = Value::Object(payload_map);
    let payload_str = serde_json::to_string(&payload_val).map_err(|e| e.to_string())?;
    let (enc_ho_ten, enc_ngay_sinh, enc_so_cccd, enc_payload_str) =
        crate::crypto::encrypt_record_fields(
            master_key,
            &ho_ten,
            &ngay_sinh,
            &so_cccd,
            &payload_str,
        );

    sqlx::query(
        r#"
        INSERT INTO ksk_records (
            ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao,
            status, payload_json, updated_by, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, 'his_bridge', datetime('now', 'localtime'))
        ON CONFLICT(ma_tiep_nhan) DO UPDATE SET
            ma_y_te = excluded.ma_y_te,
            ho_ten = excluded.ho_ten,
            gioi_tinh = excluded.gioi_tinh,
            ngay_sinh = excluded.ngay_sinh,
            so_cccd = excluded.so_cccd,
            ngay_vao = excluded.ngay_vao,
            payload_json = excluded.payload_json,
            updated_by = 'his_bridge',
            updated_at = datetime('now', 'localtime')
        "#,
    )
    .bind(&ma_tn)
    .bind(&ma_y_te)
    .bind(&enc_ho_ten)
    .bind(&gioi_tinh)
    .bind(&enc_ngay_sinh)
    .bind(&enc_so_cccd)
    .bind(&ngay_vao)
    .bind(&enc_payload_str)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    let (id,): (i64,) =
        sqlx::query_as("SELECT id FROM ksk_records WHERE ma_tiep_nhan = ?")
            .bind(&ma_tn)
            .fetch_one(pool)
            .await
            .map_err(|e| e.to_string())?;

    Ok((id, payload_val))
}

// =========================================================================
// Quản lý Công ty, Hợp đồng khám đoàn & Import hàng loạt
// =========================================================================

pub async fn list_companies(pool: &SqlitePool) -> Vec<crate::models::Company> {
    sqlx::query_as::<_, crate::models::Company>(
        "SELECT id, code, name, tax_code, address, contact_person, contact_phone, note, is_active, created_at FROM companies ORDER BY is_active DESC, name ASC",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

pub async fn upsert_company(
    pool: &SqlitePool,
    edit_id: Option<i64>,
    code: &str,
    name: &str,
    tax_code: &str,
    address: &str,
    contact_person: &str,
    contact_phone: &str,
    note: &str,
) -> Result<i64, String> {
    let code_t = code.trim();
    let name_t = name.trim();
    if code_t.is_empty() || name_t.is_empty() {
        return Err("Mã công ty và Tên công ty không được để trống!".into());
    }
    if let Some(id) = edit_id {
        sqlx::query(
            "UPDATE companies SET code = ?, name = ?, tax_code = ?, address = ?, contact_person = ?, contact_phone = ?, note = ? WHERE id = ?",
        )
        .bind(code_t)
        .bind(name_t)
        .bind(tax_code.trim())
        .bind(address.trim())
        .bind(contact_person.trim())
        .bind(contact_phone.trim())
        .bind(note.trim())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Lỗi cập nhật công ty: {}", e))?;
        Ok(id)
    } else {
        let r = sqlx::query(
            "INSERT INTO companies (code, name, tax_code, address, contact_person, contact_phone, note) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(code_t)
        .bind(name_t)
        .bind(tax_code.trim())
        .bind(address.trim())
        .bind(contact_person.trim())
        .bind(contact_phone.trim())
        .bind(note.trim())
        .execute(pool)
        .await
        .map_err(|e| format!("Lỗi thêm công ty (mã có thể đã tồn tại): {}", e))?;
        Ok(r.last_insert_rowid())
    }
}

pub async fn toggle_company_active(pool: &SqlitePool, id: i64, is_active: bool) -> Result<(), String> {
    sqlx::query("UPDATE companies SET is_active = ? WHERE id = ?")
        .bind(if is_active { 1 } else { 0 })
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn list_exam_contracts(
    pool: &SqlitePool,
    company_id: Option<i64>,
) -> Vec<crate::models::ExamContractSummary> {
    let base_sql = r#"
        SELECT
            ec.id,
            ec.company_id,
            c.code AS company_code,
            c.name AS company_name,
            ec.contract_code,
            ec.title,
            ec.exam_date,
            ec.expected_count,
            ec.status,
            ec.scan_file_path,
            ec.default_templates,
            ec.note,
            COALESCE(COUNT(r.id), 0) AS total_records,
            COALESCE(SUM(CASE WHEN r.status = 1 THEN 1 ELSE 0 END), 0) AS done_records,
            COALESCE(SUM(CASE WHEN r.status = 2 THEN 1 ELSE 0 END), 0) AS signed_records,
            ec.created_at
        FROM exam_contracts ec
        JOIN companies c ON c.id = ec.company_id
        LEFT JOIN ksk_records r ON r.contract_id = ec.id AND r.exam_mode = 'doan'
    "#;
    if let Some(cid) = company_id {
        let sql = format!("{} WHERE ec.company_id = ? GROUP BY ec.id ORDER BY ec.status ASC, ec.id DESC", base_sql);
        sqlx::query_as::<_, crate::models::ExamContractSummary>(&sql)
            .bind(cid)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
    } else {
        let sql = format!("{} GROUP BY ec.id ORDER BY ec.status ASC, ec.id DESC", base_sql);
        sqlx::query_as::<_, crate::models::ExamContractSummary>(&sql)
            .fetch_all(pool)
            .await
            .unwrap_or_default()
    }
}

pub async fn upsert_exam_contract(
    pool: &SqlitePool,
    edit_id: Option<i64>,
    company_id: i64,
    contract_code: &str,
    title: &str,
    exam_date: &str,
    expected_count: i64,
    default_templates: &str,
    note: &str,
) -> Result<i64, String> {
    let code_t = contract_code.trim();
    let title_t = title.trim();
    let tpl_t = if default_templates.trim().is_empty() {
        "TREN_18"
    } else {
        default_templates.trim()
    };
    if company_id <= 0 || code_t.is_empty() || title_t.is_empty() {
        return Err("Vui lòng chọn công ty, nhập Mã hợp đồng và Tên đợt khám!".into());
    }
    if let Some(id) = edit_id {
        sqlx::query(
            "UPDATE exam_contracts SET company_id = ?, contract_code = ?, title = ?, exam_date = ?, expected_count = ?, default_templates = ?, note = ? WHERE id = ?",
        )
        .bind(company_id)
        .bind(code_t)
        .bind(title_t)
        .bind(exam_date.trim())
        .bind(expected_count)
        .bind(tpl_t)
        .bind(note.trim())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| format!("Lỗi cập nhật hợp đồng: {}", e))?;
        Ok(id)
    } else {
        let r = sqlx::query(
            "INSERT INTO exam_contracts (company_id, contract_code, title, exam_date, expected_count, default_templates, note) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code_t)
        .bind(title_t)
        .bind(exam_date.trim())
        .bind(expected_count)
        .bind(tpl_t)
        .bind(note.trim())
        .execute(pool)
        .await
        .map_err(|e| format!("Lỗi tạo hợp đồng (mã HĐ có thể đã tồn tại): {}", e))?;
        Ok(r.last_insert_rowid())
    }
}

pub async fn set_exam_contract_status(pool: &SqlitePool, id: i64, status: i64) -> Result<(), String> {
    sqlx::query("UPDATE exam_contracts SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn update_exam_contract_scan_file(
    pool: &SqlitePool,
    id: i64,
    scan_file_path: &str,
) -> Result<(), String> {
    sqlx::query("UPDATE exam_contracts SET scan_file_path = ? WHERE id = ?")
        .bind(scan_file_path.trim())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn list_contract_records(
    pool: &SqlitePool,
    contract_id: i64,
    master_key: Option<&[u8; 32]>,
) -> Vec<crate::models::KskRecordRow> {
    let mut rows = sqlx::query_as::<_, crate::models::KskRecordRow>(
        "SELECT id, ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, payload_json, updated_by, created_at, updated_at FROM ksk_records WHERE exam_mode = 'doan' AND contract_id = ? ORDER BY id ASC",
    )
    .bind(contract_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    for r in &mut rows {
        crate::crypto::decrypt_record_row(master_key, r);
    }
    rows
}

pub async fn get_existing_ma_tiep_nhan_set(pool: &SqlitePool) -> std::collections::HashSet<String> {
    sqlx::query_as::<_, (String,)>("SELECT ma_tiep_nhan FROM ksk_records")
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(s,)| s.trim().to_string())
        .collect()
}

pub async fn get_existing_contract_cccd_set(
    pool: &SqlitePool,
    contract_id: i64,
    master_key: Option<&[u8; 32]>,
) -> std::collections::HashSet<String> {
    let rows = sqlx::query_as::<_, (String,)>(
        "SELECT so_cccd FROM ksk_records WHERE exam_mode = 'doan' AND contract_id = ?",
    )
    .bind(contract_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .filter_map(|(raw,)| {
            let dec = crate::crypto::decrypt_field(master_key, &raw);
            let t = dec.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        })
        .collect()
}

fn apply_group_row_to_payload(
    payload: &mut Map<String, Value>,
    r: &crate::models::GroupImportRow,
    ma_cskcb: &str,
    ma_gtin: &str,
) {
    payload.insert("MA_TIEP_NHAN".into(), Value::String(r.ma_tiep_nhan.clone()));
    payload.insert("MA_Y_TE".into(), Value::String(r.ma_y_te.clone()));
    payload.insert("HO_TEN".into(), Value::String(r.ho_ten.clone()));
    payload.insert("NGAY_SINH".into(), Value::String(r.ngay_sinh.clone()));
    payload.insert("GIOI_TINH".into(), Value::String(r.gioi_tinh.clone()));
    payload.insert("SO_CCCD".into(), Value::String(r.so_cccd.clone()));
    payload.insert("NGAYCAP_CCCD".into(), Value::String(r.ngaycap_cccd.clone()));
    payload.insert("NOICAP_CCCD".into(), Value::String(r.noicap_cccd.clone()));
    payload.insert("DIEN_THOAI".into(), Value::String(r.dien_thoai.clone()));
    payload.insert("MATINH_CU_TRU".into(), Value::String(r.matinh_cu_tru.clone()));
    payload.insert("MAXA_CU_TRU".into(), Value::String(r.maxa_cu_tru.clone()));
    payload.insert("DIA_CHI".into(), Value::String(r.dia_chi.clone()));
    payload.insert("MA_DAN_TOC".into(), Value::String(r.ma_dan_toc.clone()));
    payload.insert("MA_NGHE_NGHIEP".into(), Value::String(r.ma_nghe_nghiep.clone()));
    payload.insert(
        "NOI_LAM_VIEC_HOC_TAP".into(),
        Value::String(r.noi_lam_viec_hoc_tap.clone()),
    );
    payload.insert("MA_LOAI_KCB".into(), Value::String(r.ma_loai_kcb.clone()));
    payload.insert("NGAY_VAO".into(), Value::String(r.ngay_vao.clone()));
    payload.insert("NHOM_MAU".into(), Value::String(r.nhom_mau.clone()));
    payload.insert("LY_DO_VV".into(), Value::String(r.ly_do_vv.clone()));
    payload.insert("MA_CSKCB".into(), Value::String(ma_cskcb.to_string()));
    payload.insert("MA_GTIN_CSKCB".into(), Value::String(ma_gtin.to_string()));
    let ma_lk = format!("{}_{}", ma_cskcb, r.ma_tiep_nhan);
    payload.insert("MA_LK".into(), Value::String(ma_lk));
}

pub async fn update_record_templates(
    pool: &SqlitePool,
    record_id: i64,
    exam_templates: &str,
) -> Result<(), String> {
    let tpl = if exam_templates.trim().is_empty() {
        "TREN_18"
    } else {
        exam_templates.trim()
    };
    sqlx::query("UPDATE ksk_records SET exam_templates = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(tpl)
        .bind(record_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn get_contract_record_for_edit(
    pool: &SqlitePool,
    record_id: i64,
    master_key: Option<&[u8; 32]>,
) -> Result<(i64, String, Map<String, Value>), String> {
    let (status, exam_templates, enc_json) = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT status, exam_templates, payload_json FROM ksk_records WHERE id = ?",
    )
    .bind(record_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Không tìm thấy hồ sơ!".to_string())?;
    let json_str = crate::crypto::decrypt_field(master_key, &enc_json);
    let payload: Map<String, Value> = serde_json::from_str(&json_str).unwrap_or_default();
    Ok((status, exam_templates, payload))
}

pub async fn upsert_group_member_record(
    pool: &SqlitePool,
    contract_id: i64,
    edit_record_id: Option<i64>,
    exam_templates: &str,
    ma_cskcb: &str,
    ma_gtin: &str,
    updated_by: &str,
    master_key: Option<&[u8; 32]>,
    row: &crate::models::GroupImportRow,
) -> Result<(), String> {
    let ma_tn = row.ma_tiep_nhan.trim();
    let ho_ten = row.ho_ten.trim();
    let tpl = if exam_templates.trim().is_empty() {
        "TREN_18"
    } else {
        exam_templates.trim()
    };
    if ma_tn.is_empty() || ho_ten.is_empty() {
        return Err("Mã tiếp nhận và Họ tên không được để trống!".into());
    }

    let dup = if let Some(rid) = edit_record_id {
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM ksk_records WHERE ma_tiep_nhan = ? AND id != ? LIMIT 1",
        )
        .bind(ma_tn)
        .bind(rid)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM ksk_records WHERE ma_tiep_nhan = ? LIMIT 1",
        )
        .bind(ma_tn)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?
    };
    if dup.is_some() {
        return Err(format!("Mã tiếp nhận '{}' đã tồn tại!", ma_tn));
    }

    if let Some(rid) = edit_record_id {
        let (status, _, mut payload) = get_contract_record_for_edit(pool, rid, master_key).await?;
        if status == 2 {
            return Err("Hồ sơ đã ký khóa, không thể chỉnh sửa!".into());
        }
        apply_group_row_to_payload(&mut payload, row, ma_cskcb, ma_gtin);
        let json_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
        let (enc_ho_ten, enc_ngay_sinh, enc_so_cccd, enc_json_str) =
            crate::crypto::encrypt_record_fields(
                master_key,
                &row.ho_ten,
                &row.ngay_sinh,
                &row.so_cccd,
                &json_str,
            );
        sqlx::query(
            "UPDATE ksk_records SET ma_tiep_nhan = ?, ma_y_te = ?, ho_ten = ?, gioi_tinh = ?, ngay_sinh = ?, so_cccd = ?, ngay_vao = ?, exam_templates = ?, payload_json = ?, updated_by = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND contract_id = ?",
        )
        .bind(ma_tn)
        .bind(&row.ma_y_te)
        .bind(&enc_ho_ten)
        .bind(&row.gioi_tinh)
        .bind(&enc_ngay_sinh)
        .bind(&enc_so_cccd)
        .bind(&row.ngay_vao)
        .bind(tpl)
        .bind(&enc_json_str)
        .bind(updated_by)
        .bind(rid)
        .bind(contract_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    } else {
        let mut payload = Map::new();
        apply_group_row_to_payload(&mut payload, row, ma_cskcb, ma_gtin);
        let json_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
        let (enc_ho_ten, enc_ngay_sinh, enc_so_cccd, enc_json_str) =
            crate::crypto::encrypt_record_fields(
                master_key,
                &row.ho_ten,
                &row.ngay_sinh,
                &row.so_cccd,
                &json_str,
            );
        sqlx::query(
            "INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json, updated_by) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 'doan', ?, ?, ?, ?)",
        )
        .bind(ma_tn)
        .bind(&row.ma_y_te)
        .bind(&enc_ho_ten)
        .bind(&row.gioi_tinh)
        .bind(&enc_ngay_sinh)
        .bind(&enc_so_cccd)
        .bind(&row.ngay_vao)
        .bind(contract_id)
        .bind(tpl)
        .bind(&enc_json_str)
        .bind(updated_by)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub async fn bulk_update_contract_records_common(
    pool: &SqlitePool,
    contract_id: i64,
    noi_lam_viec: Option<String>,
    nguon_chi_tra: Option<String>,
    doi_tuong: Option<String>,
    ly_do_vv: Option<String>,
    exam_templates: Option<String>,
    updated_by: &str,
    master_key: Option<&[u8; 32]>,
) -> Result<usize, String> {
    if noi_lam_viec.is_none()
        && nguon_chi_tra.is_none()
        && doi_tuong.is_none()
        && ly_do_vv.is_none()
        && exam_templates.is_none()
    {
        return Err("Vui lòng chọn ít nhất một trường để cập nhật!".into());
    }
    let rows = sqlx::query_as::<_, (i64, i64, String, String)>(
        "SELECT id, status, exam_templates, payload_json FROM ksk_records WHERE contract_id = ?",
    )
    .bind(contract_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut updated = 0usize;
    for (rid, status, cur_tpl, enc_json) in rows {
        if status == 2 {
            continue;
        }
        let json_str = crate::crypto::decrypt_field(master_key, &enc_json);
        let mut payload: Map<String, Value> = serde_json::from_str(&json_str).unwrap_or_default();
        if let Some(ref v) = noi_lam_viec {
            payload.insert("NOI_LAM_VIEC_HOC_TAP".into(), Value::String(v.clone()));
        }
        if let Some(ref v) = nguon_chi_tra {
            payload.insert("NGUON_CHI_TRA".into(), Value::String(v.clone()));
        }
        if let Some(ref v) = doi_tuong {
            payload.insert("DOI_TUONG".into(), Value::String(v.clone()));
        }
        if let Some(ref v) = ly_do_vv {
            payload.insert("LY_DO_VV".into(), Value::String(v.clone()));
        }
        let new_tpl = exam_templates
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&cur_tpl);
        let new_json = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
        let enc_payload = match master_key {
            Some(mk) => crate::crypto::encrypt_field(mk, &new_json),
            None => new_json,
        };
        sqlx::query(
            "UPDATE ksk_records SET exam_templates = ?, payload_json = ?, updated_by = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(new_tpl)
        .bind(&enc_payload)
        .bind(updated_by)
        .bind(rid)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        updated += 1;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(updated)
}

pub async fn batch_insert_group_records(
    pool: &SqlitePool,
    contract_id: i64,
    ma_cskcb: &str,
    ma_gtin: &str,
    updated_by: &str,
    master_key: Option<&[u8; 32]>,
    rows: &[crate::models::GroupImportRow],
) -> Result<(usize, usize), String> {
    let default_tpl: String = sqlx::query_scalar(
        "SELECT default_templates FROM exam_contracts WHERE id = ?",
    )
    .bind(contract_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .filter(|s: &String| !s.trim().is_empty())
    .unwrap_or_else(|| "TREN_18".into());

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let mut inserted = 0usize;
    let mut skipped = 0usize;

    for r in rows {
        if r.error.is_some() {
            skipped += 1;
            continue;
        }
        let mut payload = Map::new();
        apply_group_row_to_payload(&mut payload, r, ma_cskcb, ma_gtin);

        let json_str = serde_json::to_string(&payload).map_err(|e| e.to_string())?;
        let (enc_ho_ten, enc_ngay_sinh, enc_so_cccd, enc_json_str) =
            crate::crypto::encrypt_record_fields(
                master_key,
                &r.ho_ten,
                &r.ngay_sinh,
                &r.so_cccd,
                &json_str,
            );

        let res = sqlx::query(
            "INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json, updated_by) VALUES (?, ?, ?, ?, ?, ?, ?, 0, 'doan', ?, ?, ?, ?)",
        )
        .bind(&r.ma_tiep_nhan)
        .bind(&r.ma_y_te)
        .bind(&enc_ho_ten)
        .bind(&r.gioi_tinh)
        .bind(&enc_ngay_sinh)
        .bind(&enc_so_cccd)
        .bind(&r.ngay_vao)
        .bind(contract_id)
        .bind(&default_tpl)
        .bind(&enc_json_str)
        .bind(updated_by)
        .execute(&mut *tx)
        .await;

        match res {
            Ok(_) => inserted += 1,
            Err(e) => return Err(format!("Lỗi dòng {} ({}): {}", r.row_index, r.ma_tiep_nhan, e)),
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok((inserted, skipped))
}

#[derive(Debug, Clone)]
pub struct SplitExportSummary {
    pub dest_path: String,
    pub contract_count: usize,
    pub company_count: usize,
    pub record_count: usize,
    pub contract_codes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MergeContractItem {
    pub company_code: String,
    pub company_name: String,
    pub contract_code: String,
    pub title: String,
    pub total_records: usize,
    pub updated_count: usize,
    pub inserted_count: usize,
}

#[derive(Debug, Clone)]
pub struct MergePreviewSummary {
    pub source_path: String,
    pub exported_at: String,
    pub contracts: Vec<MergeContractItem>,
    pub total_records: usize,
    pub total_updated: usize,
    pub total_inserted: usize,
    pub same_master_key: bool,
}

async fn open_readonly_sqlite(path: &str) -> Result<SqlitePool, String> {
    if !std::path::Path::new(path).exists() {
        return Err(format!("Không tìm thấy file CSDL: {}", path));
    }
    let conn_str = format!("sqlite:{}", path);
    let options = SqliteConnectOptions::from_str(&conn_str)
        .map_err(|e| e.to_string())?
        .create_if_missing(false)
        .read_only(true)
        .busy_timeout(Duration::from_secs(10));
    SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .map_err(|e| format!("Không thể mở file SQLite {}: {}", path, e))
}

pub async fn export_contracts_split_db(
    source_pool: &SqlitePool,
    contract_ids: &[i64],
    dest_path: &str,
) -> Result<SplitExportSummary, String> {
    if contract_ids.is_empty() {
        return Err("Vui lòng chọn ít nhất 1 hợp đồng để tách DB!".into());
    }
    let dest_p = std::path::Path::new(dest_path);
    if dest_p.exists() {
        fs::remove_file(dest_p)
            .map_err(|e| format!("Không thể ghi đè file đích ({}): {}", dest_path, e))?;
    }

    let dest_pool = init_db(dest_path)
        .await
        .map_err(|e| format!("Lỗi khởi tạo DB tách: {}", e))?;

    let mut tx = dest_pool.begin().await.map_err(|e| e.to_string())?;

    // Xóa dữ liệu seed mặc định để chép nguyên vẹn users, app_settings, catalogs từ DB chính
    sqlx::query("DELETE FROM users").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM app_settings").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM catalogs").execute(&mut *tx).await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM catalog_meta").execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let users: Vec<(String, String, String, String, i64, String, String)> = sqlx::query_as(
        "SELECT username, password_hash, full_name, role, is_locked, wrapped_mk, created_at FROM users",
    )
    .fetch_all(source_pool)
    .await
    .map_err(|e| e.to_string())?;
    for (u, ph, fnm, role, locked, wmk, cat) in users {
        sqlx::query(
            "INSERT INTO users (username, password_hash, full_name, role, is_locked, wrapped_mk, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(u)
        .bind(ph)
        .bind(fnm)
        .bind(role)
        .bind(locked)
        .bind(wmk)
        .bind(cat)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    let settings: Vec<(String, String)> = sqlx::query_as("SELECT key, value FROM app_settings")
        .fetch_all(source_pool)
        .await
        .map_err(|e| e.to_string())?;
    for (k, v) in settings {
        sqlx::query("INSERT INTO app_settings (key, value) VALUES (?, ?)")
            .bind(k)
            .bind(v)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    let catalogs: Vec<(String, String, String, String, i64, i64)> = sqlx::query_as(
        "SELECT cat_type, code, name, parent_code, is_active, is_default FROM catalogs",
    )
    .fetch_all(source_pool)
    .await
    .map_err(|e| e.to_string())?;
    for (ct, code, name, parent, act, def) in catalogs {
        sqlx::query(
            "INSERT INTO catalogs (cat_type, code, name, parent_code, is_active, is_default) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(ct)
        .bind(code)
        .bind(name)
        .bind(parent)
        .bind(act)
        .bind(def)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    let metas: Vec<(String, String)> = sqlx::query_as("SELECT cat_type, updated_at FROM catalog_meta")
        .fetch_all(source_pool)
        .await
        .map_err(|e| e.to_string())?;
    for (ct, ua) in metas {
        sqlx::query("INSERT INTO catalog_meta (cat_type, updated_at) VALUES (?, ?)")
            .bind(ct)
            .bind(ua)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
    }

    let mut company_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
    let mut contract_codes: Vec<String> = Vec::new();
    let mut record_count = 0usize;

    for &src_cid in contract_ids {
        let contract_row: Option<(i64, String, String, String, i64, i64, String, String, String, String)> = sqlx::query_as(
            "SELECT company_id, contract_code, title, exam_date, expected_count, status, scan_file_path, default_templates, note, created_at FROM exam_contracts WHERE id = ?",
        )
        .bind(src_cid)
        .fetch_optional(source_pool)
        .await
        .map_err(|e| e.to_string())?;

        let Some((src_comp_id, c_code, title, ex_date, exp_cnt, status, scan, def_tpl, note, created_at)) = contract_row else {
            continue;
        };

        let dest_comp_id = if let Some(&id) = company_map.get(&src_comp_id) {
            id
        } else {
            let comp: (String, String, String, String, String, String, String, i64, String) = sqlx::query_as(
                "SELECT code, name, tax_code, address, contact_person, contact_phone, note, is_active, created_at FROM companies WHERE id = ?",
            )
            .bind(src_comp_id)
            .fetch_one(source_pool)
            .await
            .map_err(|e| e.to_string())?;
            let res = sqlx::query(
                "INSERT INTO companies (code, name, tax_code, address, contact_person, contact_phone, note, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(comp.0)
            .bind(comp.1)
            .bind(comp.2)
            .bind(comp.3)
            .bind(comp.4)
            .bind(comp.5)
            .bind(comp.6)
            .bind(comp.7)
            .bind(comp.8)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            let nid = res.last_insert_rowid();
            company_map.insert(src_comp_id, nid);
            nid
        };

        let c_res = sqlx::query(
            "INSERT INTO exam_contracts (company_id, contract_code, title, exam_date, expected_count, status, scan_file_path, default_templates, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(dest_comp_id)
        .bind(&c_code)
        .bind(&title)
        .bind(&ex_date)
        .bind(exp_cnt)
        .bind(status)
        .bind(&scan)
        .bind(&def_tpl)
        .bind(&note)
        .bind(&created_at)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
        let dest_cid = c_res.last_insert_rowid();
        contract_codes.push(c_code);

        let recs: Vec<(String, String, String, String, String, String, String, i64, String, String, String, String, String)> = sqlx::query_as(
            "SELECT ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, exam_templates, payload_json, updated_by, created_at FROM ksk_records WHERE contract_id = ?",
        )
        .bind(src_cid)
        .fetch_all(source_pool)
        .await
        .map_err(|e| e.to_string())?;

        for (ma_tn, ma_yt, ht, gt, ns, cccd, nv, st, mode, tpls, payload, uby, cat) in recs {
            sqlx::query(
                "INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json, updated_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(ma_tn)
            .bind(ma_yt)
            .bind(ht)
            .bind(gt)
            .bind(ns)
            .bind(cccd)
            .bind(nv)
            .bind(st)
            .bind(mode)
            .bind(dest_cid)
            .bind(tpls)
            .bind(payload)
            .bind(uby)
            .bind(cat)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            record_count += 1;
        }
    }

    let now_str = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES ('split_contract_codes', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(contract_codes.join(","))
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES ('split_exported_at', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&now_str)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let summary = SplitExportSummary {
        dest_path: dest_path.to_string(),
        contract_count: contract_codes.len(),
        company_count: company_map.len(),
        record_count,
        contract_codes,
    };

    tx.commit().await.map_err(|e| e.to_string())?;
    dest_pool.close().await;
    Ok(summary)
}

pub async fn preview_merge_contract_db(
    target_pool: &SqlitePool,
    source_path: &str,
) -> Result<MergePreviewSummary, String> {
    let src_pool = open_readonly_sqlite(source_path).await?;

    let src_mk: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_settings WHERE key = 'recovery_wrapped_mk'")
            .fetch_optional(&src_pool)
            .await
            .ok()
            .flatten();
    let tgt_mk: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_settings WHERE key = 'recovery_wrapped_mk'")
            .fetch_optional(target_pool)
            .await
            .ok()
            .flatten();
    let same_master_key = match (src_mk, tgt_mk) {
        (Some((a,)), Some((b,))) => a == b,
        _ => true,
    };

    let exported_at: String =
        sqlx::query_as::<_, (String,)>("SELECT value FROM app_settings WHERE key = 'split_exported_at'")
            .fetch_optional(&src_pool)
            .await
            .ok()
            .flatten()
            .map(|(v,)| v)
            .unwrap_or_default();

    let src_contracts: Vec<(i64, String, String, String, String)> = sqlx::query_as(
        "SELECT c.id, co.code, co.name, c.contract_code, c.title \
         FROM exam_contracts c JOIN companies co ON co.id = c.company_id \
         ORDER BY c.id ASC",
    )
    .fetch_all(&src_pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    let mut total_records = 0usize;
    let mut total_updated = 0usize;
    let mut total_inserted = 0usize;

    for (src_cid, co_code, co_name, c_code, title) in src_contracts {
        let ma_tns: Vec<(String,)> =
            sqlx::query_as("SELECT ma_tiep_nhan FROM ksk_records WHERE contract_id = ?")
                .bind(src_cid)
                .fetch_all(&src_pool)
                .await
                .map_err(|e| e.to_string())?;
        let mut updated_count = 0usize;
        let mut inserted_count = 0usize;
        for (ma_tn,) in &ma_tns {
            let exists: Option<(i64,)> =
                sqlx::query_as("SELECT id FROM ksk_records WHERE ma_tiep_nhan = ?")
                    .bind(ma_tn)
                    .fetch_optional(target_pool)
                    .await
                    .ok()
                    .flatten();
            if exists.is_some() {
                updated_count += 1;
            } else {
                inserted_count += 1;
            }
        }
        total_records += ma_tns.len();
        total_updated += updated_count;
        total_inserted += inserted_count;
        items.push(MergeContractItem {
            company_code: co_code,
            company_name: co_name,
            contract_code: c_code,
            title,
            total_records: ma_tns.len(),
            updated_count,
            inserted_count,
        });
    }

    src_pool.close().await;
    Ok(MergePreviewSummary {
        source_path: source_path.to_string(),
        exported_at,
        contracts: items,
        total_records,
        total_updated,
        total_inserted,
        same_master_key,
    })
}

pub async fn merge_contract_db(
    target_pool: &SqlitePool,
    source_path: &str,
    mk: Option<&[u8; 32]>,
    overwrite_all: bool,
) -> Result<(usize, usize), String> {
    let src_pool = open_readonly_sqlite(source_path).await?;
    let src_contracts: Vec<(i64, i64, String, String, String, i64, i64, String, String, String, String)> = sqlx::query_as(
        "SELECT id, company_id, contract_code, title, exam_date, expected_count, status, scan_file_path, default_templates, note, created_at FROM exam_contracts ORDER BY id ASC",
    )
    .fetch_all(&src_pool)
    .await
    .map_err(|e| e.to_string())?;

    if src_contracts.is_empty() {
        src_pool.close().await;
        return Err("File DB nguồn không chứa hợp đồng nào để gộp!".into());
    }

    let mut tx = target_pool.begin().await.map_err(|e| e.to_string())?;
    let mut updated = 0usize;
    let mut inserted = 0usize;

    for (src_cid, src_comp_id, c_code, title, ex_date, exp_cnt, c_status, scan, def_tpl, c_note, c_cat) in src_contracts {
        let comp: (String, String, String, String, String, String, String, i64, String) = sqlx::query_as(
            "SELECT code, name, tax_code, address, contact_person, contact_phone, note, is_active, created_at FROM companies WHERE id = ?",
        )
        .bind(src_comp_id)
        .fetch_one(&src_pool)
        .await
        .map_err(|e| e.to_string())?;

        let target_comp_id: i64 = if let Some((eid,)) =
            sqlx::query_as::<_, (i64,)>("SELECT id FROM companies WHERE code = ?")
                .bind(&comp.0)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?
        {
            eid
        } else {
            let r = sqlx::query(
                "INSERT INTO companies (code, name, tax_code, address, contact_person, contact_phone, note, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(comp.0)
            .bind(comp.1)
            .bind(comp.2)
            .bind(comp.3)
            .bind(comp.4)
            .bind(comp.5)
            .bind(comp.6)
            .bind(comp.7)
            .bind(comp.8)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            r.last_insert_rowid()
        };

        let target_cid: i64 = if let Some((ecid,)) =
            sqlx::query_as::<_, (i64,)>("SELECT id FROM exam_contracts WHERE contract_code = ?")
                .bind(&c_code)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| e.to_string())?
        {
            ecid
        } else {
            let r = sqlx::query(
                "INSERT INTO exam_contracts (company_id, contract_code, title, exam_date, expected_count, status, scan_file_path, default_templates, note, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(target_comp_id)
            .bind(&c_code)
            .bind(&title)
            .bind(&ex_date)
            .bind(exp_cnt)
            .bind(c_status)
            .bind(&scan)
            .bind(&def_tpl)
            .bind(&c_note)
            .bind(&c_cat)
            .execute(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;
            r.last_insert_rowid()
        };

        let recs: Vec<(String, String, String, String, String, String, String, i64, String, String, String, String, String)> = sqlx::query_as(
            "SELECT ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, exam_templates, payload_json, updated_by, created_at FROM ksk_records WHERE contract_id = ?",
        )
        .bind(src_cid)
        .fetch_all(&src_pool)
        .await
        .map_err(|e| e.to_string())?;

        for (ma_tn, ma_yt, s_ht, s_gt, s_ns, s_cccd, s_nv, s_st, s_mode, s_tpls, s_payload, s_uby, s_cat) in recs {
            let existing: Option<(i64, i64, String, String, String)> = sqlx::query_as(
                "SELECT id, status, exam_templates, payload_json, ma_y_te FROM ksk_records WHERE ma_tiep_nhan = ?",
            )
            .bind(&ma_tn)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| e.to_string())?;

            if let Some((t_id, t_st, t_tpls, t_payload_enc, t_mayt)) = existing {
                if overwrite_all {
                    sqlx::query(
                        "UPDATE ksk_records SET ma_y_te = ?, ho_ten = ?, gioi_tinh = ?, ngay_sinh = ?, so_cccd = ?, ngay_vao = ?, status = ?, exam_mode = ?, contract_id = ?, exam_templates = ?, payload_json = ?, updated_by = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
                    )
                    .bind(&ma_yt)
                    .bind(&s_ht)
                    .bind(&s_gt)
                    .bind(&s_ns)
                    .bind(&s_cccd)
                    .bind(&s_nv)
                    .bind(s_st)
                    .bind(&s_mode)
                    .bind(target_cid)
                    .bind(&s_tpls)
                    .bind(&s_payload)
                    .bind(&s_uby)
                    .bind(t_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                } else {
                    // Gộp thông minh: giữ dữ liệu máy chủ, bổ sung/cập nhật từ file khám ngoài và hợp nhất _EXAMINED_SECTIONS
                    let t_plain = crate::crypto::decrypt_field(mk, &t_payload_enc);
                    let s_plain = crate::crypto::decrypt_field(mk, &s_payload);
                    let mut merged_map: Map<String, Value> =
                        serde_json::from_str(&t_plain).unwrap_or_default();
                    let src_map: Map<String, Value> =
                        serde_json::from_str(&s_plain).unwrap_or_default();

                    for (k, v) in src_map {
                        if k == "_EXAMINED_SECTIONS" {
                            let mut set: Vec<String> = merged_map
                                .get("_EXAMINED_SECTIONS")
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .split(',')
                                .map(|x| x.trim().to_string())
                                .filter(|x| !x.is_empty())
                                .collect();
                            if let Some(sv) = v.as_str() {
                                for part in sv.split(',').map(|x| x.trim()).filter(|x| !x.is_empty()) {
                                    if !set.iter().any(|x| x == part) {
                                        set.push(part.to_string());
                                    }
                                }
                            }
                            merged_map.insert(k, Value::String(set.join(",")));
                        } else if let Some(sv) = v.as_str() {
                            if !sv.trim().is_empty() {
                                merged_map.insert(k, Value::String(sv.to_string()));
                            }
                        } else if !v.is_null() {
                            merged_map.insert(k, v);
                        }
                    }

                    let mut tpl_list = crate::templates::parse_template_list(&t_tpls);
                    for tp in crate::templates::parse_template_list(&s_tpls) {
                        if !tpl_list.contains(&tp) {
                            tpl_list.push(tp);
                        }
                    }
                    let merged_tpls = crate::templates::format_template_list(&tpl_list);
                    let merged_json = serde_json::to_string(&merged_map).map_err(|e| e.to_string())?;
                    let merged_payload_enc = match mk {
                        Some(key) => crate::crypto::encrypt_field(key, &merged_json),
                        None => merged_json,
                    };
                    let final_ma_yt = if ma_yt.trim().is_empty() { t_mayt } else { ma_yt };
                    let final_st = t_st.max(s_st);

                    sqlx::query(
                        "UPDATE ksk_records SET ma_y_te = ?, ho_ten = ?, gioi_tinh = ?, ngay_sinh = ?, so_cccd = ?, ngay_vao = ?, status = ?, exam_templates = ?, payload_json = ?, updated_by = ?, updated_at = datetime('now', 'localtime') WHERE id = ?",
                    )
                    .bind(final_ma_yt)
                    .bind(&s_ht)
                    .bind(&s_gt)
                    .bind(&s_ns)
                    .bind(&s_cccd)
                    .bind(&s_nv)
                    .bind(final_st)
                    .bind(&merged_tpls)
                    .bind(&merged_payload_enc)
                    .bind(&s_uby)
                    .bind(t_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| e.to_string())?;
                }
                updated += 1;
            } else {
                sqlx::query(
                    "INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json, updated_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&ma_tn)
                .bind(&ma_yt)
                .bind(&s_ht)
                .bind(&s_gt)
                .bind(&s_ns)
                .bind(&s_cccd)
                .bind(&s_nv)
                .bind(s_st)
                .bind(&s_mode)
                .bind(target_cid)
                .bind(&s_tpls)
                .bind(&s_payload)
                .bind(&s_uby)
                .bind(&s_cat)
                .execute(&mut *tx)
                .await
                .map_err(|e| e.to_string())?;
                inserted += 1;
            }
        }
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    src_pool.close().await;
    Ok((updated, inserted))
}

#[cfg(test)]
mod split_merge_tests {
    use super::*;

    #[tokio::test]
    async fn test_split_multi_contracts_and_smart_merge() {
        let tmp = std::env::temp_dir().join(format!("ksk_split_merge_{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&tmp).unwrap();
        let main_db = tmp.join("main.db");
        let split_db = tmp.join("outreach.db");
        let main_str = main_db.to_string_lossy().to_string();
        let split_str = split_db.to_string_lossy().to_string();

        let main_pool = init_db(&main_str).await.unwrap();
        let (mk, _rk) = initialize_master_key_for_admin(&main_pool, "123456").await.unwrap();

        let comp_id = sqlx::query("INSERT INTO companies (code, name) VALUES ('CTY01', 'Cong ty A')")
            .execute(&main_pool)
            .await
            .unwrap()
            .last_insert_rowid();
        let c1_id = sqlx::query("INSERT INTO exam_contracts (company_id, contract_code, title) VALUES (?, 'HD01', 'Dot 1')")
            .bind(comp_id)
            .execute(&main_pool)
            .await
            .unwrap()
            .last_insert_rowid();
        let c2_id = sqlx::query("INSERT INTO exam_contracts (company_id, contract_code, title) VALUES (?, 'HD02', 'Dot 2')")
            .bind(comp_id)
            .execute(&main_pool)
            .await
            .unwrap()
            .last_insert_rowid();

        let (ht1, ns1, cccd1, p1) = crate::crypto::encrypt_record_fields(
            Some(&mk),
            "NGUYEN VAN A",
            "01/01/1990",
            "052090000001",
            r#"{"HO_TEN":"NGUYEN VAN A","_EXAMINED_SECTIONS":"reception"}"#,
        );
        sqlx::query("INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json) VALUES ('TN01', 'BN01', ?, '1', ?, ?, '16/11/2026', 0, 'doan', ?, 'TREN_18', ?)")
            .bind(&ht1).bind(&ns1).bind(&cccd1).bind(c1_id).bind(&p1)
            .execute(&main_pool).await.unwrap();

        let (ht2, ns2, cccd2, p2) = crate::crypto::encrypt_record_fields(
            Some(&mk),
            "TRAN THI B",
            "02/02/1992",
            "052092000002",
            r#"{"HO_TEN":"TRAN THI B","_EXAMINED_SECTIONS":"reception"}"#,
        );
        sqlx::query("INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json) VALUES ('TN02', 'BN02', ?, '2', ?, ?, '16/11/2026', 0, 'doan', ?, 'TREN_18', ?)")
            .bind(&ht2).bind(&ns2).bind(&cccd2).bind(c2_id).bind(&p2)
            .execute(&main_pool).await.unwrap();

        // 1. Tách cả 2 hợp đồng ra outreach.db
        let summary = export_contracts_split_db(&main_pool, &[c1_id, c2_id], &split_str).await.unwrap();
        assert_eq!(summary.contract_count, 2);
        assert_eq!(summary.record_count, 2);

        // 2. Trên main.db: nhập kết quả CLS cho TN01
        let main_cls_payload = crate::crypto::encrypt_field(
            &mk,
            r#"{"HO_TEN":"NGUYEN VAN A","CLS_HUYET_HOC":"Bình thường","_EXAMINED_SECTIONS":"reception,paraclinical"}"#,
        );
        sqlx::query("UPDATE ksk_records SET payload_json = ? WHERE ma_tiep_nhan = 'TN01'")
            .bind(&main_cls_payload).execute(&main_pool).await.unwrap();

        // 3. Trên outreach.db: khám lâm sàng cho TN01 + thêm người mới TN03 vào HD01
        let split_pool = init_db(&split_str).await.unwrap();
        let split_clin_payload = crate::crypto::encrypt_field(
            &mk,
            r#"{"HO_TEN":"NGUYEN VAN A","NOI_KHOA_TUAN_HOAN":"Bình thường","_EXAMINED_SECTIONS":"reception,clin_noi"}"#,
        );
        sqlx::query("UPDATE ksk_records SET payload_json = ?, status = 1 WHERE ma_tiep_nhan = 'TN01'")
            .bind(&split_clin_payload).execute(&split_pool).await.unwrap();

        let (ht3, ns3, cccd3, p3) = crate::crypto::encrypt_record_fields(
            Some(&mk),
            "LE VAN C",
            "03/03/1995",
            "052095000003",
            r#"{"HO_TEN":"LE VAN C","_EXAMINED_SECTIONS":"reception"}"#,
        );
        let split_c1_id: (i64,) = sqlx::query_as("SELECT id FROM exam_contracts WHERE contract_code = 'HD01'")
            .fetch_one(&split_pool).await.unwrap();
        sqlx::query("INSERT INTO ksk_records (ma_tiep_nhan, ma_y_te, ho_ten, gioi_tinh, ngay_sinh, so_cccd, ngay_vao, status, exam_mode, contract_id, exam_templates, payload_json) VALUES ('TN03', 'BN03', ?, '1', ?, ?, '16/11/2026', 1, 'doan', ?, 'TREN_18', ?)")
            .bind(&ht3).bind(&ns3).bind(&cccd3).bind(split_c1_id.0).bind(&p3)
            .execute(&split_pool).await.unwrap();
        split_pool.close().await;

        // 4. Preview & Merge về main.db
        let preview = preview_merge_contract_db(&main_pool, &split_str).await.unwrap();
        assert_eq!(preview.contracts.len(), 2);
        assert_eq!(preview.total_updated, 2);
        assert_eq!(preview.total_inserted, 1);
        assert!(preview.same_master_key);

        let (upd, ins) = merge_contract_db(&main_pool, &split_str, Some(&mk), false).await.unwrap();
        assert_eq!(upd, 2);
        assert_eq!(ins, 1);

        // 5. Xác minh TN01 giữ cả CLS của BV và Lâm sàng của Laptop
        let (st1, enc_p1): (i64, String) = sqlx::query_as("SELECT status, payload_json FROM ksk_records WHERE ma_tiep_nhan = 'TN01'")
            .fetch_one(&main_pool).await.unwrap();
        assert_eq!(st1, 1);
        let dec1 = crate::crypto::decrypt_field(Some(&mk), &enc_p1);
        let m1: Map<String, Value> = serde_json::from_str(&dec1).unwrap();
        assert_eq!(m1.get("CLS_HUYET_HOC").and_then(|v| v.as_str()), Some("Bình thường"));
        assert_eq!(m1.get("NOI_KHOA_TUAN_HOAN").and_then(|v| v.as_str()), Some("Bình thường"));
        let ex1 = m1.get("_EXAMINED_SECTIONS").and_then(|v| v.as_str()).unwrap_or("");
        assert!(ex1.contains("paraclinical") && ex1.contains("clin_noi"));

        main_pool.close().await;
        let _ = fs::remove_dir_all(&tmp);
    }
}
