#![allow(clippy::question_mark)]
use std::sync::Arc;
use std::path::Path;

use super::html::{parse_html, HtmlNode, HtmlDocument};


#[derive(Clone)]
pub struct MountPathConfig {
    //for http
    public_path: String,
    //for filesystem
    dest_dir: String,               //TODO - use Path structure
    //index after parsing
    pub index: Arc<HtmlDocument>,
    //path to wasm-file
    pub wasm_path: String,
}

impl MountPathConfig {
    pub fn new(dest_dir: String) -> Result<MountPathConfig, i32> {

        let index = parse_index(&dest_dir)?;
    
        let Some(wasm_path) = find_wasm_node_list(index.elements.as_ref()) else {
            log::error!("Problem with finding the path to the wasm file");
            return Err(-1);
        };

        let Some(public_path) = get_base_dir(&wasm_path) else {
            log::error!("Problem with finding the http base path");
            return Err(-1);
        };

        Ok(MountPathConfig {
            public_path,
            dest_dir,
            index,
            wasm_path,
        })
    }

    pub fn http_root(&self) -> String {
        self.public_path.clone()
    }

    pub fn fs_root(&self) -> String {
        self.dest_dir.clone()
    }

    pub fn translate_to_fs(&self, http_path: impl Into<String>) -> Result<String, i32> {
        let http_path = http_path.into();
        replace_prefix(&self.public_path, &self.dest_dir, &http_path)
    }
}


fn parse_index(dest_dir: &str) -> Result<Arc<HtmlDocument>, i32> {
    let index_path = Path::new(dest_dir).join("index.html");
    let index_html = match std::fs::read_to_string(&index_path) {
        Ok(data) => data,
        Err(err) => {
            log::error!("File read error: file={index_path:?}, error={err}");
            return Err(-1);
        }
    };

    let document = parse_html(&index_html);

    Ok(Arc::new(document))
}

fn find_wasm_node(node: &HtmlNode) -> Option<String> {
    if let HtmlNode::Element(element) = node {
        if let Some(path) = element.attr.get("data-vertigo-run-wasm") {
            return Some(path.clone());
        }

        return find_wasm_node_list(&element.children);
    }

    None
}

fn find_wasm_node_list(nodes: &[HtmlNode]) -> Option<String> {
    for node in nodes.iter() {
        if let Some(path) = find_wasm_node(node) {
            return Some(path);
        }
    }

    None
}

fn replace_prefix(public_path: &str, dest_dir: &str, path: &str) -> Result<String, i32> {
    let Some(rest) = path.strip_prefix(public_path) else {
        log::error!("Incorrect path http: path={path} (public_path={public_path}, dest_dir={dest_dir})");
        return Err(-1);
    };

    Ok(format!("{dest_dir}{rest}"))
}


fn get_base_dir(path: &str) -> Option<String> {
    let mut chunks: Vec<&str> = path.split('/').collect();
    let last = chunks.pop();

    if last.is_none() {
        return None;
    }

    if chunks.is_empty() {
        return None;
    }

    Some(chunks.join("/"))
}



#[cfg(test)] 
mod tests {
    use super::replace_prefix;
    use super::get_base_dir;

    #[test]
    fn test_replace_prefix() {
        assert_eq!(
            replace_prefix("/build", "demo_build", "/build/vertigo_demo.33.wasm"),
            Ok("demo_build/vertigo_demo.33.wasm".to_string())
        );

        assert_eq!(
            replace_prefix("/aaaa", "demo_build", "/build/vertigo_demo.33.wasm"),
            Err(-1)
        );
    }

    #[test]
    fn test_get_base_dir() {
        assert_eq!(
            get_base_dir("/build/vertigo_demo.b64f38e19fe1e36419c23ca9fe2cb26b6c9f2f75dc61b078ec7b7b5aca0430db.wasm"),
            Some("/build".to_string())
        );

        assert_eq!(
            get_base_dir("/vertigo_demo.b64f38e19fe1e36419c23ca9fe2cb26b6c9f2f75dc61b078ec7b7b5aca0430db.wasm"),
            Some("".to_string())
        );

        assert_eq!(
            get_base_dir("vertigo_demo.b64f38e19fe1e36419c23ca9fe2cb26b6c9f2f75dc61b078ec7b7b5aca0430db.wasm"),
            None
        );

        assert_eq!(
            get_base_dir("//vertigo_demo.b64f38e19fe1e36419c23ca9fe2cb26b6c9f2f75dc61b078ec7b7b5aca0430db.wasm"),
            Some("/".to_string())
        );

        assert_eq!(
            get_base_dir("/ddd/vertigo_demo.b64f38e19fe1e36419c23ca9fe2cb26b6c9f2f75dc61b078ec7b7b5aca0430db.wasm"),
            Some("/ddd".to_string())
        );

        assert_eq!(
            get_base_dir("dsadas/ddd/vertigo_demo.b64f38e19fe1e36419c23ca9fe2cb26b6c9f2f75dc61b078ec7b7b5aca0430db.wasm"),
            Some("dsadas/ddd".to_string())
        );

    }
}

