//! A picture made smaller by the browser: one too big scaled down and encoded as a JPEG, one
//! that fits kept as it is, and a file that is no picture refused.

use vertigo::{
    Value, bind, component, dom, get_driver,
    image::{Resize, resize},
};

/// The size of the picture the tab makes.
const WIDTH: u32 = 200;
const HEIGHT: u32 = 100;

/// A picture as a BMP file, which needs no encoder: a header and the pixels as they are, three
/// bytes each, bottom row first.
fn picture() -> Vec<u8> {
    // Rows are padded to four bytes; 200 pixels of three bytes need none
    let pixels = WIDTH * HEIGHT * 3;
    let mut file = Vec::with_capacity(54 + pixels as usize);

    file.extend_from_slice(b"BM");
    file.extend_from_slice(&(54 + pixels).to_le_bytes());
    file.extend_from_slice(&[0; 4]);
    file.extend_from_slice(&54_u32.to_le_bytes());

    file.extend_from_slice(&40_u32.to_le_bytes());
    file.extend_from_slice(&WIDTH.to_le_bytes());
    file.extend_from_slice(&HEIGHT.to_le_bytes());
    file.extend_from_slice(&1_u16.to_le_bytes());
    file.extend_from_slice(&24_u16.to_le_bytes());
    file.extend_from_slice(&[0; 24]);

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            // blue, green, red
            file.extend_from_slice(&[(x * 255 / WIDTH) as u8, (y * 255 / HEIGHT) as u8, 128]);
        }
    }

    file
}

/// What a file starts with tells its format.
fn format(data: &[u8]) -> &'static str {
    match data {
        [0xff, 0xd8, 0xff, ..] => "JPEG",
        [0x89, b'P', b'N', b'G', ..] => "PNG",
        _ => "another format",
    }
}

#[component]
pub fn ImageDemo() {
    let result = Value::new("Nothing resized yet".to_string());

    let run = |result: &Value<String>, data: Vec<u8>, how: Resize| {
        let result = result.clone();
        get_driver().spawn(async move {
            let text = match resize(data, how).await {
                Ok(resized) => match resized.data {
                    Some(data) => format!(
                        "{}x{} made {}x{}, a {} of {} bytes",
                        resized.original_width,
                        resized.original_height,
                        resized.width,
                        resized.height,
                        format(&data),
                        data.len()
                    ),
                    None => format!("{}x{} kept as it is", resized.width, resized.height),
                },
                Err(message) => format!("Not resized: {message}"),
            };
            result.set(text);
        });
    };

    let smaller = bind!(result, |_| run(&result, picture(), Resize::fit(64, 64)));
    let keep = bind!(result, |_| run(
        &result,
        picture(),
        Resize {
            keep_fitting: true,
            ..Resize::fit(400, 400)
        }
    ));
    let no_picture = bind!(result, |_| run(
        &result,
        b"not a picture at all".to_vec(),
        Resize::fit(64, 64)
    ));

    dom! {
        <div>
            <p>"A picture of " {WIDTH} "x" {HEIGHT} " pixels, made here as a BMP file."</p>
            <button on_click={smaller}>"Make it fit 64x64"</button>
            <button on_click={keep}>"Fit 400x400, keeping one that fits"</button>
            <button on_click={no_picture}>"Resize what is no picture"</button>
            <p>{result}</p>
        </div>
    }
}
