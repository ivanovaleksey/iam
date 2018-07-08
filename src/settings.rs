use config::{Config, File};
use failure;
use uuid::Uuid;

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;

use authn;

lazy_static! {
    #[allow(missing_debug_implementations)]
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::default());
}

#[macro_export]
macro_rules! get_settings {
    () => {{
        use $crate::settings::SETTINGS;
        SETTINGS.read().expect("Settings RwLock is poisoned")
    }};
}

#[derive(Debug, Default, Deserialize)]
pub struct Settings {
    pub iam_namespace_id: Uuid,
    pub authentication: Authentication,
    pub tokens: Tokens,
    pub providers: BTreeMap<authn::AuthKey, Provider>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Authentication {
    pub keyfile: PathBuf,
    #[serde(skip_deserializing)]
    pub key: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Tokens {
    pub keyfile: PathBuf,
    #[serde(skip_deserializing)]
    pub key: String,
    pub expires_in: u16,
    pub expires_in_max: u16,
}

#[derive(Debug, Default, Deserialize)]
pub struct Provider {
    pub keyfile: PathBuf,
    #[serde(skip_deserializing)]
    pub key: String,
}

pub fn init() -> Result<(), failure::Error> {
    debug!("Initializing settings");
    let mut settings = SETTINGS.write().unwrap();

    let mut c = Config::new();
    c.merge(File::with_name("Settings.toml"))?;
    *settings = c.try_into::<Settings>()?;

    let mut file = fs::File::open(&settings.authentication.keyfile)?;
    file.read_to_string(&mut settings.authentication.key)?;

    let mut file = fs::File::open(&settings.tokens.keyfile)?;
    file.read_to_string(&mut settings.tokens.key)?;

    for provider in settings.providers.values_mut() {
        let mut file = fs::File::open(&provider.keyfile)?;
        file.read_to_string(&mut provider.key)?;
    }

    Ok(())
}

pub fn iam_namespace_id() -> Uuid {
    let settings = get_settings!();
    settings.iam_namespace_id
}
