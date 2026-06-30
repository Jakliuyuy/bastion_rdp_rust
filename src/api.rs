use crate::config::Config;
use crate::crypto;
use serde_json::Value;

const BASTION: &str = "https://10.233.83.246";

pub async fn login_and_connect(cfg: &Config) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .build()
        .map_err(|e| e.to_string())?;

    // Init session
    client.get(&format!("{}/ais-ifort/login", BASTION))
        .send().await.map_err(|e| e.to_string())?;

    // Get SM2 key
    let resp: Value = client.post(&format!("{}/ifort/encrypt/getSm2PublicKey", BASTION))
        .json(&serde_json::json!({}))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let pubkey = resp["data"]["publicKey"].as_str().ok_or("no pubkey")?;
    let csrf = resp["data"]["csrfToken"].as_str().ok_or("no csrf")?;
    let ts = resp["data"]["timeStamp"].as_u64().ok_or("no ts")?;

    // Login
    let enc_user = crypto::encrypt(&cfg.user, pubkey);
    let enc_pwd = crypto::encrypt(&cfg.password, pubkey);
    let resp: Value = client.post(&format!("{}/ifort/login/authnStep1", BASTION))
        .header("CSRF-TOKEN", csrf)
        .header("encryptVersion", ts.to_string())
        .json(&serde_json::json!({
            "loginAcct": enc_user, "password": enc_pwd, "captcha": ""
        }))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(resp["desc"].as_str().unwrap_or("login failed").into());
    }

    // Query servers
    let resp: Value = client.post(&format!("{}/ifort/sso/querySsoList", BASTION))
        .json(&serde_json::json!({
            "deptId":"","devType":"","sortColumn":"string","sortDesc":true,
            "userSystem":"Windows","searchContent":"","isPaging":0,"pageNum":1,"pageSize":100
        }))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let srv = resp["data"].as_array().ok_or("no servers")?.iter()
        .find(|x| x["devIp"].as_str() == Some(&cfg.server_ip))
        .ok_or_else(|| format!("server {} not found", cfg.server_ip))?;
    let dev_id = srv["id"].as_u64().ok_or("no dev id")?;

    // Get new SM2 key for devSso
    let resp: Value = client.post(&format!("{}/ifort/encrypt/getSm2PublicKey", BASTION))
        .json(&serde_json::json!({}))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let pubkey2 = resp["data"]["publicKey"].as_str().ok_or("no pubkey")?;
    let csrf2 = resp["data"]["csrfToken"].as_str().ok_or("no csrf")?;
    let ts2 = resp["data"]["timeStamp"].as_u64().ok_or("no ts")?;

    // Query tool
    let resp: Value = client.post(&format!("{}/ifort/ssoParam/querySsoTool", BASTION))
        .header("CSRF-TOKEN", csrf2)
        .header("encryptVersion", ts2.to_string())
        .json(&serde_json::json!({"devId": dev_id, "protocol": "rdp", "userSystem": "Windows"}))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    let tool_id = resp["data"]["defaultToolId"].as_u64().ok_or("no tool")?;

    // devSso
    let su = if cfg.server_user.is_empty() { &cfg.user } else { &cfg.server_user };
    let sp_enc = if cfg.server_pwd.is_empty() { String::new() } else { crypto::encrypt(&cfg.server_pwd, pubkey2) };
    let resp: Value = client.post(&format!("{}/ifort/sso/devSso", BASTION))
        .header("CSRF-TOKEN", csrf2)
        .header("encryptVersion", ts2.to_string())
        .json(&serde_json::json!({
            "devId": dev_id, "protocol": "rdp", "port": "3389",
            "resolution": "0*0", "loginMode": "0", "toolId": tool_id, "toolName": "Mstsc",
            "acctName": su, "acctPwd": sp_enc,
            "devIp": cfg.server_ip, "ipUrl": "10.233.83.246",
            "ssoLoginType": 0, "acctType": "2", "userSystem": "Windows", "toolType": "L"
        }))
        .send().await.map_err(|e| e.to_string())?
        .json().await.map_err(|e| e.to_string())?;
    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(resp["desc"].as_str().unwrap_or("devSso failed").into());
    }
    Ok(resp["data"]["url"].as_str().ok_or("no url")?.to_string())
}
