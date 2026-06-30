fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winres::WindowsResource::new();
        // Only set icon if file exists, otherwise skip
        if std::path::Path::new("icon.ico").exists() {
            res.set_icon("icon.ico");
        }
        res.compile().ok();
    }
}
