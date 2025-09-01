use color_eyre::eyre::Result;

use crate::domain::{Email, EmailClient};

pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, to: &Email, subject: &str, body: &str) -> Result<()> {
        tracing::debug!("Sending email to: {:?}", to.as_ref());
        tracing::debug!("Subject: {}", subject);
        tracing::debug!("Body: {}", body);
        Ok(())
    }
}
