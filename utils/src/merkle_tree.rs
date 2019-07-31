use super::hash;

pub fn compute_merkle_root(mut hashes: Vec<Vec<u8>>) -> Vec<u8> {
    if hashes.is_empty() {
        return vec![0; 32]
    };

    if hashes.len() == 1 {
        hashes.push(hashes[0].clone());
    };

    while hashes.len() > 1 {
        if hashes.len() % 2 != 0 {
            hashes.push(hashes.last().unwrap().clone());
        }

        let mut left_hash = hashes[0].clone();
        for i in 0..hashes.len() {
            let mut h = hashes[i].clone();

            if i % 2 == 0 {
                left_hash = h.clone();
            } else {
                let mut buffer = left_hash.clone();
                buffer.append(&mut h);
                hashes[((i + 1) / 2) - 1] = hash(hash(buffer.to_vec()));
            }
        }

        hashes.split_off(hashes.len() / 2);
    }

    hashes[0].clone()
}
