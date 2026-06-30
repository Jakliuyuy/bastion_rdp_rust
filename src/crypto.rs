use libsm::sm2::ecc::EccCtx;
use libsm::sm2::encrypt::EncryptCtx;

pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let pk = if pubkey_hex.len() > 128 { &pubkey_hex[..130] } else { pubkey_hex };
    let pk_bytes = hex::decode(pk).expect("invalid pubkey hex");

    let ecc = EccCtx::new();
    let point = ecc.bytes_to_point(&pk_bytes).expect("invalid pubkey point");

    let ctx = EncryptCtx::new(plaintext.len(), point);
    let cipher = ctx.encrypt(plaintext.as_bytes()).expect("encrypt failed");
    hex::encode(cipher)
}
