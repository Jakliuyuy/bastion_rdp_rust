use std::process::Command;
use std::path::PathBuf;

fn get_base() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if cfg!(debug_assertions) {
                // In debug mode, look for node_modules in the source dir
                let src = dir.join("..").join("..").join("node_modules");
                if src.exists() { return dir.join("..").join(".."); }
            }
            // Check beside the exe first
            let nm = dir.join("node_modules");
            if nm.exists() { return dir.to_path_buf(); }
        }
    }
    // Fallback: check common locations
    for p in &[
        "C:/Users/admin/AppData/Local/Temp/opencode",
        "C:/Users/admin/Desktop",
    ] {
        let pb = PathBuf::from(p);
        if pb.join("node_modules").exists() { return pb; }
    }
    std::env::current_dir().unwrap_or_default()
}

pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let code = r#"const sm2=require('sm-crypto').sm2;const[p,k]=process.argv.slice(1);process.stdout.write('04'+sm2.doEncrypt(p,k,0));"#;

    let base = get_base();
    let node = if cfg!(windows) { "node.exe" } else { "node" };

    let output = Command::new(node)
        .arg("-e")
        .arg(code)
        .arg(plaintext)
        .arg(pubkey_hex)
        .current_dir(&base)
        .output()
        .expect("Failed to execute node. Make sure Node.js is installed.");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Node.js SM2 encrypt failed: {}", stderr);
    }

    String::from_utf8(output.stdout).expect("Invalid UTF-8 from node").trim().to_string()
}
