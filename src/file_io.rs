use std::{
    io::{Read, Write},
    path::PathBuf,
};

use actix_web::web::Bytes;
use moka::future::Cache;

use crate::app_config::get_config;

pub struct FileIo {
    pub cache: Cache<String, Bytes>,
}

fn get_file_path(path: &str) -> PathBuf {
    get_config::static_image_dir().join(path)
}

fn load_static_file(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(get_file_path(path))?;
    let mut data = Vec::with_capacity(file.metadata()?.len() as usize);
    file.read_to_end(&mut data)?;
    Ok(data)
}

fn save_static_file(path: &str, data: Bytes) -> std::io::Result<()> {
    let mut file = std::fs::File::open(get_file_path(path))?;
    file.write_all(data.as_ref())?;
    Ok(())
}

impl FileIo {
    pub async fn load_image<'a>(&mut self, nym: String) -> std::io::Result<Bytes> {
        if self.cache.contains_key(&nym) {
            Ok(self.cache.get(&nym).unwrap())
        } else {
            let data: Bytes = load_static_file(&nym)?.into();
            self.cache.insert(nym, data.clone()).await;
            Ok(data)
        }
    }

    pub async fn save_image<'a>(&mut self, nym: String, data: Bytes) -> std::io::Result<()> {
        save_static_file(&nym, data.clone())?;
        self.cache.insert(nym, data).await;
        Ok(())
    }
}
