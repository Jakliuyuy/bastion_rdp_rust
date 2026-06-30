use crate::config::Config;
use crate::crypto;
use serde_json::Value;

const BASTION: &str = "https://10.233.83.246";

pub async fn login_and_connect(cfg: &Config) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let mut jsessionid = String::new();
    let mut csrf_token = String::new();
    let mut encrypt_ver = String::new();
    let mut pubkey = String::new();

    // Helper to make requests with session cookie
    macro_rules! post {
        ($path:expr, $body:expr) => {{
            let mut r = client.post(&format!("{}{}", BASTION, $path)).json(&$body);
            if !jsessionid.is_empty() {
                r = r.header("Cookie", &jsessionid);
            }
            if !csrf_token.is_empty() {
                r = r.header("CSRF-TOKEN", &csrf_token);
            }
            if !encrypt_ver.is_empty() {
                r = r.header("encryptVersion", &encrypt_ver);
            }
            let resp = r.send().await.map_err(|e| e.to_string())?;
            // Extract Set-Cookie
            if let Some(sc) = resp.headers().get("set-cookie").and_then(|v| v.to_str().ok()) {
                if let Some(sid) = sc.split(';').next() {
                    if jsessionid.is_empty() {
                        jsessionid = sid.to_string();
                    }
                }
            }
            let val: Value = resp.json().await.map_err(|e| e.to_string())?;
            val
        }};
    }

    // Init session
    client.get(&format!("{}/ais-ifort/login", BASTION)).send().await.map_err(|e| e.to_string())?;

    // Get SM2 key
    let resp = post!("/ifort/encrypt/getSm2PublicKey", serde_json::json!({}));
    pubkey = resp["data"]["publicKey"].as_str().ok_or("no pubkey")?.to_string();
    csrf_token = resp["data"]["csrfToken"].as_str().ok_or("no csrf")?.to_string();
    encrypt_ver = resp["data"]["timeStamp"].as_u64().ok_or("no ts")?.to_string();

    // Login
    let enc_user = crypto::encrypt(&cfg.user, &pubkey);
    let enc_pwd = crypto::encrypt(&cfg.password, &pubkey);
    let resp = post!("/ifort/login/authnStep1", serde_json::json!({
        "loginAcct": enc_user, "password": enc_pwd, "captcha": ""
    }));
    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(resp["desc"].as_str().unwrap_or("login failed").into());
    }

    // Query servers
    let resp = post!("/ifort/sso/querySsoList", serde_json::json!({
        "deptId":"","devType":"","sortColumn":"string","sortDesc":true,
        "userSystem":"Windows","searchContent":"","isPaging":0,"pageNum":1,"pageSize":100
    }));
    let srv = resp["data"].as_array().ok_or("no servers")?.iter()
        .find(|x| x["devIp"].as_str() == Some(&cfg.server_ip))
        .ok_or_else(|| format!("server {} not found", cfg.server_ip))?;
    let dev_id = srv["id"].as_u64().ok_or("no dev id")?;

    // Get new SM2 key
    let resp = post!("/ifort/encrypt/getSm2PublicKey", serde_json::json!({}));
    pubkey = resp["data"]["publicKey"].as_str().ok_or("no pubkey")?.to_string();
    csrf_token = resp["data"]["csrfToken"].as_str().ok_or("no csrf")?.to_string();
    encrypt_ver = resp["data"]["timeStamp"].as_u64().ok_or("no ts")?.to_string();

    // Query tool
    let resp = post!("/ifort/ssoParam/querySsoTool", serde_json::json!({
        "devId": dev_id, "protocol": "rdp", "userSystem": "Windows"
    }));
    let tool_id = resp["data"]["defaultToolId"].as_u64().ok_or("no tool")?;

    // devSso
    let su = if cfg.server_user.is_empty() { &cfg.user } else { &cfg.server_user };
    let sp_enc = if cfg.server_pwd.is_empty() { String::new() } else { crypto::encrypt(&cfg.server_pwd, &pubkey) };
    let resp = post!("/ifort/sso/devSso", serde_json::json!({
        "devId": dev_id, "protocol": "rdp", "port": "3389",
        "resolution": "0*0", "loginMode": "0", "toolId": tool_id, "toolName": "Mstsc",
        "acctName": su, "acctPwd": sp_enc,
        "devIp": cfg.server_ip, "ipUrl": "10.233.83.246",
        "ssoLoginType": 0, "acctType": "2", "userSystem": "Windows", "toolType": "L"
    }));
    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(resp["desc"].as_str().unwrap_or("devSso failed").into());
    }
    Ok(resp["data"]["url"].as_str().ok_or("no url")?.to_string())
}
