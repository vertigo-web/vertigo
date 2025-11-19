use base64::{engine::general_purpose, Engine};

pub fn vec_to_string(data: &[u8]) -> String {
    general_purpose::STANDARD_NO_PAD.encode(data)
}

pub fn string_to_vec(encoded: &str) -> Result<Vec<u8>, String> {
    general_purpose::STANDARD_NO_PAD
        .decode(encoded)
        .map_err(|err| format!("ssr cache decoding error: {err}"))
}

#[test]
fn test() {
    let data = vec![84, 68, 83, 32, 106];

    let str = vec_to_string(&data);
    assert_eq!(str, "VERTIGo".to_string());

    let data2 = string_to_vec(&str);

    assert_eq!(Ok(data), data2);
}
