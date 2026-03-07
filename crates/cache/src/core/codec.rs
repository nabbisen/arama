//! `Vec<f32>` ↔ SQLite BLOB の変換。
//!
//! f32 は little-endian バイト列として格納する。

use file_feature_cache::CacheError;

pub(crate) fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// バイト長が 4 の倍数でない場合は [`CacheError::InvalidVectorBlob`] を返す。
pub(crate) fn blob_to_vec(b: &[u8]) -> Result<Vec<f32>, CacheError> {
    if b.len() % 4 != 0 {
        return Err(CacheError::InvalidVectorBlob(b.len()));
    }
    Ok(b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let v = vec![1.0f32, 2.0, 3.14];
        assert_eq!(blob_to_vec(&vec_to_blob(&v)).unwrap(), v);
    }

    #[test]
    fn empty_roundtrip() {
        assert!(blob_to_vec(&vec_to_blob(&[])).unwrap().is_empty());
    }

    #[test]
    fn invalid_blob_returns_error() {
        assert!(blob_to_vec(&[0u8; 5]).is_err());
    }
}
