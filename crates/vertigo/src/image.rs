//! Pictures made smaller by the browser, f. ex. a photo from a phone before it goes to the
//! server: [`resize`].
//!
//! The browser reads the picture, in whatever format it shows and turned as its camera wrote
//! into it (EXIF), draws it scaled on a canvas nobody sees, and encodes what it drew. So an app
//! needs no decoders and no encoder of its own: for JPEG, PNG, WebP and GIF those are over
//! 400 KB of wasm. The browser's code for it is a chunk of its own, loaded with the first
//! picture.
//!
//! ```rust
//! use vertigo::image::{Resize, resize};
//!
//! /// The photo as a JPEG of at most 1600 px a side.
//! async fn for_upload(photo: Vec<u8>) -> Result<Vec<u8>, String> {
//!     let resized = resize(photo, Resize::fit(1600, 1600)).await?;
//!     log::info!(
//!         "{}x{} made {}x{}",
//!         resized.original_width, resized.original_height, resized.width, resized.height
//!     );
//!     resized.data.ok_or_else(|| "left as it was".to_string())
//! }
//! ```
//!
//! Only a browser does it: on the server, where pages are rendered, and in an app's tests
//! [`resize`] fails.

use crate::{
    JsJson,
    dev::command::{ImageAnswer, ImageOp, decode_json},
    driver_module::api::{api_browser_command, api_callbacks},
    future_box::FutureBox,
};

/// The format a picture is encoded to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Jpeg,
    Png,
    Webp,
}

impl Format {
    fn mime(self) -> &'static str {
        match self {
            Format::Jpeg => "image/jpeg",
            Format::Png => "image/png",
            Format::Webp => "image/webp",
        }
    }
}

/// What [`resize`] makes of a picture.
#[derive(Clone, Debug, PartialEq)]
pub struct Resize {
    /// The width the picture has to fit in, in pixels.
    pub max_width: u32,
    /// The height the picture has to fit in, in pixels. It keeps its proportions, and one that
    /// fits already isn't made bigger.
    pub max_height: u32,
    /// What it is encoded to. A browser that can't encode it gives a PNG, as browsers do.
    pub format: Format,
    /// Of a JPEG or a WebP, from 0, the smallest file, to 1.
    pub quality: f32,
    /// What shows where the picture is transparent when it becomes a JPEG, which has no
    /// transparency: a color as CSS takes it.
    pub background: String,
    /// Leave a picture that fits as it is: [`Resized::data`] is `None` then, and nothing is
    /// encoded. For a picture which already is of the format wanted, so that encoding it once
    /// more wouldn't only cost quality.
    pub keep_fitting: bool,
}

impl Resize {
    /// A JPEG of quality 0.85 fitting `max_width` x `max_height`, on white where the picture
    /// is transparent.
    pub fn fit(max_width: u32, max_height: u32) -> Self {
        Self {
            max_width,
            max_height,
            format: Format::Jpeg,
            quality: 0.85,
            background: "#fff".to_string(),
            keep_fitting: false,
        }
    }
}

/// A picture [`resize`] made.
#[derive(Clone, Debug, PartialEq)]
pub struct Resized {
    /// The picture encoded. `None` when it fitted and was to be kept, see
    /// [`Resize::keep_fitting`].
    pub data: Option<Vec<u8>>,
    /// The size of the picture in `data`, in pixels - of the one given, when it was kept.
    pub width: u32,
    pub height: u32,
    /// The size of the picture given, turned as its camera wrote into it.
    pub original_width: u32,
    pub original_height: u32,
}

/// The picture in `data` scaled down to fit `how` and encoded, by the browser. `data` is a file
/// of any format the browser shows: JPEG, PNG, WebP, GIF (its first frame), AVIF and so on.
///
/// Fails with what the browser said when it can't read the picture, and when there is no
/// browser: on the server and in tests.
pub async fn resize(data: Vec<u8>, how: Resize) -> Result<Resized, String> {
    if !api_browser_command().is_browser() {
        return Err("a picture is resized by a browser, and there is none here".to_string());
    }

    let (sender, answer) = FutureBox::new();
    // Let go of once the answer came, or when nobody waits for it any more
    let (callback, _callback) = api_callbacks().register(move |answer| {
        sender.publish(answer);
        JsJson::Null
    });

    api_browser_command().image(ImageOp::Resize {
        callback,
        data: JsJson::Vec(data),
        max_width: how.max_width,
        max_height: how.max_height,
        mime: how.format.mime().to_string(),
        quality: how.quality,
        background: how.background,
        keep_fitting: how.keep_fitting,
    });

    resized(answer.await)
}

fn resized(answer: JsJson) -> Result<Resized, String> {
    match decode_json::<ImageAnswer>(answer) {
        Ok(ImageAnswer::Resized {
            width,
            height,
            original_width,
            original_height,
            data,
        }) => Ok(Resized {
            data: match data {
                JsJson::Vec(data) => Some(data),
                _ => None,
            },
            width,
            height,
            original_width,
            original_height,
        }),
        Ok(ImageAnswer::Failed { message }) => Err(message),
        Err(err) => Err(format!("resize -> decode error = {err}")),
    }
}

#[cfg(test)]
mod tests {
    use crate::{JsJson, JsJsonSerialize, dev::command::ImageAnswer};

    use super::{Format, Resize, Resized, resize, resized};

    #[test]
    fn the_answer_of_the_browser() {
        let answer = ImageAnswer::Resized {
            width: 1600,
            height: 1200,
            original_width: 4000,
            original_height: 3000,
            data: JsJson::Vec(vec![0xff, 0xd8, 0xff]),
        };
        assert_eq!(
            resized(answer.to_json()),
            Ok(Resized {
                data: Some(vec![0xff, 0xd8, 0xff]),
                width: 1600,
                height: 1200,
                original_width: 4000,
                original_height: 3000,
            })
        );

        // kept as it was: no data
        let kept = ImageAnswer::Resized {
            width: 800,
            height: 600,
            original_width: 800,
            original_height: 600,
            data: JsJson::Null,
        };
        assert_eq!(
            resized(kept.to_json()).map(|resized| resized.data),
            Ok(None)
        );

        let failed = ImageAnswer::Failed {
            message: "The source image could not be decoded.".to_string(),
        };
        assert_eq!(
            resized(failed.to_json()),
            Err("The source image could not be decoded.".to_string())
        );
        assert!(resized(JsJson::Null).is_err());
    }

    #[test]
    fn a_jpeg_by_default() {
        let how = Resize::fit(1600, 1200);
        assert_eq!((how.max_width, how.max_height), (1600, 1200));
        assert_eq!(how.format, Format::Jpeg);
        assert_eq!(how.format.mime(), "image/jpeg");
        assert_eq!(Format::Webp.mime(), "image/webp");
        assert_eq!(Format::Png.mime(), "image/png");
        assert!(!how.keep_fitting);
    }

    /// No browser in a test, so nothing is asked of one.
    #[test]
    fn fails_without_a_browser() {
        let future = resize(vec![1, 2, 3], Resize::fit(10, 10));
        let mut future = std::pin::pin!(future);
        let waker = std::task::Waker::noop();
        let mut context = std::task::Context::from_waker(waker);

        let std::task::Poll::Ready(result) = future.as_mut().poll(&mut context) else {
            panic!("resize waits for a browser there isn't");
        };
        assert!(result.is_err());
    }
}
