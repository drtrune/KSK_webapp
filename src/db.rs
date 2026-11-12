use crate::models::DbInfo;
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    SqlitePool,
};
use std::fs;
use std::str::FromStr;

const SEED_CATALOGS_TSV: &str = include_str!("../asset/seed_catalogs.tsv");

pub fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hex::encode(hasher.finalize())
}

pub async fn init_db(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let conn_str = format!("sqlite:{}", db_path);
    let options = SqliteConnectOptions::from_str(&conn_str)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal);

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
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
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
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
            payload_json TEXT NOT NULL DEFAULT '{}',
            updated_by TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );
        "#,
    )
    .execute(&pool)
    .await?;

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

    // Seed danh mục chuẩn từ SEED_CATALOGS_TSV nếu bảng catalogs đang trống
    let (catalog_count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM catalogs")
        .fetch_one(&pool)
        .await?;
    if catalog_count == 0 {
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

    // Lưu lại các thiết lập mặc định / tạm ngưng hiện có để khôi phục sau khi nạp lại từ asset
    let preserved_flags: Vec<(String, String, String, i64, i64)> = sqlx::query_as(
        "SELECT cat_type, code, parent_code, is_active, is_default FROM catalogs WHERE is_default = 1 OR is_active = 0",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

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

    // Khôi phục lại lựa chọn mặc định và trạng thái tạm ngưng của người dùng
    for (cat_type, code, parent_code, is_active, is_default) in preserved_flags {
        let _ = sqlx::query(
            "UPDATE catalogs SET is_active = ?, is_default = ? WHERE cat_type = ? AND code = ? AND parent_code = ?",
        )
        .bind(is_active)
        .bind(is_default)
        .bind(cat_type)
        .bind(code)
        .bind(parent_code)
        .execute(&mut *tx)
        .await;
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

    Ok(count)
}

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
