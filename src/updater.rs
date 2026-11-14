use crate::models::UpdateManifest;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Dọn dẹp file .exe.old sau khi tự cập nhật và khởi động lại
pub fn cleanup_old_exe_on_startup() {
    if let Ok(exe_path) = std::env::current_exe() {
        let old_path = PathBuf::from(format!("{}.old", exe_path.display()));
        if old_path.exists() {
            let _ = fs::remove_file(&old_path);
        }
        let new_path = PathBuf::from(format!("{}.new", exe_path.display()));
        if new_path.exists() {
            let _ = fs::remove_file(&new_path);
        }
    }
}

pub fn parse_semver(v: &str) -> (u32, u32, u32) {
    let clean = v.trim().trim_start_matches('v').trim_start_matches('V');
    let mut parts = clean.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let patch = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor, patch)
}

pub fn is_newer_version(remote: &str, current: &str) -> bool {
    let r = parse_semver(remote);
    let c = parse_semver(current);
    r.cmp(&c) == Ordering::Greater
}

/// Kiểm tra phiên bản mới từ thư mục Update trên ổ đĩa chung (ví dụ: \\Server\Share\KSK\Update)
pub fn check_for_update(update_dir: &str) -> Result<Option<UpdateManifest>, String> {
    let dir = update_dir.trim();
    if dir.is_empty() {
        return Ok(None);
    }

    let manifest_path = Path::new(dir).join("version.json");
    if !manifest_path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Không thể đọc {}: {}", manifest_path.display(), e))?;
    let manifest: UpdateManifest = serde_json::from_str(&raw)
        .map_err(|e| format!("Lỗi định dạng version.json: {}", e))?;

    if is_newer_version(&manifest.version, CURRENT_VERSION) {
        Ok(Some(manifest))
    } else {
        Ok(None)
    }
}

/// Tải file .exe mới từ ổ đĩa chung, đổi tên file đang chạy -> .old, thay thế và khởi động lại
pub fn apply_update_and_restart(update_dir: &str, manifest: &UpdateManifest) -> Result<(), String> {
    let dir = update_dir.trim();
    if dir.is_empty() {
        return Err("Chưa cấu hình thư mục cập nhật (update_dir_path)".to_string());
    }

    let source_exe = Path::new(dir).join(&manifest.exe_name);
    if !source_exe.exists() {
        return Err(format!(
            "Không tìm thấy file cập nhật: {}",
            source_exe.display()
        ));
    }

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Không xác định được đường dẫn ứng dụng: {}", e))?;
    let new_exe = PathBuf::from(format!("{}.new", current_exe.display()));
    let old_exe = PathBuf::from(format!("{}.old", current_exe.display()));

    // 1. Copy bản mới về máy trạm trước để tránh lỗi mạng giữa chừng
    fs::copy(&source_exe, &new_exe)
        .map_err(|e| format!("Không thể tải file cập nhật về máy trạm: {}", e))?;

    // 2. Xóa .old cũ nếu còn sót
    if old_exe.exists() {
        let _ = fs::remove_file(&old_exe);
    }

    // 3. Đổi tên file đang chạy -> .old (Windows cho phép rename file exe đang chạy)
    fs::rename(&current_exe, &old_exe)
        .map_err(|e| format!("Không thể đổi tên file thực thi hiện tại: {}", e))?;

    // 4. Đưa file .new vào vị trí chính thức
    if let Err(e) = fs::rename(&new_exe, &current_exe) {
        // Rollback nếu rename thất bại
        let _ = fs::rename(&old_exe, &current_exe);
        return Err(format!("Không thể thay thế file thực thi mới: {}", e));
    }

    // 5. Khởi chạy phiên bản mới và thoát tiến trình cũ
    std::process::Command::new(&current_exe)
        .spawn()
        .map_err(|e| format!("Đã cập nhật nhưng không thể tự khởi động lại: {}", e))?;

    std::process::exit(0);
}

/// Dành cho Admin: Phát hành phiên bản hiện tại lên thư mục Update trên ổ đĩa chung
pub fn publish_update_to_share(update_dir: &str, changelog: &str) -> Result<UpdateManifest, String> {
    let dir = update_dir.trim();
    if dir.is_empty() {
        return Err("Vui lòng chọn hoặc nhập thư mục Update trên ổ đĩa chung".to_string());
    }

    let target_dir = Path::new(dir);
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Không thể tạo thư mục {}: {}", target_dir.display(), e))?;

    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Không xác định được file .exe hiện tại: {}", e))?;
    let exe_name = current_exe
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("ksk_webapp.exe")
        .to_string();

    let target_exe = target_dir.join(&exe_name);
    fs::copy(&current_exe, &target_exe)
        .map_err(|e| format!("Lỗi copy .exe lên ổ đĩa chung: {}", e))?;

    let manifest = UpdateManifest {
        version: CURRENT_VERSION.to_string(),
        exe_name,
        changelog: changelog.trim().to_string(),
        updated_at: chrono::Local::now().format("%d/%m/%Y %H:%M:%S").to_string(),
    };

    let manifest_path = target_dir.join("version.json");
    let json_str = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, json_str)
        .map_err(|e| format!("Lỗi ghi version.json: {}", e))?;

    Ok(manifest)
}

