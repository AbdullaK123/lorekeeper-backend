use askama::Template;
use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::service::ServiceError;

#[derive(Template)]
#[template(path = "password_reset.html")]
pub struct PasswordResetEmail<'a> {
    reset_url: &'a str,
}

#[derive(Template)]
#[template(path = "email_verification.html")]
pub struct VerificationEmail<'a> {
    verification_url: &'a str,
}

#[derive(Clone)]
pub struct EmailService {
    pub resend: Resend,
}

impl EmailService {
    pub fn new(resend_api_key: &str) -> Self {
        Self {
            resend: Resend::new(resend_api_key),
        }
    }

    #[tracing::instrument(name = "email.send_password_reset", level = "info", skip_all)]
    pub async fn send_password_reset_email(
        &self,
        user_email: &str,
        reset_url: &str,
    ) -> Result<(), ServiceError> {
        let template = PasswordResetEmail { reset_url };

        let rendered_template = template.render().map_err(|e| {
            tracing::error!("auth email template render failed");
            ServiceError::InternalError(e.to_string())
        })?;

        let email =
            CreateEmailBaseOptions::new("noreply@nexlore.net", [user_email], "Reset your password")
                .with_html(&rendered_template);

        tracing::debug!("submitting auth email to provider");
        self.resend.emails.send(email).await.map_err(|e| {
            tracing::error!("auth email provider request failed");
            ServiceError::InternalError(e.to_string())
        })?;
        tracing::info!("auth email accepted by provider");

        Ok(())
    }

    #[tracing::instrument(name = "email.send_verification", level = "info", skip_all)]
    pub async fn send_verification_email(
        &self,
        user_email: &str,
        verification_url: &str,
    ) -> Result<(), ServiceError> {
        let template = VerificationEmail { verification_url };
        let rendered_template = template.render().map_err(|e| {
            tracing::error!("auth email template render failed");
            ServiceError::InternalError(e.to_string())
        })?;
        let email =
            CreateEmailBaseOptions::new("noreply@nexlore.net", [user_email], "Verify your email")
                .with_html(&rendered_template);
        tracing::debug!("submitting auth email to provider");
        self.resend.emails.send(email).await.map_err(|e| {
            tracing::error!("auth email provider request failed");
            ServiceError::InternalError(e.to_string())
        })?;
        tracing::info!("auth email accepted by provider");
        Ok(())
    }
}
