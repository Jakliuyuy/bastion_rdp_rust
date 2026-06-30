use std::process::Command;
use std::path::PathBuf;

fn find_node_modules() -> Option<PathBuf> {
    // Check exe location
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            if dir.join("node_modules").join("sm-crypto").exists() { return Some(dir.to_path_buf()); }
        }
    }
    // Check hardcoded paths
    for p in &["C:/Users/admin/AppData/Local/Temp/opencode", "C:/Users/admin/Desktop"] {
        let pb = PathBuf::from(p);
        if pb.join("node_modules").join("sm-crypto").exists() { return Some(pb); }
    }
    None
}

pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let base = find_node_modules()
        .expect("node_modules/sm-crypto not found. Place it beside the exe.");

    let code = r#"const sm2=require('sm-crypto').sm2;const[p,k]=process.argv.slice(1);process.stdout.write('04'+sm2.doEncrypt(p,k,0));"#;

    let output = Command::new("node")
        .arg("-e").arg(code).arg(plaintext).arg(pubkey_hex)
        .current_dir(&base)
        .output()
        .expect("Failed to execute node.exe");

    let result = String::from_utf8(output.stdout).expect("Invalid UTF-8 from node").trim().to_string();
    if result.is_empty() {
        panic!("Node.js encrypt returned empty output");
    }
    result
}
