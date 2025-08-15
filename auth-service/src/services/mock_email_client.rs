use crate::domain::{Email, EmailClient};

pub struct MockEmailClient;

#[async_trait::async_trait]
impl EmailClient for MockEmailClient {
    async fn send_email(&self, to: &Email, subject: &str, body: &str) -> Result<(), String> {
        println!("Sending email to: {}", to.as_ref());
        println!("Subject: {}", subject);
        println!("Body: {}", body);
        Ok(())
    }
}
