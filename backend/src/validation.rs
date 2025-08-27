//! This module provides functions to validate user input when creating a new player account.
//!
//! The rules for each input field are defined within the specific validation function.
//!
//! This module also provides a function to validate all input fields at once, which is the most
//! concise way to utilize this module's functionality

use regex::Regex;
use serde::Serialize;

/// Check a string to make sure that it could be a valid username.
///
/// A valid username must pass the following checks:
///
/// - Its length must be between 6-16 characters.
/// - It may only include letters, numbers, and underscores.
/// - It may not start with an underscore.
/// - It may not contain two or more consecutive underscores.
///
/// **Note**: A username must also be *case-insensitively unique* - however, that check is beyond
/// the scope of this function.
///
/// ### Arguments
/// - `input`: The username to be tested
///
/// ### Returns
/// - `Ok`: The unit type.
/// - `Err`: A list of problems with the input.
pub fn validate_username(input: &str) -> Result<(), Vec<String>> {
    let mut problems: Vec<String> = vec![];

    let length = input.len();
    if length < 6 || length > 16 {
        problems.push(format!(
            "Username must be between 6 and 16 characters - found: {}",
            length
        ));
    }

    let legal_chars = Regex::new(r"^[\w\d]+$").unwrap();
    if !legal_chars.is_match(input) {
        problems.push(String::from(
            "Username contains illegal characters - it may only include letters, numbers, and underscores."
        ));
    }

    if input.starts_with("_") {
        problems.push(String::from("Username cannot start with an underscore."));
    }

    if input.contains("__") {
        problems.push(String::from(
            "Username may not contain consecutive underscores.",
        ));
    }

    match problems.len() {
        0 => Ok(()),
        _ => Err(problems),
    }
}

/// Check a string to make sure that it could be a valid password.
///
/// A valid password must pass the following checks:
///
/// - Its length must be between 8 and 32 characters.
/// - It must include at lease one of each of the following:
///   - An uppercase letter
///   - A lowercase letter
///   - A number
///   - One of the following symbols: `! @ # $ % ^ & * + = ?`
/// - It may not contain any spaces or symbols not listed above.
///
/// ### Arguments
/// - `input`: The password to be tested
///
/// ### Returns
/// - `Ok`: The unit type
/// - `Err`: A list of problems with the input.
pub fn validate_password(input: &str) -> Result<(), Vec<String>> {
    let mut problems: Vec<String> = vec![];

    let length = input.len();
    if length < 8 || length > 32 {
        problems.push(format!(
            "Password must be between 8 and 32 characters - found {}",
            length
        ));
    }

    let lower = Regex::new("[a-z]").unwrap();
    if !lower.is_match(input) {
        problems.push(String::from("Password must include a lowercase letter."));
    }

    let upper = Regex::new("[A-Z]").unwrap();
    if !upper.is_match(input) {
        problems.push(String::from("Password must include an uppercase letter."))
    }

    let digit = Regex::new(r"\d").unwrap();
    if !digit.is_match(input) {
        problems.push(String::from("Password must include a number."))
    }

    let symbol = Regex::new("[!@#$%^&*+=?]").unwrap();
    if !symbol.is_match(input) {
        problems.push(String::from(
            "Password must include one of the following symbols: ! @ # $ % ^ & * + = ?",
        ));
    }

    let illegal_char = Regex::new(r"^[\dA-Za-z!@#$%^&*+=?]+$").unwrap();
    if !illegal_char.is_match(input) {
        problems.push(String::from("Password includes illegal characters."))
    }

    match problems.len() {
        0 => Ok(()),
        _ => Err(problems),
    }
}

/// Check a string to make sure that it could be a valid email address.
///
/// A valid email address must pass the following checks:
///
/// - It must contain a **single** `@` character, separating the **prefix** and the **domain**.
/// - The *prefix* must **not**:
///   - Be an empty string.
///   - Contain characters other than letters, numbers, and the following characters: `. _ + -`
///   - Begin nor end with a dot.
///   - Contain consecutive dots.
/// - The *domain* must **not**:
///   - Be an empty string.
///   - Contain characters other than letters, numbers, dots, and hyphens.
/// - Each *level* of the domain (separated by dots) must **not**:
///   - Be an empty string (domain may not include consecutive dots).
///   - Begin nor end with a hyphen.
/// - The *top level domain* (the final level) must contain two or more characters.
///
/// **Note**: An email address must also be *case-insensitively unique* - however, that check is
/// beyond the scope of this function.
///
/// ### Arguments
/// - `input`: The email address to be tested
///
/// ### Returns
/// - `Ok`: The unit type
/// - `Err`: A list of problems with the input.
pub fn validate_email(input: &str) -> Result<(), Vec<String>> {
    let mut problems: Vec<String> = vec![];

    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() != 2 {
        problems.push(String::from(
            "Email must include a single @ character, separating the prefix and the domain.",
        ));
        return Err(problems);
    }

    let prefix = parts[0];
    let domain = parts[1];

    if prefix.len() == 0 {
        problems.push(String::from("Email prefix is empty!"));
    } else {
        let illegal_chars = Regex::new(r"^[A-Za-z\d._+-]+$").unwrap();
        if !illegal_chars.is_match(prefix) {
            problems.push(String::from("Email prefix contains illegal characters. Allowable characters are letters, numbers, and the following symbols: . _ + -"));
        }

        if prefix.starts_with('.') || prefix.ends_with('.') {
            problems.push(String::from(
                "Email prefix cannot begin nor end with a dot.",
            ));
        }

        let consecutive_dots = Regex::new(r"\.\.").unwrap();
        if consecutive_dots.is_match(prefix) {
            problems.push(String::from(
                "Email prefix cannot contain consecutive dots.",
            ));
        }
    }

    if domain.len() == 0 {
        problems.push(String::from("Email domain is empty!"));
    } else {
        let illegal_chars = Regex::new(r"^[A-Za-z\d\.-]+$").unwrap();
        if !illegal_chars.is_match(domain) {
            problems.push(String::from("Email domain includes illegal characters. Allowed characters are letters, numbers, and hyphens."));
        }

        let levels: Vec<&str> = domain.split('.').collect();
        if levels.len() < 2 {
            problems.push(String::from("Email domain must include at least one subdomain and a top level domain, separated by a dot."))
        }

        for &level in &levels {
            if level.len() == 0 {
                problems.push(String::from(
                    "Email domain may not include consecutive dots.",
                ));
            }
            if level.starts_with('-') || level.ends_with('-') {
                problems.push(String::from(
                    "Email contains a domain level which either starts or ends with a hyphen.",
                ));
            }
        }

        let tld = levels[levels.len() - 1];
        if tld.len() < 2 {
            problems.push(String::from(
                "Email top level domain must meet or exceed two characters.",
            ));
        }
    }

    match problems.len() {
        0 => Ok(()),
        _ => Err(problems),
    }
}

/// A simple helper function for use in the serialization of InputValidation structs.
fn is_ok(res: &Result<(), Vec<String>>) -> bool {
    res.is_ok()
}

/// Contains the validation information for all input fields at once.\
///
/// **Note**: This struct is serializable as it will be returned in the HTTP response body when a
/// user provides bad input. However, it will only include the fields which **failed validation**
/// within that serialized version.
#[derive(Serialize)]
pub struct InputValidation {
    #[serde(skip_serializing_if = "is_ok")]
    username: Result<(), Vec<String>>,

    #[serde(skip_serializing_if = "is_ok")]
    password: Result<(), Vec<String>>,

    #[serde(skip_serializing_if = "is_ok")]
    email: Result<(), Vec<String>>,
}

/// Check the input to make sure that all fields are valid, according to the defined rules for each
/// input field.
///
/// ### Arguments:
/// - `username`: The input to test for a valid username.
/// - `password`: The input to test for a valid password.
/// - `email`: The input to test for a valid email address.
///
/// ### Returns:
/// - `Ok`: The unit type. This signifies that all input has passed validation.
/// - `Err`: The output of the validation functions for each input. This signifies that **at least
///   one** input string has failed validation.
pub fn validate_all(username: &str, password: &str, email: &str) -> Result<(), InputValidation> {
    let username_validation = validate_username(username);
    let password_validation = validate_password(password);
    let email_validation = validate_email(email);

    if username_validation.is_err() || password_validation.is_err() || email_validation.is_err() {
        Err(InputValidation {
            username: username_validation,
            password: password_validation,
            email: email_validation,
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username() {
        // --- Valid usernames ---
        let valid_usernames = vec![
            "alice123",
            "bob_smith",
            "user_1",
            "abcdef",
            "a1b2c3d4e5f6g7h", // 16 chars
        ];

        for username in valid_usernames {
            let result = validate_username(username);
            assert!(
                result.is_ok(),
                "Expected '{}' to be valid, got errors: {:?}",
                username,
                result.err()
            );
        }

        // --- Invalid usernames ---
        let invalid_usernames = vec![
            "",                  // empty
            "abc",               // too short
            "aaaaaaaaaaaaaaaaa", // too long
            "_alice",            // starts with underscore
            "bob__smith",        // consecutive underscores
            "user!name",         // illegal character
            "john doe",          // space not allowed
            "name-with-dash",    // dash not allowed
        ];

        for username in invalid_usernames {
            let username = username.to_string(); // ensure owned String if using repeat
            let result = validate_username(&username);
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid, but got ok",
                username
            );

            if let Err(errors) = result {
                println!("'{}' errors: {:?}", username, errors);
            }
        }
    }

    #[test]
    fn test_validate_password() {
        // --- Valid passwords ---
        let valid_passwords = vec![
            "Password1!",
            "Abcdef1@",
            "MySecurePass9#",
            "A1b2C3d4$",
            "Complex+Pass=9",
        ];

        for pwd in valid_passwords {
            let result = validate_password(pwd);
            assert!(
                result.is_ok(),
                "Expected '{}' to be valid, got errors: {:?}",
                pwd,
                result.err()
            );
        }

        // --- Invalid passwords ---
        let invalid_passwords = vec![
            "",                                  // empty
            "short1!",                           // too short
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!A0", // too long
            "alllowercase1!",                    // no uppercase
            "ALLUPPERCASE1!",                    // no lowercase
            "NoNumber!",                         // no number
            "MissingSymbol1",                    // no symbol
            "BadSymbol1~",                       // invalid symbol
            "Spaces Not1!",                      // space not allowed
        ];

        for pwd in invalid_passwords {
            let pwd = pwd.to_string(); // ensure owned String if using repeat
            let result = validate_password(&pwd);
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid, but got ok",
                pwd
            );

            if let Err(errors) = result {
                println!("'{}' errors: {:?}", pwd, errors);
            }
        }
    }

    #[test]
    fn test_email_validation() {
        // --- Valid emails ---
        let valid_emails = vec![
            "alice@example.com",
            "bob.smith@sub.domain.io",
            "user+test@my-site.org",
            "a_b-c.d@letters123.com",
        ];

        for email in valid_emails {
            assert!(
                validate_email(email).is_ok(),
                "Expected '{}' to be valid",
                email
            );
        }

        // --- Invalid emails ---
        let invalid_emails = vec![
            "",                       // empty email
            "alice@",                 // empty domain
            "@example.com",           // empty prefix
            "alice..bob@example.com", // consecutive dots in prefix
            ".alice@example.com",     // prefix starts with dot
            "bob.@example.com",       // prefix ends with dot
            "alice@-example.com",     // domain starts with hyphen
            "alice@example-.com",     // domain ends with hyphen
            "alice@exam..ple.com",    // consecutive dots in domain
            "alice@example.c",        // TLD too short
            "alice@example",          // no dot in domain
            "alice@exa mple.com",     // space in domain
            "ali ce@example.com",     // space in prefix
        ];

        for email in invalid_emails {
            let result = validate_email(email);
            assert!(
                result.is_err(),
                "Expected '{}' to be invalid, but got ok",
                email
            );

            if let Err(errors) = result {
                println!("'{}' errors: {:?}", email, errors);
            }
        }
    }
}
