//! This module provides the lazy variable `ASSETS`, containing all email templates and images used
//! within the application.

use std::{
    fs::{read, read_to_string},
    path::{Path, PathBuf},
};

use lettre::message::header::ContentType;
use once_cell::sync::Lazy;

use crate::models::submodels::LanguagePreference;

// Directories storing different types of assets
#[doc(hidden)]
static EMAIL_TEMPLATES_DIRECTORY: &str = "assets/templates";
#[doc(hidden)]
static IMAGES_DIRECTORY: &str = "assets/img";

// File suffixes indicating language
#[doc(hidden)]
static DOT_ES: &str = ".es";
#[doc(hidden)]
static DOT_EN: &str = ".en";

// File extensions used in assets
#[doc(hidden)]
static DOT_HTML: &str = ".html";
#[doc(hidden)]
static DOT_TXT: &str = ".txt";
#[doc(hidden)]
static DOT_PNG: &str = ".png";

// Email template filenames
#[doc(hidden)]
static CHANGE_EMAIL_WARNING: &str = "change.email.warn";
#[doc(hidden)]
static CHANGE_EMAIL_CONFIRMATION: &str = "change.email.confirm";
#[doc(hidden)]
static CHANGE_PASSWORD: &str = "change.password";
#[doc(hidden)]
static CHANGE_USERNAME: &str = "change.username";
#[doc(hidden)]
static REGISTRATION_EMAIL: &str = "registration";
#[doc(hidden)]
static LOCKOUT_EMAIL: &str = "lockout";

// Email subjects
#[doc(hidden)]
static EN_SUB_CHANGE_EMAIL_WARNING: &str = "Your email address for D-Bo is about to change.";
#[doc(hidden)]
static ES_SUB_CHANGE_EMAIL_WARNING: &str =
    "Su dirección de correo electrónico de D-Bo está a punto de cambiar.";

#[doc(hidden)]
static EN_SUB_CHANGE_EMAIL_CONF: &str = "Confirm your new email address for D-Bo.";
#[doc(hidden)]
static ES_SUB_CHANGE_EMAIL_CONF: &str =
    "Confirme su nueva dirección de correo electronico de D-Bo.";

#[doc(hidden)]
static EN_SUB_CHANGE_PASSWORD: &str = "Your password for D-Bo has changed.";
#[doc(hidden)]
static ES_SUB_CHANGE_PASSWORD: &str = "Su contraseña de D-Bo ha cambiado.";

#[doc(hidden)]
static EN_SUB_CHANGE_USERNAME: &str = "Your username for D-Bo has changed.";
#[doc(hidden)]
static ES_SUB_CHANGE_USERNAME: &str = "Su nombre de usuario de D-Bo ha cambiado.";

#[doc(hidden)]
static EN_SUB_REGISTRATION: &str = "Confirm your email address to start playing D-Bo!";
#[doc(hidden)]
static ES_SUB_REGISTRATION: &str =
    "¡Confirme su dirección de correo electrónico para empezar a jugar D-Bo!";

#[doc(hidden)]
static EN_SUB_LOCKOUT: &str = "Your D-Bo account has been blocked!";
#[doc(hidden)]
static ES_SUB_LOCKOUT: &str = "¡Su cuenta de D-Bo ha sido bloqueado!";

// Image filenames
#[doc(hidden)]
static D_BO_LOGO: &str = "d_bo_logo";
#[doc(hidden)]
static BIGDEVDOG_LOGO: &str = "bigdevdog_logo";

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
fn template_path(template_name: &str, language_suffix: &str, extension: &str) -> PathBuf {
    Path::new(EMAIL_TEMPLATES_DIRECTORY)
        .join(format!("{}{}{}", template_name, language_suffix, extension))
}

/// Holds the subject, as well as both HTML and plaintext templates of a specific email, in a
/// specific language.
pub struct LocalizedEmailInfo {
    /// The subject line of the email.
    pub subject: String,
    /// The HTML template for the email, which is most-often used.
    pub html: String,
    /// The plaintext template for the email, which is sent as a backup for primitive email clients,
    /// and which also decrease the "spam score", increasing the likelihood that an email reaches
    /// the player's primary inbox.
    pub txt: String,
}

impl LocalizedEmailInfo {
    /// Create a new LocalizedEmailInfo struct
    ///
    /// ### Arguments
    /// - `template_name`: The email template title
    /// - `language_suffix`: The language suffix for the files
    /// - `subject`: The subject line of the email
    ///
    /// ### Panics
    /// If either file cannot be found
    fn new(template_name: &str, language_suffix: &str, subject: &str) -> Self {
        Self {
            subject: String::from(subject),
            html: read_template(&template_path(template_name, language_suffix, DOT_HTML)),
            txt: read_template(&template_path(template_name, language_suffix, DOT_TXT)),
        }
    }
}

/// Holds all variants of a single email template, sorted by language first, and then by format.
pub struct EmailLocalizationVariants {
    /// The English translations of the email template.
    pub en: LocalizedEmailInfo,
    /// The Spanish translations of the email template.
    pub es: LocalizedEmailInfo,
}

impl EmailLocalizationVariants {
    /// Construct a new EmailLocalizationVariants struct
    ///
    /// ### Arguments
    /// - `template_name`: The email template title
    ///
    /// ### Panics
    /// If any of the four required files cannot be found
    fn new(template_name: &str, en_subject: &str, es_subject: &str) -> Self {
        Self {
            en: LocalizedEmailInfo::new(template_name, DOT_EN, en_subject),
            es: LocalizedEmailInfo::new(template_name, DOT_ES, es_subject),
        }
    }

    pub fn language(&self, language: &LanguagePreference) -> &LocalizedEmailInfo {
        match language {
            LanguagePreference::English => &self.en,
            LanguagePreference::Spanish => &self.es,
        }
    }
}

/// Holds all email templates used by the application, sorted by purpose first, then by language,
/// and finally by format.
pub struct EmailTemplates {
    /// An email sent to the proposed email address following a player changing their email address,
    /// allowing them to confirm the new mailbox.
    pub change_email_confirmation: EmailLocalizationVariants,
    /// An email sent to the current email address following a player changing their email address,
    /// informing them of the change.
    pub change_email_warning: EmailLocalizationVariants,
    /// An email sent to the player following a password change, informing them and allowing them to
    /// reset their password securely for 24 hours.
    pub change_password: EmailLocalizationVariants,
    /// An email sent to the player to inform them of a changed username.
    pub change_username: EmailLocalizationVariants,
    /// The lockout notification email template, sent after five or more failed login attempts.
    pub lockout: EmailLocalizationVariants,
    /// The registration email template, sent immediately upon player account creation.
    pub registration: EmailLocalizationVariants,
}

impl EmailTemplates {
    /// Configure all email templates within the application.
    ///
    /// ### Panics
    /// If any of the required template files cannot be found.
    fn configure() -> Self {
        Self {
            change_email_confirmation: EmailLocalizationVariants::new(
                CHANGE_EMAIL_CONFIRMATION,
                EN_SUB_CHANGE_EMAIL_CONF,
                ES_SUB_CHANGE_EMAIL_CONF,
            ),
            change_email_warning: EmailLocalizationVariants::new(
                CHANGE_EMAIL_WARNING,
                EN_SUB_CHANGE_EMAIL_WARNING,
                ES_SUB_CHANGE_EMAIL_WARNING,
            ),
            change_password: EmailLocalizationVariants::new(
                CHANGE_PASSWORD,
                EN_SUB_CHANGE_PASSWORD,
                ES_SUB_CHANGE_PASSWORD,
            ),
            change_username: EmailLocalizationVariants::new(
                CHANGE_USERNAME,
                EN_SUB_CHANGE_USERNAME,
                ES_SUB_CHANGE_USERNAME,
            ),
            lockout: EmailLocalizationVariants::new(LOCKOUT_EMAIL, EN_SUB_LOCKOUT, ES_SUB_LOCKOUT),
            registration: EmailLocalizationVariants::new(
                REGISTRATION_EMAIL,
                EN_SUB_REGISTRATION,
                ES_SUB_REGISTRATION,
            ),
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
