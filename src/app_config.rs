use config::Config;
use lazy_static::lazy_static;

use crate::consts::{DEFAULT_STATIC_DIR, MAX_FILE_SIZE};

lazy_static! {
    pub static ref CONFIG: Config = {
        let mut conf_builder = Config::builder();
        for (nym, val) in [
            ("max_file_size", MAX_FILE_SIZE),
            ("static_image_dir", DEFAULT_STATIC_DIR),
        ] {
            conf_builder = if let Ok(cb) = conf_builder.set_default(nym, val) {
                cb
            } else {
                panic!["Failed to init variable '{nym}' with value '{val}'"];
            };
        }
        conf_builder
            .build()
            .expect("Unable to create configuration.")
    };
}

pub mod get_config {
    use std::path::PathBuf;

    use byte_unit::Byte;

    use super::*;

    pub fn max_file_size<T>() -> T
    where
        T: TryFrom<u128>,
        <T as std::convert::TryFrom<u128>>::Error: std::fmt::Debug,
    {
        CONFIG
            .get::<Byte>("max_file_size")
            .map(|b| b.get_bytes())
            .unwrap_or(Byte::from_str(MAX_FILE_SIZE).unwrap().get_bytes())
            .try_into()
            .unwrap()
    }

    //    pub fn max_cache_size() -> usize {
    //        CONFIG
    //            .get::<Byte>("max_cache_size")
    //            .map(|b| b.get_bytes())
    //            .unwrap_or(MAX_CACHE_SIZE as u128)
    //            .try_into()
    //            .unwrap_or(MAX_CACHE_SIZE)
    //    }

    pub fn static_image_dir() -> PathBuf {
        CONFIG
            .get::<PathBuf>("static_image_dir")
            .unwrap_or(DEFAULT_STATIC_DIR.into())
    }
}
