use sha3::{Digest, Sha3_256};
use unicode_normalization::UnicodeNormalization;

use crate::consts::MAX_RESOURCE_NAME_LENGTH;

pub fn validate_input(input: &str) -> Option<String> {
    let normalized_input = input.nfc().collect::<String>();

    if input.len() > MAX_RESOURCE_NAME_LENGTH {
        return None;
    }

    for c in normalized_input.chars() {
        if c.is_control() || c.is_whitespace() {
            return None;
        }
    }
    Some(normalized_input)
}

pub fn hash(s: String) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(s.as_bytes());
    let mut h = hasher.finalize().to_vec();
    h.truncate(7);
    format!["{h:x?}"]
}
