use utils::hash;

pub fn compute_merkle_root(mut hashes: Vec<Vec<u8>>) -> Vec<u8> {
    if hashes.len() == 0 {
        return hash::hash(vec![]);
    } //TODO: delete when blocks have always at least one tx

    let mut val : Vec<u8>;
    if hashes.len() == 1 {
        val = hashes[0].clone();
        hashes.push(val);
    }

    while hashes.len() > 1 {
        if hashes.len()%2 != 0 {
            val = hashes[hashes.len()-1].clone();
            hashes.push(val);
        }
        let mut left_hash : Vec<u8> = Vec::new();
        for i in 0..hashes.len() {
            if i%2 != 0 {
                left_hash.append(&mut hashes[i].clone());
                hashes[(i+1)/2-1] = hash::hash(left_hash.clone());
            } else {
                left_hash = hashes[i].clone();
            }
        }
        hashes = hashes[..hashes.len()/2].to_vec();
    }

    hashes[0].clone()
}
