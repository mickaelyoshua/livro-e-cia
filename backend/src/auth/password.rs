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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_password_produces_phc_string_format() {
        let hash = hash_password("TestPassword123!").expect("Should hash");
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn hash_password_produces_unique_hashes_for_same_password() {
        let password = "SamePassword123!";
        let hash1 = hash_password(password).expect("Should hash");
        let hash2 = hash_password(password).expect("Should hash");
        assert_ne!(hash1, hash2); // Different salts
    }

    #[test]
    fn hash_password_uses_owasp_recommended_parameters() {
        let hash = hash_password("password").expect("Should hash");
        assert!(hash.contains("m=19456"), "Should use 19456 KiB memory");
        assert!(hash.contains("t=2"), "Should use 2 iterations");
        assert!(hash.contains("p=1"), "Should use 1 parallelism");
    }

    #[test]
    fn verify_password_returns_true_for_correct_password() {
        let password = "CorrectPassword123!";
        let hash = hash_password(password).expect("Should hash");
        let result = verify_password(password, &hash).expect("Should verify");
        assert!(result);
    }

    #[test]
    fn verify_password_returns_false_for_wrong_password() {
        let hash = hash_password("CorrectPassword").expect("Should hash");
        let result = verify_password("WrongPassword", &hash).expect("Should verify");
        assert!(!result);
    }

    #[test]
    fn verify_password_is_case_sensitive() {
        let hash = hash_password("Password123").expect("Should hash");
        let result = verify_password("password123", &hash).expect("Should verify");
        assert!(!result);
    }

    #[test]
    fn verify_password_rejects_invalid_hash_format() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }

    #[test]
    fn hash_password_handles_empty_password() {
        let result = hash_password("");
        assert!(result.is_ok());
    }

    #[test]
    fn hash_password_handles_very_long_password() {
        let long_password = "a".repeat(10000);
        let result = hash_password(&long_password);
        assert!(result.is_ok());
    }

    #[test]
    fn hash_password_handles_unicode() {
        let unicode_password = "senhaSegura123!";
        let hash = hash_password(unicode_password).expect("Should hash");
        let verified = verify_password(unicode_password, &hash).expect("Should verify");
        assert!(verified);
    }

    #[test]
    fn hash_password_handles_special_characters() {
        let special_password = "P@$$w0rd!#%^&*()[]{}";
        let hash = hash_password(special_password).expect("Should hash");
        let verified = verify_password(special_password, &hash).expect("Should verify");
        assert!(verified);
    }
}
