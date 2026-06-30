use libsm::sm2::ecc::EccCtx;
use libsm::sm2::encrypt::EncryptCtx;

pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let pk = if pubkey_hex.len() > 128 { &pubkey_hex[..130] } else { pubkey_hex };
    // pk is "04" + x(64 hex) + y(64 hex) = 130 hex chars = 65 bytes
    let pk_bytes = hex::decode(pk).expect("invalid pubkey hex");

    let ecc = EccCtx::new();
    let point = ecc.bytes_to_point(&pk_bytes).expect("invalid pubkey point");

    let klen = plaintext.len();
    let ctx = EncryptCtx::new(klen, point);
    let cipher = ctx.encrypt(plaintext.as_bytes()).expect("encrypt failed");

    // cipher format: [04||x||y(65 bytes)] [C2(klen)] [C3(32 bytes)]
    // Server expects: 04||x||y||C2||C3 (same format)
    // cipher is Vec<u8>, return as hex
    hex::encode(cipher)
}
