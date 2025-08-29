use color_eyre::eyre::Result;

use crate::domain::Email;

// This trait represents the email client interface all
// concrete implementations must adhere to.
#[async_trait::async_trait]
pub trait EmailClient {
    async fn send_email(&self, to: &Email, subject: &str, body: &str) -> Result<()>;
}
