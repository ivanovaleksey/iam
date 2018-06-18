use config::{Config, File};
use failure;

use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::sync::RwLock;

lazy_static! {
    #[allow(missing_debug_implementations)]
    pub static ref SETTINGS: RwLock<Settings> = RwLock::new(Settings::default());
}

pub fn init() -> Result<(), failure::Error> {
    debug!("Initializing settings");
    let mut settings = SETTINGS.write().unwrap();

    let mut c = Config::new();
    c.merge(File::with_name("Settings.toml"))?;
    *settings = c.try_into::<Settings>()?;

    let mut file = fs::File::open(settings.public_key_path.clone())?;
    file.read_to_string(&mut settings.public_key)?;

    let mut file = fs::File::open(settings.private_key_path.clone())?;
    file.read_to_string(&mut settings.private_key)?;

    Ok(())
}

#[derive(Debug, Default, Deserialize)]
pub struct Settings {
    #[serde(skip_deserializing)]
    pub public_key: String,
    pub public_key_path: PathBuf,

    #[serde(skip_deserializing)]
    pub private_key: String,
    pub private_key_path: PathBuf,
}
