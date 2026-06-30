use crate::config::Config;
use crate::crypto;
use serde_json::Value;

const BASTION: &str = "https://10.233.83.246";

pub async fn login_and_connect(cfg: &Config) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| e.to_string())?;

    let mut cookie = String::new();

    async fn post(
        client: &reqwest::Client,
        path: &str,
        body: Value,
        cookie: &mut String,
        csrf: &str,
        enc_ver: &str,
    ) -> Result<Value, String> {
        let url = format!("{}{}", BASTION, path);
        let mut r = client.post(&url).header("Content-Type", "application/json;charset=UTF-8");
        if !cookie.is_empty() { r = r.header("Cookie", cookie.as_str()); }
        if !csrf.is_empty() { r = r.header("CSRF-TOKEN", csrf); }
        if !enc_ver.is_empty() { r = r.header("encryptVersion", enc_ver); }

        let resp = r.json(&body).send().await.map_err(|e| e.to_string())?;

        for h in resp.headers().get_all("set-cookie") {
            if let Some(s) = h.to_str().ok().and_then(|v| v.split(';').next()) {
                // Merge cookies: replace existing or append
                let parts: Vec<&str> = cookie.split("; ").filter(|c| !c.is_empty()).collect();
                let name = s.split('=').next().unwrap_or("");
                let mut new_cookie = String::new();
                let mut replaced = false;
                for p in &parts {
                    let pname = p.split('=').next().unwrap_or("");
                    if pname == name { new_cookie.push_str(s); replaced = true; }
                    else { if !new_cookie.is_empty() { new_cookie.push_str("; "); } new_cookie.push_str(p); }
                }
                if !replaced {
                    if !new_cookie.is_empty() { new_cookie.push_str("; "); }
                    new_cookie.push_str(s);
                }
                *cookie = new_cookie;
            }
        }
        resp.json().await.map_err(|e| e.to_string())
    }

    // Init session
    client.get(&format!("{}/ais-ifort/login", BASTION)).send().await.map_err(|e| e.to_string())?;

    // 1. Get SM2 public key
    let resp = post(&client, "/ifort/encrypt/getSm2PublicKey", serde_json::json!({}), &mut cookie, "", "").await?;
    let pubkey = resp["data"]["publicKey"].as_str().ok_or("no pubkey")?.to_string();
    let csrf = resp["data"]["csrfToken"].as_str().ok_or("no csrf")?.to_string();
    let ts = resp["data"]["timeStamp"].as_u64().ok_or("no ts")?.to_string();

    eprintln!("DBG: pubkey={}...", &pubkey[..40]);
    eprintln!("DBG: cookie={}", cookie);

    // 2. Login
    let enc_user = crypto::encrypt(&cfg.user, &pubkey);
    let enc_pwd = crypto::encrypt(&cfg.password, &pubkey);
    let resp = post(&client, "/ifort/login/authnStep1", serde_json::json!({
        "loginAcct": enc_user, "password": enc_pwd, "captcha": ""
    }), &mut cookie, &csrf, &ts).await?;
    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(format!("登录失败: {}", resp["desc"].as_str().unwrap_or("unknown")));
    }
    eprintln!("DBG: login ok, cookie={}", cookie);

    // 3. Query servers
    let resp = post(&client, "/ifort/sso/querySsoList", serde_json::json!({
        "deptId":"","devType":"","sortColumn":"string","sortDesc":true,
        "userSystem":"Windows","searchContent":"","isPaging":0,"pageNum":1,"pageSize":100
    }), &mut cookie, "", "").await?;
    let code = resp["code"].as_str().unwrap_or("");
    if code != "0" {
        return Err(format!("查询服务器失败: code={}, desc={}", code, resp["desc"].as_str().unwrap_or("?")));
    }
    let arr = resp["data"].as_array().ok_or_else(|| format!("响应: {}", resp))?;
    eprintln!("DBG: found {} servers", arr.len());
    let srv = arr.iter()
        .find(|x| x["devIp"].as_str() == Some(&cfg.server_ip))
        .ok_or_else(|| format!("未找到服务器 {}，共{}台", cfg.server_ip, arr.len()))?;
    let dev_id = srv["id"].as_u64().ok_or("no dev id")?;

    // 4. Get new SM2 key for devSso
    let resp = post(&client, "/ifort/encrypt/getSm2PublicKey", serde_json::json!({}), &mut cookie, "", "").await?;
    let pubkey2 = resp["data"]["publicKey"].as_str().ok_or("no pubkey2")?.to_string();
    let csrf2 = resp["data"]["csrfToken"].as_str().ok_or("no csrf2")?.to_string();
    let ts2 = resp["data"]["timeStamp"].as_u64().ok_or("no ts2")?.to_string();

    // 5. Query tool
    let resp = post(&client, "/ifort/ssoParam/querySsoTool", serde_json::json!({
        "devId": dev_id, "protocol": "rdp", "userSystem": "Windows"
    }), &mut cookie, &csrf2, &ts2).await?;
    let tool_id = resp["data"]["defaultToolId"].as_u64().ok_or("no tool")?;

    // 6. devSso
    let su = if cfg.server_user.is_empty() { &cfg.user } else { &cfg.server_user };
    let sp_enc = if cfg.server_pwd.is_empty() { String::new() } else { crypto::encrypt(&cfg.server_pwd, &pubkey2) };
    let resp = post(&client, "/ifort/sso/devSso", serde_json::json!({
        "devId": dev_id, "protocol": "rdp", "port": "3389",
        "resolution": "0*0", "loginMode": "0", "toolId": tool_id, "toolName": "Mstsc",
        "acctName": su, "acctPwd": sp_enc,
        "devIp": cfg.server_ip, "ipUrl": "10.233.83.246",
        "ssoLoginType": 0, "acctType": "2", "userSystem": "Windows", "toolType": "L"
    }), &mut cookie, &csrf2, &ts2).await?;
    if !resp["success"].as_bool().unwrap_or(false) {
        return Err(format!("获取令牌失败: {}", resp["desc"].as_str().unwrap_or("?")));
    }
    Ok(resp["data"]["url"].as_str().ok_or("no url")?.to_string())
}
