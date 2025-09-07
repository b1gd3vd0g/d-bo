//! This module provides the lazy variable `ASSETS`, containing all email templates and images used
//! within the application.

use std::{
    fs::{read, read_to_string},
    path::{Path, PathBuf},
};

use lettre::message::header::ContentType;
use once_cell::sync::Lazy;

// Directories storing different types of assets
#[doc(hidden)]
static EMAIL_TEMPLATES_DIRECTORY: &str = "assets/templates";
#[doc(hidden)]
static IMAGES_DIRECTORY: &str = "assets/img";

// File extensions used in assets
#[doc(hidden)]
static DOT_HTML: &str = ".html";
#[doc(hidden)]
static DOT_TXT: &str = ".txt";
#[doc(hidden)]
static DOT_PNG: &str = ".png";

// Email template filenames
#[doc(hidden)]
static CONFIRMATION_EMAIL: &str = "confirmation";

// Image filenames
#[doc(hidden)]
static D_BO_LOGO: &str = "d_bo_logo";
#[doc(hidden)]
static BIGDEVDOG_LOGO: &str = "bigdevdog_logo";

/// Holds both HTML and plaintext versions of a single email template.
pub struct EmailTemplate {
    /// The HTML template for the email, which is most-often used.
    pub html: String,
    /// The plaintext template to send as a backup.
    pub txt: String,
}

/// Read a template file into a String.
///
/// ### Arguments
/// - `path`: The path to the template
///
/// ### Panics
/// If the template cannot be found.
#[doc(hidden)]
fn read_template(path: &PathBuf) -> String {
    read_to_string(path).unwrap_or_else(|_| panic!("Could not read template at {:?}", path))
}

/// Construct a path to the template based on filename and extension.
///
/// ### Arguments
/// - `template_name`: The name of the file
#[doc(hidden)]
fn template_path(template_name: &str, extension: &str) -> PathBuf {
    Path::new(EMAIL_TEMPLATES_DIRECTORY).join(format!("{}{}", template_name, extension))
}

impl EmailTemplate {
    /// Construct a new EmailTemplate based on the filename.
    ///
    /// ### Arguments
    /// - `template_name`: The name of the templates, without the extension. This requires that both
    ///   the .txt and .html files have **the same name** and are located within the `
    ///   EMAIL_TEMPLATES_DIRECTORY`.
    ///
    /// ### Panics
    /// If either template file could not be found.
    pub fn new(template_name: &str) -> Self {
        Self {
            html: read_template(&template_path(template_name, DOT_HTML)),
            txt: read_template(&template_path(template_name, DOT_TXT)),
        }
    }
}

/// Contains all email templates used within the application.
pub struct EmailTemplates {
    /// The email sent when confirming a player account for the first time. This email may be resent
    /// if the player's confirmation token has expired before confirming the email address, but
    /// should **not** be sent when confirming a player's **new** email address after they have
    /// changed it.
    pub confirmation: EmailTemplate,
}

impl EmailTemplates {
    /// Configure all email templates.
    ///
    /// ### Panics
    /// If any of the email templates could not be found.
    pub fn configure() -> Self {
        Self {
            confirmation: EmailTemplate::new(CONFIRMATION_EMAIL),
        }
    }
}

/// Holds all information related to a single image.
pub struct Image {
    /// The bytes for the image
    bytes: Vec<u8>,
    /// The CID the image should use
    cid: String,
    /// The MIME type of the image
    mime_type: ContentType,
}

/// Construct a path to the image based on filename and extension.
#[doc(hidden)]
fn image_path(image_name: &str, extension: &str) -> PathBuf {
    Path::new(IMAGES_DIRECTORY).join(format!("{}{}", image_name, extension))
}

/// Read the image into a vector of bytes.
///
/// ### Panics
/// If the image cannot be found.
#[doc(hidden)]
fn read_image(path: &PathBuf) -> Vec<u8> {
    read(path).unwrap_or_else(|_| panic!("Could not read image at {:?}", path))
}

impl Image {
    /// Construct a new image based on filename and extension.
    ///
    /// ### Arguments
    /// - `image_name`: The name of the image
    /// - `extension`: The file extension associated with the image
    ///
    /// ### Panics
    /// - If the image could not be found
    /// - If mapping the extension to a MIME type fails
    /// - If the extension is unrecognized by the application
    pub fn new(image_name: &str, extension: &str) -> Self {
        Self {
            bytes: read_image(&image_path(image_name, extension)),
            cid: String::from(image_name),
            mime_type: match extension {
                ".png" => ContentType::parse("image/png").unwrap_or_else(|_| {
                    panic!(
                        "Could not create a content type for extension {}",
                        extension
                    )
                }),
                _ => panic!("Unrecognized image extension {}", extension),
            },
        }
    }

    /// Get a copy of the image bytes
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    /// Get a copy of the image CID
    pub fn cid(&self) -> String {
        self.cid.clone()
    }

    /// Get a copy of the image MIME type
    pub fn mime_type(&self) -> ContentType {
        self.mime_type.clone()
    }
}

/// A collection of all images needed within the application.
pub struct Images {
    /// The text logo for BigDevDog.
    pub bigdevdog_logo: Image,
    /// The logo for D-Bo.
    pub d_bo_logo: Image,
}

impl Images {
    /// Configure all images within the application
    ///
    /// ### Panics
    /// If any of the images could not be constructed for any reason
    pub fn configure() -> Self {
        Self {
            bigdevdog_logo: Image::new(BIGDEVDOG_LOGO, DOT_PNG),
            d_bo_logo: Image::new(D_BO_LOGO, DOT_PNG),
        }
    }
}

/// Holds all the assets required by the application.
pub struct Assets {
    /// A collection of all the email templates.
    pub templates: EmailTemplates,
    /// A collection of all the images.
    pub images: Images,
}

impl Assets {
    /// Configure all assets required by the application
    ///
    /// ### Panics
    /// If **any** asset cannot be constructed for any reason. The panic message will reflect the
    /// first configuration error that occurs.
    pub fn configure() -> Self {
        Self {
            templates: EmailTemplates::configure(),
            images: Images::configure(),
        }
    }
}

/// Holds all of the required assets for safe use throughout the application.
pub static ASSETS: Lazy<Assets> = Lazy::new(Assets::configure);
