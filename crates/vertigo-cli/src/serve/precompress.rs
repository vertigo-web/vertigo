use actix_web::{
    HttpRequest, HttpResponse, Resource,
    guard::{self, GuardContext},
    http::header::{
        self, AcceptEncoding, ETag, Encoding, EntityTag, Header, HttpDate, IfNoneMatch,
        LastModified,
    },
    web::{self, Bytes},
};
use std::{collections::HashMap, io::Write, path::Path, sync::Arc, time::Instant};
use walkdir::{DirEntry, WalkDir};

use crate::build::get_hash;

/// The build's files compressed by [`precompress`], by the path they are served at
pub type Precompressed = HashMap<String, PrecompressedFile>;

pub struct PrecompressedFile {
    content_type: String,
    brotli: Bytes,
    etag: EntityTag,
    last_modified: Option<HttpDate>,
}

/// Formats worth it: images and fonts come compressed already
const COMPRESSIBLE: &[&str] = &[
    "wasm", "js", "mjs", "map", "css", "json", "svg", "html", "txt", "xml",
];

/// Compresses the files in `dest_dir` with brotli at its highest level, too slow to run per
/// response, to serve them under `http_root`.
pub fn precompress(dest_dir: impl AsRef<Path>, http_root: &str) -> Precompressed {
    let dest_dir = dest_dir.as_ref();
    let started = Instant::now();
    let mut files = Precompressed::new();
    let (mut size, mut compressed_size) = (0, 0);

    // `Files` doesn't serve hidden ones
    let entries = WalkDir::new(dest_dir)
        .into_iter()
        .filter_entry(|entry| entry.depth() == 0 || !is_hidden(entry))
        .filter_map(Result::ok);

    for entry in entries {
        let path = entry.path();

        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };

        if !entry.file_type().is_file() || !COMPRESSIBLE.contains(&extension) {
            continue;
        }

        let (content, brotli) = match read_compressed(path) {
            Ok(compressed) => compressed,
            Err(err) => {
                log::warn!("Can't precompress {}: {err}", path.display());
                continue;
            }
        };

        // A small file can come out bigger
        if brotli.len() >= content.len() {
            continue;
        }

        let Ok(relative) = path.strip_prefix(dest_dir) else {
            continue;
        };

        let url_path = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        size += content.len();
        compressed_size += brotli.len();

        files.insert(
            format!("{}/{url_path}", http_root.trim_end_matches('/')),
            PrecompressedFile {
                content_type: content_type(extension),
                etag: EntityTag::new_strong(get_hash(&brotli)),
                last_modified: entry
                    .metadata()
                    .ok()
                    .and_then(|metadata| metadata.modified().ok())
                    .map(HttpDate::from),
                brotli: Bytes::from(brotli),
            },
        );
    }

    log::info!(
        "Precompressed {} files with brotli: {}K -> {}K in {:.1?}",
        files.len(),
        size / 1024,
        compressed_size / 1024,
        started.elapsed()
    );

    files
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name().to_string_lossy().starts_with('.')
}

/// The file, and the file compressed by [`brotli_compress`]
fn read_compressed(path: &Path) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let content = std::fs::read(path)?;
    let brotli = brotli_compress(&content)?;
    Ok((content, brotli))
}

/// Compresses at quality 11, where `Compress` uses 3 to keep up with responses
pub(crate) fn brotli_compress(content: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut writer = brotli::CompressorWriter::new(Vec::new(), 64 * 1024, 11, 22);
    writer.write_all(content)?;
    Ok(writer.into_inner())
}

/// What `Files` sends, which adds the charset to text (`prefer_utf8`)
fn content_type(extension: &str) -> String {
    let mime = actix_files::file_extension_to_mime(extension);

    match mime.essence_str() {
        "application/javascript"
        | "text/html"
        | "text/css"
        | "text/plain"
        | "text/csv"
        | "text/tab-separated-values" => format!("{mime}; charset=utf-8"),
        _ => mime.to_string(),
    }
}

/// Serves the files [`precompress`] compressed to the browsers accepting brotli. Anything else
/// falls through to the next service.
pub fn precompressed_files(http_root: &str, files: Arc<Precompressed>) -> Resource {
    let pattern = format!("{}/{{file:.*}}", http_root.trim_end_matches('/'));
    let served = files.clone();

    web::resource(pattern)
        .guard(guard::Get())
        .guard(guard::fn_guard(move |ctx| {
            served.contains_key(ctx.head().uri.path()) && accepts_brotli(ctx)
        }))
        .to(move |req: HttpRequest| {
            let files = files.clone();
            async move { respond(&req, files.get(req.path())) }
        })
}

fn accepts_brotli(ctx: &GuardContext) -> bool {
    // A range is of the file as it is, which `Files` serves
    if ctx.head().headers().contains_key(header::RANGE) {
        return false;
    }

    ctx.header::<AcceptEncoding>()
        .and_then(|accept| accept.negotiate([Encoding::brotli()].iter()))
        == Some(Encoding::brotli())
}

fn respond(req: &HttpRequest, file: Option<&PrecompressedFile>) -> HttpResponse {
    let Some(file) = file else {
        return HttpResponse::NotFound().finish();
    };

    let unchanged = match IfNoneMatch::parse(req) {
        Ok(IfNoneMatch::Any) => true,
        Ok(IfNoneMatch::Items(etags)) => etags.iter().any(|etag| etag.weak_eq(&file.etag)),
        Err(_) => false,
    };

    let mut response = if unchanged {
        HttpResponse::NotModified()
    } else {
        HttpResponse::Ok()
    };

    response
        .insert_header(ETag(file.etag.clone()))
        .insert_header((header::VARY, "accept-encoding"));

    if let Some(last_modified) = file.last_modified {
        response.insert_header(LastModified(last_modified));
    }

    if unchanged {
        return response.finish();
    }

    // `Compress` leaves a response that has its `Content-Encoding` alone
    response
        .insert_header((header::CONTENT_TYPE, file.content_type.as_str()))
        .insert_header((header::CONTENT_ENCODING, "br"))
        .body(file.brotli.clone())
}

#[cfg(test)]
mod tests {
    use actix_files::Files;
    use actix_web::{
        App,
        dev::ServiceResponse,
        http::{
            StatusCode,
            header::{self, HeaderName},
        },
        test::{TestRequest, call_service, init_service, read_body},
    };
    use std::{io::Read, sync::Arc};

    use super::{precompress, precompressed_files};

    fn header(response: &ServiceResponse, name: HeaderName) -> Option<&str> {
        response
            .headers()
            .get(name)
            .and_then(|value| value.to_str().ok())
    }

    #[actix_web::test]
    async fn brotli_goes_to_browsers_accepting_it() -> std::io::Result<()> {
        let dest_dir =
            std::env::temp_dir().join(format!("vertigo-precompress-{}", std::process::id()));
        std::fs::create_dir_all(dest_dir.join("assets"))?;

        let script = "console.log('vertigo');\n".repeat(100);
        std::fs::write(dest_dir.join("assets/app.js"), &script)?;
        std::fs::write(dest_dir.join("logo.png"), vec![0; 1000])?;
        std::fs::write(dest_dir.join("index.json"), "{}")?;
        std::fs::write(dest_dir.join(".hidden.js"), &script)?;

        let files = precompress(&dest_dir, "/build");
        let mut paths = files.keys().collect::<Vec<_>>();
        paths.sort();
        // Not an image, nor a file brotli makes bigger, nor a hidden one
        assert_eq!(paths, ["/build/assets/app.js"]);

        let app = init_service(
            App::new()
                .service(precompressed_files("/build", Arc::new(files)))
                .service(Files::new("/build", &dest_dir)),
        )
        .await;

        let get = |accept_encoding: &str| {
            TestRequest::get()
                .uri("/build/assets/app.js")
                .insert_header((header::ACCEPT_ENCODING, accept_encoding.to_string()))
        };

        let response = call_service(&app, get("gzip, deflate, br, zstd").to_request()).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(header(&response, header::CONTENT_ENCODING), Some("br"));
        assert_eq!(header(&response, header::VARY), Some("accept-encoding"));
        let Some(etag) = header(&response, header::ETAG).map(str::to_string) else {
            panic!("a precompressed file should carry an ETag");
        };
        let mut decompressed = String::new();
        brotli::Decompressor::new(&read_body(response).await[..], 4096)
            .read_to_string(&mut decompressed)?;
        assert_eq!(decompressed, script);

        let revalidation = get("br")
            .insert_header((header::IF_NONE_MATCH, etag))
            .to_request();
        let response = call_service(&app, revalidation).await;
        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
        assert!(read_body(response).await.is_empty());

        // The file as it is, from `Files`
        for accept_encoding in ["gzip", "br;q=0, gzip"] {
            let response = call_service(&app, get(accept_encoding).to_request()).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(header(&response, header::CONTENT_ENCODING), None);
            assert_eq!(read_body(response).await, script.as_bytes());
        }

        let range = get("br")
            .insert_header((header::RANGE, "bytes=0-9"))
            .to_request();
        let response = call_service(&app, range).await;
        assert_eq!(response.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(read_body(response).await, script.as_bytes()[..10]);

        std::fs::remove_dir_all(&dest_dir)
    }
}
