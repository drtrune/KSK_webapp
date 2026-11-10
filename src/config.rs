use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_database_path")]
    pub database_path: String,
    #[serde(default = "default_server_host")]
    pub server_host: String,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_ma_cskcb")]
    pub ma_cskcb: String,
    #[serde(default = "default_ma_gtin_cskcb")]
    pub ma_gtin_cskcb: String,
    #[serde(default = "default_ten_cskcb")]
    pub ten_cskcb: String,
    #[serde(default = "default_dia_danh")]
    pub dia_danh: String,
    #[serde(default = "default_his_use_mock")]
    pub his_use_mock: bool,
    #[serde(default = "default_his_api_url")]
    pub his_api_url: String,
    #[serde(default = "default_his_http_method")]
    pub his_http_method: String,
    #[serde(default = "default_his_auth_header")]
    pub his_auth_header: String,
    #[serde(default = "default_his_api_key")]
    pub his_api_key: String,
    #[serde(default = "default_his_payload_template")]
    pub his_payload_template: String,
    #[serde(default = "default_his_timeout_seconds")]
    pub his_timeout_seconds: u64,
    #[serde(default = "default_template_docx_path")]
    pub template_docx_path: String,
}

fn default_database_path() -> String {
    "ksk_data.db".to_string()
}
fn default_server_host() -> String {
    "0.0.0.0".to_string()
}
fn default_server_port() -> u16 {
    8080
}
fn default_ma_cskcb() -> String {
    "52208".to_string()
}
fn default_ma_gtin_cskcb() -> String {
    "8930000522080".to_string()
}
fn default_ten_cskcb() -> String {
    "BỆNH VIỆN BÌNH ĐỊNH".to_string()
}
fn default_dia_danh() -> String {
    "Gia Lai".to_string()
}
fn default_his_use_mock() -> bool {
    true
}
fn default_his_api_url() -> String {
    "http://127.0.0.1:8080/api/his/mock".to_string()
}
fn default_his_http_method() -> String {
    "POST".to_string()
}
fn default_his_auth_header() -> String {
    "x-api-key".to_string()
}
fn default_his_api_key() -> String {
    "HIS_SECRET_API_KEY_2026".to_string()
}
fn default_his_payload_template() -> String {
    "{\"ma_tiep_nhan\": \"{{ma_tn}}\"}".to_string()
}
fn default_his_timeout_seconds() -> u64 {
    10
}
fn default_template_docx_path() -> String {
    "asset/template18.docx".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_path: default_database_path(),
            server_host: default_server_host(),
            server_port: default_server_port(),
            ma_cskcb: default_ma_cskcb(),
            ma_gtin_cskcb: default_ma_gtin_cskcb(),
            ten_cskcb: default_ten_cskcb(),
            dia_danh: default_dia_danh(),
            his_use_mock: default_his_use_mock(),
            his_api_url: default_his_api_url(),
            his_http_method: default_his_http_method(),
            his_auth_header: default_his_auth_header(),
            his_api_key: default_his_api_key(),
            his_payload_template: default_his_payload_template(),
            his_timeout_seconds: default_his_timeout_seconds(),
            template_docx_path: default_template_docx_path(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = Path::new("config.toml");
        if config_path.exists() {
            match fs::read_to_string(config_path) {
                Ok(content) => toml::from_str(&content).unwrap_or_default(),
                Err(_) => AppConfig::default(),
            }
        } else {
            let default_cfg = AppConfig::default();
            let _ = default_cfg.save();
            default_cfg
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let toml_str = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write("config.toml", toml_str).map_err(|e| e.to_string())
    }
}
