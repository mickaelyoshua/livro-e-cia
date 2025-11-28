use argon2::{
    password_hash, Algorithm, Argon2, ParamsBuilder, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use password_hash::{rand_core::OsRng, SaltString};

pub fn hash_password(password: &str) -> Result<String, password_hash::Error> {
    let params = ParamsBuilder::new()
        .m_cost(19456) // 19MiB - Makes attackers use more RAM per hash, make attacks expensive
        .t_cost(2) // 2 iterations - More iterations = slower hashing = harder to crack
        .p_cost(1) // 1 thread - high parallelism could cause resource contention
        .build()?;
    // Those parameters match OWASP recommendations
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    // Argon2id combine Argon2d (resistant to GPU attacks) with Argon2i (resistant to sode-channel
    // attacks)
    let salt = SaltString::generate(&mut OsRng); // OsRng uses the OS's cryptographically secure
                                                 // randon number generator
                                                 // Salt will ensure that same passwords will make different hashs
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    // PHC string format - params stored in the hash

    Ok(password_hash)
}

pub fn verify_password(password: &str, hashed: &str) -> Result<bool, password_hash::Error> {
    let parsed_hash = PasswordHash::new(hashed)?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}
