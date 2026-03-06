//! `Vec<f32>` ↔ SQLite BLOB の変換。
//!
//! f32 は little-endian バイト列として格納する。

use file_feature_cache::CacheError;

/// `Vec<f32>` を BLOB (`Vec<u8>`) に変換する。
pub(crate) fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

/// BLOB (`Vec<u8>`) を `Vec<f32>` に変換する。
///
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
        let original = vec![1.0f32, 2.0, 3.14];
        let blob = vec_to_blob(&original);
        let restored = blob_to_vec(&blob).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn empty_vec_roundtrip() {
        let blob = vec_to_blob(&[]);
        let restored = blob_to_vec(&blob).unwrap();
        assert!(restored.is_empty());
    }

    #[test]
    fn invalid_blob_returns_error() {
        let bad = vec![0u8; 5]; // 5 は 4 の倍数でない
        assert!(blob_to_vec(&bad).is_err());
    }
}
