use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Cfg {
    pub cache_path: Option<std::path::PathBuf>,
    pub provider: String,
    pub language: Option<String>,
    pub timeout: Option<u32>,
}

