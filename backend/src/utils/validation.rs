use once_cell::sync::Lazy;
use regex::Regex;

// 使用 Lazy 静态变量避免在每次调用时重新编译正则表达式
// 这些正则表达式只会在第一次使用时编译一次，后续调用直接复用
pub static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

pub static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap());

// 静态正则表达式，用于解析 Netscape 书签文件
// 只在第一次使用时编译，避免每次导入时重新编译
#[allow(unused)]
pub static LINK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)<a[^>]*href="(?P<url>[^"]+)"[^>]*>(?P<title>[^<]+)"#)
        .expect("Failed to compile bookmark regex"));

#[allow(unused)]
pub static TAG_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)tags="(?P<tags>[^"]+)""#)
        .expect("Failed to compile tag regex"));

pub fn validate_email(email: &str) -> bool {
    EMAIL_REGEX.is_match(email)
}

pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Err("Password must contain at least one uppercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Err("Password must contain at least one lowercase letter".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Password must contain at least one digit".to_string());
    }

    Ok(())
}

pub fn validate_url(url: &str) -> bool {
    URL_REGEX.is_match(url)
}

pub fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }

    if username.len() > 50 {
        return Err("Username must be no more than 50 characters long".to_string());
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(
            "Username can only contain letters, numbers, underscores, and hyphens".to_string(),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("test@example.com"));
        assert!(validate_email("user.name+tag@domain.co.uk"));
        assert!(validate_email("user123@test-domain.com"));
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(!validate_email(""));
        assert!(!validate_email("invalid-email"));
        assert!(!validate_email("@example.com"));
        assert!(!validate_email("test@"));
        assert!(!validate_email("test.example.com"));
    }

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("Password123").is_ok());
        assert!(validate_password("MySecurePass456").is_ok());
        assert!(validate_password("TestPass1!").is_ok());
    }

    #[test]
    fn test_validate_password_invalid() {
        assert_eq!(
            validate_password("short"),
            Err("Password must be at least 8 characters long".to_string())
        );
        assert_eq!(
            validate_password("nouppercase1"),
            Err("Password must contain at least one uppercase letter".to_string())
        );
        assert_eq!(
            validate_password("NOLOWERCASE1"),
            Err("Password must contain at least one lowercase letter".to_string())
        );
        assert_eq!(
            validate_password("NoDigitsHere"),
            Err("Password must contain at least one digit".to_string())
        );
    }

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com"));
        assert!(validate_url("http://test-site.org"));
        assert!(validate_url("https://www.example.com/path"));
        assert!(validate_url("http://localhost:3000"));
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(!validate_url(""));
        assert!(!validate_url("not-a-url"));
        assert!(!validate_url("ftp://example.com"));
        assert!(!validate_url("://missing-protocol.com"));
    }

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("testuser").is_ok());
        assert!(validate_username("user_name").is_ok());
        assert!(validate_username("user-name").is_ok());
        assert!(validate_username("user123").is_ok());
        assert!(validate_username("a").is_err()); // Too short
    }

    #[test]
    fn test_validate_username_invalid() {
        assert_eq!(
            validate_username("ab"),
            Err("Username must be at least 3 characters long".to_string())
        );
        assert_eq!(
            validate_username(&"a".repeat(51)),
            Err("Username must be no more than 50 characters long".to_string())
        );
        assert_eq!(
            validate_username("user@name"),
            Err("Username can only contain letters, numbers, underscores, and hyphens".to_string())
        );
        assert_eq!(
            validate_username("user name"),
            Err("Username can only contain letters, numbers, underscores, and hyphens".to_string())
        );
    }
}
