use std::process::Command;
use std::path::PathBuf;

fn find_node_modules() -> Option<PathBuf> {
    // Debug: check current dir
    if let Ok(cwd) = std::env::current_dir() {
        let nm = cwd.join("node_modules").join("sm-crypto");
        if nm.exists() { eprintln!("DBG: found node_modules at cwd"); return Some(cwd); }
    }
    // Check exe location
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let nm = dir.join("node_modules").join("sm-crypto");
            if nm.exists() { eprintln!("DBG: found node_modules beside exe"); return Some(dir.to_path_buf()); }
        }
    }
    // Check hardcoded paths
    for p in &[
        "C:/Users/admin/AppData/Local/Temp/opencode",
        "C:/Users/admin/Desktop",
    ] {
        let pb = PathBuf::from(p);
        if pb.join("node_modules").join("sm-crypto").exists() {
            eprintln!("DBG: found node_modules at {}", p);
            return Some(pb);
        }
    }
    eprintln!("DBG: node_modules NOT FOUND!");
    None
}

pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let base = match find_node_modules() {
        Some(b) => b,
        None => {
            eprintln!("ERROR: node_modules/sm-crypto not found!");
            eprintln!("      place node_modules/sm-crypto beside the exe");
            return String::new();
        }
    };
    eprintln!("DBG: base dir = {:?}", base);

    let code = r#"const sm2=require('sm-crypto').sm2;const[p,k]=process.argv.slice(1);process.stdout.write('04'+sm2.doEncrypt(p,k,0));"#;

    let output = Command::new("node")
        .arg("-e")
        .arg(code)
        .arg(plaintext)
        .arg(pubkey_hex)
        .current_dir(&base)
        .output()
        .expect("Failed to execute node.exe. Install Node.js first.");

    eprintln!("DBG: node exit code = {:?}", output.status.code());
    if !output.stderr.is_empty() {
        eprintln!("DBG: node stderr = {}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        eprintln!("ERROR: Node.js encrypt failed");
        return String::new();
    }

    let result = String::from_utf8(output.stdout).expect("Invalid UTF-8 from node").trim().to_string();
    eprintln!("DBG: encrypted len = {}", result.len());
    result
}
