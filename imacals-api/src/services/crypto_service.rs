use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core, SaltString, Error as PasswordHashError};

pub struct CryptoService<'a> {
    argon2: Argon2<'a>,
}

impl<'a> CryptoService<'a> {
    pub fn new() -> Self {
        Self {
            argon2: Argon2::default(),
        }
    }

    pub fn hash_password(&self, string: &str) -> Result<String, PasswordHashError> {
        let salt = SaltString::generate(&mut rand_core::OsRng);

        Ok(self.argon2.hash_password(string.as_bytes(), &salt)?.to_string())
    }

    pub fn verify_password(&self, old_password: &str, new_password: &str) -> bool {
        let parsed_hash = PasswordHash::new(old_password).unwrap();

        self.argon2.verify_password(new_password.as_bytes(), &parsed_hash).is_ok()
    }
}