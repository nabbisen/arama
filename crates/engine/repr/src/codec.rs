use crate::error::ReprError;

pub fn vec_to_blob(vec: &Vec<f32>) -> Vec<u8> {
    let blob: Vec<u8> = vec
        .iter()
        .flat_map(|&n| n.to_le_bytes()) // f32 -> [u8; 4]
        .collect();
    blob
}

pub fn blob_to_vec(bytes: &[u8]) -> Result<Vec<f32>, ReprError> {
    if bytes.len() % 4 != 0 {
        return Err(ReprError::CodecInvalidLength(bytes.len()));
    }
    Ok(bytes
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect())
}
