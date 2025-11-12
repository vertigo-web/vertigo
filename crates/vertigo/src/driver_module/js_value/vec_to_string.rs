
pub fn vec_to_string(data: &[u8]) -> String {
    let encoded = data.iter().map(|b| format!("{:02x} ", b)).collect::<String>();
    encoded

}

pub fn string_to_vec(encoded: &str) -> Result<Vec<u8>, String> {
    let mut result = Vec::with_capacity((encoded.len() / 3) + 1);

    for i in (0..encoded.len()).step_by(3) {
        match u8::from_str_radix(&encoded[i..i + 2], 16) {
            Ok(char) => {
                result.push(char);
            },
            Err(_err) => {
                return Err(format!("decoding error on character index={i}"));
            }
        }
    }

    Ok(result)
}

#[test]
fn test() {
    let mut data = Vec::new();
    data.push(44);
    data.push(55);
    data.push(66);
    data.push(70);

    let str = vec_to_string(&data);
    assert_eq!(str, "2c 37 42 46 ".to_string());

    let data2 = string_to_vec(&str);

    assert_eq!(Ok(data), data2);
}