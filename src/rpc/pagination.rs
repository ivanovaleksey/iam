use rpc::{Error, Result};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Pagination {
    #[serde(default = "Pagination::default_limit")]
    pub limit: u16,
    #[serde(default = "Pagination::default_offset")]
    pub offset: u16,
}

impl Pagination {
    pub fn default_limit() -> u16 {
        let settings = get_settings!();
        settings.pagination.limit
    }

    pub fn default_offset() -> u16 {
        0
    }
}

pub fn check_limit(limit: u16) -> Result<()> {
    let settings = get_settings!();
    if limit > settings.pagination.limit_max {
        Err(Error::BadRequest)
    } else {
        Ok(())
    }
}
