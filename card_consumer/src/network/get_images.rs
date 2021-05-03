use std::fs;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;

use crate::Error;
use super::{Cfg, get_cards};

pub async fn thumbs<P: AsRef<Path>>(dir: &P, cfg: &Cfg, ordinal_list: Option<Vec<u32>>) -> Result<BTreeMap<u32, (PathBuf, PathBuf)>, Error> {
    let (card_details, card_names) = if let Some(ords) = ordinal_list {
        get_cards::by_ordinal(cfg, ords).await?
    } else {
        get_cards::til_latest(cfg).await?
    };
    let dir_path = dir.as_ref();
    if !dir_path.exists() {
        fs::create_dir(dir_path)?;
    }
    let mut thumb_paths = BTreeMap::new();

    for (ord, card_name) in card_names.iter() {
        let lemma = card_name.asset_lemma();
        let (name_n, name_i) = (format!("{}.png", lemma), format!("{}-i.png", lemma));
        let (path_n, path_i) = (dir_path.join(name_n), dir_path.join(name_i));
        if !path_n.exists() || !path_i.exists() {
            let details = &card_details[ord];
            println!("downloading {} <- {}", path_n.to_string_lossy(), details.normal_appearance.thumbnail_asset_path);
            download(&path_n, &details.normal_appearance.thumbnail_asset_path).await?;
            println!("downloading {} <- {}", path_i.to_string_lossy(), details.idolized_appearance.thumbnail_asset_path);
            download(&path_i, &details.idolized_appearance.thumbnail_asset_path).await?;
        }
        thumb_paths.insert(*ord, (path_n, path_i));
    }

    Ok(thumb_paths)
}

async fn download<P: AsRef<Path>>(p: &P, url: &str) -> Result<(), Error> {
    let data = reqwest::get(url).await?.bytes().await?;
    std::fs::write(p, data)?;
    Ok(())
}

