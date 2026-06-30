use num_bigint::BigUint;
use rand::Rng;
use sm3::{Digest, Sm3};

struct Sm2Curve;
impl Sm2Curve {
    fn p() -> BigUint { BigUint::parse_bytes(b"FFFFFFFEFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000FFFFFFFFFFFFFFFF", 16).unwrap() }
    fn a() -> BigUint { BigUint::parse_bytes(b"FFFFFFFEFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF00000000FFFFFFFFFFFFFFFC", 16).unwrap() }
    fn gx() -> BigUint { BigUint::parse_bytes(b"32C4AE2C1F1981195F9904466A39C9948FE30BBFF2660BE1715A4589334C74C7", 16).unwrap() }
    fn gy() -> BigUint { BigUint::parse_bytes(b"BC3736A2F4F6779C59BDCEE36B692153D0A9877CC62A474002DF32E52139F0A0", 16).unwrap() }
    fn n() -> BigUint { BigUint::parse_bytes(b"FFFFFFFEFFFFFFFFFFFFFFFFFFFFFFFF7203DF6B21C6052B53BBF40939D54123", 16).unwrap() }
}

fn mod_inv(a: &BigUint, p: &BigUint) -> BigUint {
    a.modpow(&(p - 2u64), p)
}

fn ec_add(p1: (&BigUint, &BigUint), p2: (&BigUint, &BigUint)) -> (BigUint, BigUint) {
    let p = Sm2Curve::p();
    if *p1.0 == p { return (p2.0.clone(), p2.1.clone()); }
    if *p2.0 == p { return (p1.0.clone(), p1.1.clone()); }
    let dx = (&p2.0 + &p - p1.0) % &p;
    let dy = (&p2.1 + &p - p1.1) % &p;
    let lam = (&dy * mod_inv(&dx, &p)) % &p;
    let x3 = (&lam * &lam + &p - p1.0 - p2.0) % &p;
    let y3 = (&lam * (p1.0 + &p - &x3) + &p - p1.1) % &p;
    (x3, y3)
}

fn ec_double(p: (&BigUint, &BigUint)) -> (BigUint, BigUint) {
    let p_field = Sm2Curve::p();
    let a = Sm2Curve::a();
    let dy = (&a + 3u64 * &p.0 * &p.0) % &p_field;
    let dx = (2u64 * &p.1) % &p_field;
    let lam = (&dy * mod_inv(&dx, &p_field)) % &p_field;
    let x3 = (&lam * &lam + &p_field - &p.0 - &p.0) % &p_field;
    let y3 = (&lam * (&p.0 + &p_field - &x3) + &p_field - &p.1) % &p_field;
    (x3, y3)
}

fn ec_mul(k: &BigUint, gx: &BigUint, gy: &BigUint) -> (BigUint, BigUint) {
    let p = Sm2Curve::p();
    let mut r = (p.clone(), p.clone());
    let mut a = (gx.clone(), gy.clone());
    let mut bits = k.clone();
    while bits > BigUint::from(0u64) {
        if &bits & BigUint::from(1u64) != BigUint::from(0u64) {
            r = if r.0 == p { a.clone() } else { ec_add((&r.0, &r.1), (&a.0, &a.1)) };
        }
        a = ec_double((&a.0, &a.1));
        bits >>= 1;
    }
    r
}

fn sm3(data: &[u8]) -> Vec<u8> {
    let mut h = Sm3::new();
    h.update(data);
    h.finalize().to_vec()
}

/// SM2 encrypt, returns "04" + C1x||C1y + C2 + C3 in hex
pub fn encrypt(plaintext: &str, pubkey_hex: &str) -> String {
    let pk = if pubkey_hex.len() > 128 { &pubkey_hex[pubkey_hex.len()-128..] } else { pubkey_hex };
    let px = BigUint::parse_bytes(pk[..64].as_bytes(), 16).unwrap();
    let py = BigUint::parse_bytes(pk[64..].as_bytes(), 16).unwrap();
    let n = Sm2Curve::n();

    let mut rng = rand::thread_rng();
    let k_bytes: [u8; 32] = rng.gen();
    let k = BigUint::from_bytes_be(&k_bytes) % &n;
    if k == BigUint::from(0u64) { panic!("zero k"); }

    let (c1x, c1y) = ec_mul(&k, &Sm2Curve::gx(), &Sm2Curve::gy());
    let (x2, y2) = ec_mul(&k, &px, &py);

    let pt = plaintext.as_bytes();
    let x2b = pad32(&x2.to_bytes_be());
    let y2b = pad32(&y2.to_bytes_be());

    // KDF: SM3(x||y||ct=1), take first pt.len() bytes
    let mut kdf_in = x2b.clone();
    kdf_in.extend_from_slice(&y2b);
    kdf_in.extend_from_slice(&[0, 0, 0, 1]);
    let kdf_out = sm3(&kdf_in);

    let c2: Vec<u8> = pt.iter().zip(kdf_out.iter()).map(|(a, b)| a ^ b).collect();

    // C3 = SM3(x||M||y)
    let mut c3_in = x2b.clone();
    c3_in.extend_from_slice(pt);
    c3_in.extend_from_slice(&y2b);
    let c3 = sm3(&c3_in);

    format!("04{:064x}{:064x}{}{}", c1x, c1y, hex::encode(&c2), hex::encode(&c3))
}

fn pad32(b: &[u8]) -> Vec<u8> {
    let mut v = vec![0u8; 32 - b.len()];
    v.extend_from_slice(b);
    v
}
