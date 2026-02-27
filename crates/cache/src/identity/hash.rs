use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

use sha2::{Digest, Sha256};

pub mod hash_strategy;
pub(super) mod mode;

pub fn full_hash(path: &Path) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(256 * 1024, file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn partial_hash(path: &Path, file_size: u64, partial_bytes: usize) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();

    // ファイルサイズ自体もハッシュに含める
    // → 異なるサイズのファイルで先頭/末尾の内容が偶然一致する誤検知を防ぐ
    hasher.update(file_size.to_le_bytes());

    // 先頭ブロック
    let head_len = (partial_bytes as u64).min(file_size) as usize;
    let mut buf = vec![0u8; head_len];
    file.read_exact(&mut buf)?;
    hasher.update(&buf);

    // 末尾ブロック (先頭と重複しない範囲のみ)
    let tail_offset = file_size.saturating_sub(partial_bytes as u64);
    if tail_offset > head_len as u64 {
        file.seek(SeekFrom::Start(tail_offset))?;
        let tail_len = (file_size - tail_offset) as usize;
        buf.resize(tail_len, 0);
        file.read_exact(&mut buf)?;
        hasher.update(&buf);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
