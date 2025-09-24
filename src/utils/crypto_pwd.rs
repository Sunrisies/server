//! 密码哈希/校验工具，统一使用 argon2id
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

type Result<T> = std::result::Result<T, argon2::password_hash::Error>;

/// 明文 → 哈希字符串（存入数据库）
pub fn hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// 校验用户输入
pub fn verify(password: &str, hash: &str) -> Result<bool> {
    let parsed = PasswordHash::new(hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pwd_hash_and_verify() {
        let pwd = "123456";
        let hash = hash(pwd).unwrap();
        assert_ne!(pwd, hash);
        assert!(verify(pwd, &hash).unwrap());
        assert!(!verify("wrong", &hash).unwrap());
    }
}
