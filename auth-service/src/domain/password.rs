use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Password {
    pub fn parse(password: &Secret<String>) -> Result<Self> {
        if password.expose_secret().len() < 8 {
            return Err(eyre!("Password must be at least 8 characters long"));
        }

        if !password.expose_secret().chars().any(|c| c.is_ascii_digit()) {
            return Err(eyre!("Password must contain at least 1 number"));
        }

        if !password
            .expose_secret()
            .chars()
            .any(|c| !c.is_alphanumeric())
        {
            return Err(eyre!("Password must contain at least 1 special character"));
        }

        Ok(Self(password.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_passwords() {
        let valid_passwords = vec![
            "password123!",
            "mySecure1@",
            "complex8#test",
            "Strong9$",
            "validPass1*",
            "12345678!",
            "abcdefgh1!",
        ];

        for password in valid_passwords {
            let password = Secret::new(password.to_string());
            let result = Password::parse(&password);
            assert!(
                result.is_ok(),
                "Expected '{}' to be valid, but got error: {:?}",
                password.expose_secret(),
                result
            );

            if let Ok(parsed_password) = result {
                assert_eq!(
                    parsed_password.as_ref().expose_secret(),
                    password.expose_secret()
                );
            }
        }
    }

    #[test]
    fn test_password_too_short() {
        let short_passwords = vec!["short1!", "abc1!", "1234567", "Pass1!", ""];

        for password in short_passwords {
            let result = Password::parse(&Secret::new(password.to_string()));
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid due to length",
                password
            );
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("at least 8 characters"));
        }
    }

    #[test]
    fn test_password_missing_number() {
        let passwords_without_numbers = vec![
            "password!",
            "noNumbers@",
            "onlyLetters#",
            "UPPERCASE$",
            "mixedCase%",
        ];

        for password in passwords_without_numbers {
            let result = Password::parse(&Secret::new(password.to_string()));
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid due to missing number",
                password
            );
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("at least 1 number"));
        }
    }

    #[test]
    fn test_password_missing_special_character() {
        let passwords_without_special_chars = vec![
            "password123",
            "onlyLetters1",
            "UPPERCASE123",
            "mixedCase456",
            "nospechars789",
        ];

        for password in passwords_without_special_chars {
            let result = Password::parse(&Secret::new(password.to_string()));
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid due to missing special character",
                password
            );
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("at least 1 special character"));
        }
    }

    #[test]
    fn test_password_multiple_validation_failures() {
        // Test passwords that fail multiple criteria
        let invalid_passwords = vec![
            ("short", "at least 8 characters"), // too short, no number, no special char
            ("password", "at least 1 number"),  // no number (first failure encountered)
            ("short1", "at least 8 characters"), // too short (first failure encountered)
        ];

        for (password, expected_error_part) in invalid_passwords {
            let result = Password::parse(&Secret::new(password.to_string()));
            assert!(result.is_err(), "Expected '{}' to be invalid", password);
            assert!(result
                .unwrap_err()
                .to_string()
                .contains(expected_error_part));
        }
    }

    #[test]
    fn test_as_ref_implementation() {
        let password_str = "validPass123!";
        let password = Password::parse(&Secret::new(password_str.to_string())).unwrap();
        assert_eq!(password.as_ref().expose_secret(), password_str);
    }

    #[test]
    fn test_various_special_characters() {
        let passwords_with_different_special_chars = vec![
            "password1!",
            "password2@",
            "password3#",
            "password4$",
            "password5%",
            "password6^",
            "password7&",
            "password8*",
            "password9(",
            "password0)",
            "password1-",
            "password2_",
            "password3=",
            "password4+",
            "password5[",
            "password6]",
            "password7{",
            "password8}",
            "password9|",
            "password0\\",
            "password1:",
            "password2;",
            "password3\"",
            "password4'",
            "password5<",
            "password6>",
            "password7,",
            "password8.",
            "password9?",
            "password0/",
            "password1 ", // space is also a special character
        ];

        for password in passwords_with_different_special_chars {
            let result = Password::parse(&Secret::new(password.to_string()));
            assert!(
                result.is_ok(),
                "Expected '{}' to be valid with special character",
                password
            );
        }
    }
}
