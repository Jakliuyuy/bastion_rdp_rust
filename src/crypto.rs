use libsm::sm2::encrypt as sm2_enc;

/// SM2 encrypt, returns hex string matching browser format: "04" + x||y + C2 + C3
pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let pk = if pubkey_hex.len() > 128 { &pubkey_hex[..130] } else { pubkey_hex };
    let raw = sm2_enc::encrypt(plaintext.as_bytes(), pk);
    // raw is hex string: 04 + 04||x||y + C2 + C3
    // Convert to: 04 + x||y + C2 + C3
    if raw.starts_with("04") && raw.len() > 132 {
        // Standard format: first "04" is overall prefix, next "04" is C1 point prefix
        let c1_x_y = &raw[4..132];  // skip "0404", get x||y (128 hex)
        let rest = &raw[132..];     // C2 + C3
        format!("04{}{}", c1_x_y, rest)
    } else {
        // Already in our format or unknown format
        raw
    }
}
