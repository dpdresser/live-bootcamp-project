use std::hash::{Hash, Hasher};

use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};
use validator::validate_email;

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl Email {
    pub fn parse(email: &str) -> Result<Self> {
        let email_struct = Self(Secret::new(email.to_string()));

        if validate_email(email_struct.0.expose_secret()) {
            Ok(email_struct)
        } else {
            Err(eyre!("Invalid email format"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.org",
            "123@example.com",
            "a@b.co",
        ];

        for email in valid_emails {
            let result = Email::parse(email);
            assert!(
                result.is_ok(),
                "Expected {} to be valid, but got error: {:?}",
                email,
                result
            );

            if let Ok(parsed_email) = result {
                assert_eq!(parsed_email.as_ref().expose_secret(), email);
            }
        }
    }

    #[test]
    fn test_invalid_email() {
        let invalid_emails = vec![
            "invalid-email",
            "@example.com",
            "user@",
            "user@@example.com",
            "",
            "user@.com",
            "user@example.",
            "user name@example.com", // space in local part
        ];

        for email in invalid_emails {
            let result = Email::parse(email);
            assert!(
                result.is_err(),
                "Expected {} to be invalid, but it was accepted",
                email
            );
        }
    }

    #[test]
    fn test_as_ref_implementation() {
        let email_str = "test@example.com";
        let email = Email::parse(email_str).unwrap();
        assert_eq!(email.as_ref().expose_secret(), email_str);
    }
}
