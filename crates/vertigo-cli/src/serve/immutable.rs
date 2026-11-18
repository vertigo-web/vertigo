use std::path::Path;

use actix_web::{
    dev::ServiceRequest,
    dev::ServiceResponse,
    http::{
        StatusCode,
        header::{self, HeaderValue},
    },
};

/// A year - the longest a browser or a cache on the way (a CDN) keeps anything.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Length of the content hash in a file name: 64 bits in base64url (see `build::get_hash`).
const HASH_LEN: usize = 11;

/// Whether the build's file under `path` is named by its content - `name.HASH.ext`, as
/// `vertigo build` and `include_static!` save the files. A new content gets a new name, so
/// what is under this one never changes. The rest (`index.json`, files put into the build by
/// hand) stays for the browser to check with its ETag.
fn is_named_by_content(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let mut parts = name.rsplitn(3, '.');
    let (Some(_extension), Some(hash), Some(stem)) = (parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    !stem.is_empty()
        && hash.len() == HASH_LEN
        && hash
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// Whether the request, in the scope of the build's files, asks for a file in `dest_dir` named
/// by its content. The file has to be there: a missing one goes to the app's default service,
/// f. ex. a page, which must not stay in caches under its name.
pub(crate) fn is_build_file_named_by_content(dest_dir: &Path, req: &ServiceRequest) -> bool {
    let file = req.match_info().unprocessed().trim_start_matches('/');
    is_named_by_content(file)
        && !file.split('/').any(|part| part == "..")
        && dest_dir.join(file).is_file()
}

/// A file of the build named by its content never changes, so a browser keeps it without asking
/// again (also on reload) and a CDN keeps it too. Only when served - not an error.
pub(crate) fn mark_immutable<B>(mut res: ServiceResponse<B>, named: bool) -> ServiceResponse<B> {
    let status = res.status();
    if named && (status.is_success() || status == StatusCode::NOT_MODIFIED) {
        res.headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static(IMMUTABLE));
    }
    res
}

#[cfg(test)]
mod tests {
    use super::{HASH_LEN, is_named_by_content};
    use crate::build::get_hash;

    #[test]
    fn hash_in_the_name() {
        assert_eq!(get_hash(b"vertigo").len(), HASH_LEN);

        assert!(is_named_by_content("/build/app.p54ahL9yCpg.wasm"));
        assert!(is_named_by_content("/build/wasm_run.Mh5rjJWAfXI.js"));
        assert!(is_named_by_content("/build/images/logo.NGKVE9gB2p0.png"));
        assert!(is_named_by_content("/build/font.jx-LW7_aanc.woff2"));

        assert!(!is_named_by_content("/build/index.json"));
        assert!(!is_named_by_content("/build/wasm_run.js"));
        assert!(!is_named_by_content("/build/.p54ahL9yCpg.wasm"));
        assert!(!is_named_by_content("/build/app.p54ahL9yCp.wasm"));
        assert!(!is_named_by_content("/build/app.p54ahL9yC!g.wasm"));
        assert!(!is_named_by_content("/build/p54ahL9yCpg.wasm"));
    }
}
