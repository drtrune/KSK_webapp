fn main() {
    #[cfg(target_os = "windows")]
    {
        if std::path::Path::new("asset/app.ico").exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon("asset/app.ico");
            let _ = res.compile();
        }
    }
    println!("cargo:rerun-if-changed=asset/app.ico");
}
