// Copyright (C) 2026 Marcos Gabriel Miller
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_exp_hours: i64,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_user: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: String,
    pub smtp_starttls: bool,
    pub audit_email_to: Option<String>,
    pub password_reset_exp_minutes: i64,
    pub password_reset_url_base: Option<String>,
    pub email_verification_exp_minutes: i64,
    pub email_verification_url_base: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("APP_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8000);
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://money_manager:money_manager@localhost:5432/money_manager".to_string());
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string());
        let jwt_exp_hours = std::env::var("JWT_EXP_HOURS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(24);
        let smtp_host = std::env::var("SMTP_HOST").ok().filter(|value| !value.trim().is_empty());
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(1025);
        let smtp_user = std::env::var("SMTP_USER").ok().filter(|value| !value.trim().is_empty());
        let smtp_password = std::env::var("SMTP_PASSWORD")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let smtp_from = std::env::var("SMTP_FROM")
            .unwrap_or_else(|_| "no-reply@money-manager.local".to_string());
        let smtp_starttls = std::env::var("SMTP_STARTTLS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(false);
        let audit_email_to = std::env::var("AUDIT_EMAIL_TO")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let password_reset_exp_minutes = std::env::var("PASSWORD_RESET_EXP_MINUTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(30);
        let password_reset_url_base = std::env::var("PASSWORD_RESET_URL_BASE")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let email_verification_exp_minutes = std::env::var("EMAIL_VERIFICATION_EXP_MINUTES")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(60);
        let email_verification_url_base = std::env::var("EMAIL_VERIFICATION_URL_BASE")
            .ok()
            .filter(|value| !value.trim().is_empty());

        Self {
            host,
            port,
            database_url,
            jwt_secret,
            jwt_exp_hours,
            smtp_host,
            smtp_port,
            smtp_user,
            smtp_password,
            smtp_from,
            smtp_starttls,
            audit_email_to,
            password_reset_exp_minutes,
            password_reset_url_base,
            email_verification_exp_minutes,
            email_verification_url_base,
        }
    }
}
