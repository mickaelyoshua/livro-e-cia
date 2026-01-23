use argon2::{
    Algorithm, Argon2, ParamsBuilder, PasswordHash, PasswordHasher, PasswordVerifier, Version,
    password_hash::{SaltString, rand_core::OsRng},
};
use tracing::{debug, instrument, warn};

use crate::error::AppError;

// instrument automatically logs function entries with parameters
#[instrument(skip(password))]
pub fn hash_password(password: &str) -> Result<String, AppError> {
    let params = ParamsBuilder::new()
        .m_cost(19456) // 19MiB - Makes attackers use more RAM per hash, make attacks expensive
        .t_cost(2) // 2 iterations - More iterations = slower hashing = harder to crack
        .p_cost(1) // 1 thread - high parallelism could cause resource contention
        .build()?;
    // Those parameters match OWASP recommendations
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html

    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    // Argon2id combine Argon2d (resistant to GPU attacks) with Argon2i (resistant to sode-channel
    // attacks)

    let salt = SaltString::generate(&mut OsRng);
    // OsRng uses the OS's cryptographically secure
    // randon number generator
    // Salt will ensure that same passwords will make different hashs

    let hash = argon.hash_password(password.as_bytes(), &salt)?;
    debug!("Password hashed successfully");
    Ok(hash.to_string())
}

#[instrument(skip(password, hash))]
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)?;
    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => {
            debug!("Password verification successful");
            Ok(true)
        }
        Err(argon2::password_hash::Error::Password) => {
            warn!("Password verification failed - invalid password");
            Ok(false)
        }
        Err(e) => Err(e.into()),
    }
}
