pub fn vector_to_blob(vec: Vec<f32>) -> Vec<u8> {
    let blob: Vec<u8> = vec
        .iter()
        .flat_map(|&n| n.to_le_bytes()) // f32 -> [u8; 4]
        .collect();
    blob
}

pub fn blob_to_vector(blob: Vec<u8>) -> Vec<f32> {
    let vec: Vec<f32> = blob
        .chunks_exact(4)
        .map(|chunk| {
            // chunkは&[u8]なので、[u8; 4]に変換して復元
            let array = chunk.try_into().expect("Slice with incorrect length");
            f32::from_le_bytes(array)
        })
        .collect();
    vec
}
