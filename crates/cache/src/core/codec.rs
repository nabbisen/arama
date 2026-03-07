//! `Vec<f32>` / `Vec<Vec<f32>>` ↔ SQLite BLOB の変換。
//!
//! f32 は little-endian バイト列として格納する。
//!
//! ## `Vec<Vec<f32>>` のフォーマット
//!
//! 内側の各ベクトルが異なる長さを持ちうるため、
//! 先頭にベクトル数、各ベクトルの先頭に要素数を記録する。
//!
//! ```text
//! ┌──────────────┬──────────────┬───────────────────┬──────────────┬──── ─ ─
//! │ count: u32LE │ len_0: u32LE │ f32×len_0 (bytes) │ len_1: u32LE │ ...
//! └──────────────┴──────────────┴───────────────────┴──────────────┴──── ─ ─
//! ```

use file_feature_cache::CacheError;

// ---------------------------------------------------------------------------
// Vec<f32>  (画像 CLIP ベクトル)
// ---------------------------------------------------------------------------

pub(crate) fn vec_to_blob(v: &[f32]) -> Vec<u8> {
    v.iter().flat_map(|f| f.to_le_bytes()).collect()
}

pub(crate) fn blob_to_vec(b: &[u8]) -> Result<Vec<f32>, CacheError> {
    if b.len() % 4 != 0 {
        return Err(CacheError::InvalidVectorBlob(b.len()));
    }
    Ok(b.chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}

// ---------------------------------------------------------------------------
// Vec<Vec<f32>>  (動画 CLIP / wav2vec2 ベクトル列)
// ---------------------------------------------------------------------------

/// `Vec<Vec<f32>>` を BLOB に変換する。
///
/// フォーマット: `[count: u32LE][len_i: u32LE, f32×len_i ...]`
pub(crate) fn vecs_to_blob(vecs: &[Vec<f32>]) -> Vec<u8> {
    let total_f32s: usize = vecs.iter().map(|v| v.len()).sum();
    // count(4) + len_i(4)×n + f32(4)×total
    let mut buf = Vec::with_capacity(4 + 4 * vecs.len() + 4 * total_f32s);

    buf.extend_from_slice(&(vecs.len() as u32).to_le_bytes());
    for v in vecs {
        buf.extend_from_slice(&(v.len() as u32).to_le_bytes());
        for f in v {
            buf.extend_from_slice(&f.to_le_bytes());
        }
    }
    buf
}

/// BLOB を `Vec<Vec<f32>>` に変換する。
///
/// フォーマット不正の場合は [`CacheError::InvalidVectorBlob`] を返す。
pub(crate) fn blob_to_vecs(b: &[u8]) -> Result<Vec<Vec<f32>>, CacheError> {
    let err = || CacheError::InvalidVectorBlob(b.len());

    if b.len() < 4 {
        return Err(err());
    }

    let count = u32::from_le_bytes(b[..4].try_into().map_err(|_| err())?) as usize;
    let mut pos = 4;
    let mut result = Vec::with_capacity(count);

    for _ in 0..count {
        if pos + 4 > b.len() {
            return Err(err());
        }
        let len = u32::from_le_bytes(b[pos..pos + 4].try_into().map_err(|_| err())?) as usize;
        pos += 4;

        let byte_len = len * 4;
        if pos + byte_len > b.len() {
            return Err(err());
        }

        let floats = b[pos..pos + byte_len]
            .chunks_exact(4)
            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect();
        pos += byte_len;
        result.push(floats);
    }

    if pos != b.len() {
        return Err(err());
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Vec<f32> ---

    #[test]
    fn vec_roundtrip() {
        let v = vec![1.0f32, 2.0, 3.14];
        assert_eq!(blob_to_vec(&vec_to_blob(&v)).unwrap(), v);
    }

    #[test]
    fn vec_empty_roundtrip() {
        assert!(blob_to_vec(&vec_to_blob(&[])).unwrap().is_empty());
    }

    #[test]
    fn vec_invalid_blob_returns_error() {
        assert!(blob_to_vec(&[0u8; 5]).is_err());
    }

    // --- Vec<Vec<f32>> ---

    #[test]
    fn vecs_roundtrip() {
        let vecs = vec![vec![1.0f32, 2.0, 3.0], vec![4.0f32, 5.0], vec![6.0f32]];
        assert_eq!(blob_to_vecs(&vecs_to_blob(&vecs)).unwrap(), vecs);
    }

    #[test]
    fn vecs_empty_outer_roundtrip() {
        let vecs: Vec<Vec<f32>> = vec![];
        assert_eq!(blob_to_vecs(&vecs_to_blob(&vecs)).unwrap(), vecs);
    }

    #[test]
    fn vecs_empty_inner_roundtrip() {
        let vecs = vec![vec![], vec![1.0f32, 2.0], vec![]];
        assert_eq!(blob_to_vecs(&vecs_to_blob(&vecs)).unwrap(), vecs);
    }

    #[test]
    fn vecs_single_vector_roundtrip() {
        let vecs = vec![vec![0.1f32, 0.2, 0.3]];
        assert_eq!(blob_to_vecs(&vecs_to_blob(&vecs)).unwrap(), vecs);
    }

    #[test]
    fn vecs_invalid_truncated_returns_error() {
        let vecs = vec![vec![1.0f32, 2.0]];
        let mut blob = vecs_to_blob(&vecs);
        blob.pop(); // 末尾を1バイト削る
        assert!(blob_to_vecs(&blob).is_err());
    }

    #[test]
    fn vecs_invalid_trailing_bytes_returns_error() {
        let vecs = vec![vec![1.0f32]];
        let mut blob = vecs_to_blob(&vecs);
        blob.push(0xff); // 余分なバイトを追加
        assert!(blob_to_vecs(&blob).is_err());
    }
}
