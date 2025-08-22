use regex::Regex;

/// Check a string to make sure that it could be a valid username.
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
/// - `un`: The username to be tested
///
/// ### Returns
/// - `Ok`: The unit type.
/// - `Err`: A list of problems with the input.
pub fn validate_username(un: &str) -> Result<(), Vec<String>> {
    let mut problems: Vec<String> = vec![];

    let length = un.len();
    if length < 6 || length > 16 {
        problems.push(format!(
            "Username must be between 6 and 16 characters - found: {}",
            length
        ));
    }

    let legal_chars = Regex::new(r"^[\w\d]+$").unwrap();
    if !legal_chars.is_match(un) {
        problems.push(String::from(
            "Username contains illegal characters - it may only include letters, numbers, and underscores."
        ));
    }

    if un.starts_with("_") {
        problems.push(String::from("Username cannot start with an underscore."));
    }

    if un.contains("__") {
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
/// - `pw`: The password to be tested
///
/// ### Returns
/// - `Ok`: The unit type
/// - `Err`: A list of problems with the input.
pub fn validate_password(pw: &str) -> Result<(), Vec<String>> {
    let mut problems: Vec<String> = vec![];

    let length = pw.len();
    if length < 8 || length > 32 {
        problems.push(format!(
            "Password must be between 8 and 32 characters - found {}",
            length
        ));
    }

    let lower = Regex::new("[a-z]").unwrap();
    if !lower.is_match(pw) {
        problems.push(String::from("Password must include a lowercase letter."));
    }

    let upper = Regex::new("[A-Z]").unwrap();
    if !upper.is_match(pw) {
        problems.push(String::from("Password must include an uppercase letter."))
    }

    let digit = Regex::new(r"\d").unwrap();
    if !digit.is_match(pw) {
        problems.push(String::from("Password must include a number."))
    }

    let symbol = Regex::new("[!@#$%^&*+=?]").unwrap();
    if !symbol.is_match(pw) {
        problems.push(String::from(
            "Password must include one of the following symbols: ! @ # $ % ^ & * + = ?",
        ));
    }

    let illegal_char = Regex::new(r"^[\dA-Za-z!@#$%^&*+=?]+$").unwrap();
    if !illegal_char.is_match(pw) {
        problems.push(String::from("Password includes illegal characters."))
    }

    match problems.len() {
        0 => Ok(()),
        _ => Err(problems),
    }
}
