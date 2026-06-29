fn main() {
    use std::sync::LazyLock;
    static DUMMY_VERIFY_HASH: LazyLock<String> =
        LazyLock::new(|| password_auth::generate_hash("timing-equalization-dummy-fixture"));

    let dummy = DUMMY_VERIFY_HASH.as_str();
    println!("dummy hash len={} head={:?}", dummy.len(), &dummy[..dummy.len().min(40)]);

    // 1. Confirm the dummy hash is a valid PHC string that verify_password accepts the parse of.
    //    verify against a WRONG password -> must be Err (PasswordInvalid), NOT a ParseError.
    let r_wrong = password_auth::verify_password("anything-else", dummy);
    println!("verify(wrong, dummy) = Err? {}", r_wrong.is_err());

    // 2. Confirm legacy-style real hash still verifies correctly (no regression to lookups/login).
    let real = password_auth::generate_hash("hunter2");
    let r_good = password_auth::verify_password("hunter2", &real);
    let r_bad  = password_auth::verify_password("nope", &real);
    println!("verify(good, real) = Ok? {}", r_good.is_ok());
    println!("verify(bad,  real) = Err? {}", r_bad.is_err());

    // 3. Cost sanity: both wrong-vs-dummy and wrong-vs-real should do real Argon2 work
    //    (no parse-time early exit that would make the dummy cheaper).
    use std::time::Instant;
    let t = Instant::now();
    for _ in 0..3 { let _ = password_auth::verify_password("x", dummy); }
    println!("3x dummy verify: {} ms", t.elapsed().as_millis());
    let t = Instant::now();
    for _ in 0..3 { let _ = password_auth::verify_password("x", &real); }
    println!("3x real  verify: {} ms", t.elapsed().as_millis());
}
