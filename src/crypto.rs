use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use sha2::Sha256;

const ENC_PREFIX: &str = "ENC1:";
const MK_PREFIX: &str = "MK1:";
const PBKDF2_ITERS: u32 = 100_000;

pub fn generate_master_key() -> [u8; 32] {
    let mut mk = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut mk);
    mk
}

/// Tạo Recovery Key dạng `KSK-XXXX-XXXX-XXXX-XXXX-XXXX-XXXX` (24 ký tự).
pub fn generate_recovery_key() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut raw = [0u8; 24];
    rand::rngs::OsRng.fill_bytes(&mut raw);
    let chars: Vec<char> = raw
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();
    let groups: Vec<String> = chars.chunks(4).map(|c| c.iter().collect()).collect();
    format!("KSK-{}", groups.join("-"))
}

fn derive_kek(secret: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut kek = [0u8; 32];
    let _ = pbkdf2::<Hmac<Sha256>>(secret.trim().as_bytes(), salt, PBKDF2_ITERS, &mut kek);
    kek
}

pub fn wrap_master_key(mk: &[u8; 32], secret: &str) -> Result<String, String> {
    let mut salt = [0u8; 16];
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);

    let kek = derive_kek(secret, &salt);
    let cipher = Aes256Gcm::new_from_slice(&kek).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, mk.as_slice())
        .map_err(|e| format!("Lỗi bọc Master Key: {}", e))?;

    let mut combined = Vec::with_capacity(16 + 12 + ct.len());
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ct);
    Ok(format!("{}{}", MK_PREFIX, B64.encode(combined)))
}

pub fn unwrap_master_key(wrapped: &str, secret: &str) -> Result<[u8; 32], String> {
    let payload = wrapped
        .trim()
        .strip_prefix(MK_PREFIX)
        .ok_or_else(|| "Định dạng Master Key không hợp lệ".to_string())?;
    let bytes = B64
        .decode(payload)
        .map_err(|e| format!("Lỗi giải mã base64 MK: {}", e))?;
    if bytes.len() < 16 + 12 + 16 {
        return Err("Dữ liệu Master Key bị thiếu hoặc hỏng".into());
    }
    let salt: [u8; 16] = bytes[0..16].try_into().unwrap();
    let nonce_bytes: [u8; 12] = bytes[16..28].try_into().unwrap();
    let ct = &bytes[28..];

    let kek = derive_kek(secret, &salt);
    let cipher = Aes256Gcm::new_from_slice(&kek).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let pt = cipher
        .decrypt(nonce, ct)
        .map_err(|_| "Mật khẩu hoặc Recovery Key không đúng".to_string())?;
    if pt.len() != 32 {
        return Err("Độ dài Master Key không hợp lệ".into());
    }
    let mut mk = [0u8; 32];
    mk.copy_from_slice(&pt);
    Ok(mk)
}

pub fn encrypt_field(mk: &[u8; 32], plaintext: &str) -> String {
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let Ok(cipher) = Aes256Gcm::new_from_slice(mk) else {
        return plaintext.to_string();
    };
    let nonce = Nonce::from_slice(&nonce_bytes);
    match cipher.encrypt(nonce, plaintext.as_bytes()) {
        Ok(ct) => {
            let mut combined = Vec::with_capacity(12 + ct.len());
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&ct);
            format!("{}{}", ENC_PREFIX, B64.encode(combined))
        }
        Err(_) => plaintext.to_string(),
    }
}

pub fn decrypt_field(mk: Option<&[u8; 32]>, value: &str) -> String {
    let Some(payload) = value.strip_prefix(ENC_PREFIX) else {
        return value.to_string();
    };
    let Some(key) = mk else {
        return "[Đã mã hóa]".to_string();
    };
    let Ok(bytes) = B64.decode(payload) else {
        return "[Lỗi giải mã]".to_string();
    };
    if bytes.len() < 12 + 16 {
        return "[Lỗi giải mã]".to_string();
    }
    let nonce = Nonce::from_slice(&bytes[0..12]);
    let ct = &bytes[12..];
    let Ok(cipher) = Aes256Gcm::new_from_slice(key) else {
        return "[Lỗi giải mã]".to_string();
    };
    match cipher.decrypt(nonce, ct) {
        Ok(pt) => String::from_utf8(pt).unwrap_or_else(|_| "[Lỗi UTF-8]".to_string()),
        Err(_) => "[Sai Master Key]".to_string(),
    }
}

pub fn encrypt_record_fields(
    mk: Option<&[u8; 32]>,
    ho_ten: &str,
    ngay_sinh: &str,
    so_cccd: &str,
    payload_json: &str,
) -> (String, String, String, String) {
    match mk {
        Some(k) => (
            encrypt_field(k, ho_ten),
            encrypt_field(k, ngay_sinh),
            encrypt_field(k, so_cccd),
            encrypt_field(k, payload_json),
        ),
        None => (
            ho_ten.to_string(),
            ngay_sinh.to_string(),
            so_cccd.to_string(),
            payload_json.to_string(),
        ),
    }
}

pub fn decrypt_record_row(mk: Option<&[u8; 32]>, row: &mut crate::models::KskRecordRow) {
    row.ho_ten = decrypt_field(mk, &row.ho_ten);
    row.ngay_sinh = decrypt_field(mk, &row.ngay_sinh);
    row.so_cccd = decrypt_field(mk, &row.so_cccd);
    row.payload_json = decrypt_field(mk, &row.payload_json);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_unwrap_and_field_encryption() {
        let mk = generate_master_key();
        let rk = generate_recovery_key();
        assert!(rk.starts_with("KSK-"));

        let wrapped_pw = wrap_master_key(&mk, "AdminPass!123").unwrap();
        let wrapped_rk = wrap_master_key(&mk, &rk).unwrap();

        assert_eq!(unwrap_master_key(&wrapped_pw, "AdminPass!123").unwrap(), mk);
        assert_eq!(unwrap_master_key(&wrapped_rk, &rk).unwrap(), mk);
        assert!(unwrap_master_key(&wrapped_pw, "WrongPass").is_err());

        let enc = encrypt_field(&mk, "NGUYỄN VĂN A");
        assert!(enc.starts_with(ENC_PREFIX));
        assert_ne!(enc, "NGUYỄN VĂN A");
        assert_eq!(decrypt_field(Some(&mk), &enc), "NGUYỄN VĂN A");
        assert_eq!(decrypt_field(None, &enc), "[Đã mã hóa]");
        assert_eq!(decrypt_field(Some(&mk), "legacy_plain"), "legacy_plain");
    }
}
