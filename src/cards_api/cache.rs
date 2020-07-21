use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Error, Cfg};
use super::json_card::JsonCard;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct CardCache {
    pub provider: String,
    pub language: Option<String>,
    pub cards: HashMap<u32, JsonCard>,
}

pub fn load_cache(cfg: &Cfg) -> Result<CardCache, Error> {
    if let Some(p) = cfg.cache_path.as_ref() {
        // only try to load the cache if we think it exists
        if p.exists() {
            let cache: CardCache = serde_json::from_slice(&std::fs::read(p)?)?;
            if cache.provider == cfg.provider && cache.language == cfg.language {
                return Ok(cache);
            }
        }
    }
    Ok(CardCache { provider: cfg.provider.clone(), language: cfg.language.clone(), cards: HashMap::new() })
}

pub fn save_cache(cfg: &Cfg, cache: &CardCache) -> Result<(), Error> {
    if let Some(p) = cfg.cache_path.as_ref() {
        let cache_json = serde_json::to_string_pretty(cache)?;
        std::fs::write(p, cache_json)?;
    }
    Ok(())
}

