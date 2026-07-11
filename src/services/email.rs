// Copyright (C) 2026 Marcos Gabriel Miller
use lettre::{
    message::Mailbox,
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::{config::AppConfig, errors::AppError};

fn mailer(config: &AppConfig) -> Result<Option<AsyncSmtpTransport<Tokio1Executor>>, AppError> {
    let Some(host) = &config.smtp_host else {
        return Ok(None);
    };

    let transport_builder = if config.smtp_starttls {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
            .map_err(|error| AppError::InternalServerError(error.to_string()))?
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(host)
    }
    .port(config.smtp_port);

    let transport_builder = if let (Some(user), Some(password)) =
        (&config.smtp_user, &config.smtp_password)
    {
        transport_builder.credentials(Credentials::new(user.clone(), password.clone()))
    } else {
        transport_builder
    };

    Ok(Some(transport_builder.build()))
}

pub async fn send_audit_email(
    config: &AppConfig,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), AppError> {
    let Some(mailer) = mailer(config)? else {
        return Ok(());
    };

    let from = config
        .smtp_from
        .parse::<Mailbox>()
        .map_err(|error| AppError::InternalServerError(format!("Invalid SMTP_FROM: {error}")))?;

    let to = to
        .parse::<Mailbox>()
        .map_err(|error| AppError::InternalServerError(format!("Invalid recipient email: {error}")))?;

    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body.to_string())
        .map_err(|error| AppError::InternalServerError(error.to_string()))?;

    mailer
        .send(message)
        .await
        .map_err(|error| AppError::InternalServerError(error.to_string()))?;

    Ok(())
}
