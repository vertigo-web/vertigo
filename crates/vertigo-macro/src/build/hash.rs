use sha2::{Sha256, Digest};

pub fn get_hash(data: &[u8]) -> String {
    // create a Sha256 object
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();

    hex::encode(&result[..])
}

#[test]
fn test_hash() {
    let ddd = get_hash("vertigo".as_bytes());
    assert_eq!(ddd, "e5a559c8ce04fb73d98cfc83e140713600c1134ac676d0b4debcc9838c09e2d7");
}
