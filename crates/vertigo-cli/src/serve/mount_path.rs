#![allow(clippy::question_mark)]
use std::sync::Arc;
use std::path::Path;
use html_query_parser::Node;


#[derive(Clone)]
pub struct MountPathConfig {
    //for http
    public_path: String,
    //for filesystem
    dest_dir: String,               //TODO - use Path structure
    //index after parsing
    pub index: Arc<Vec<Node>>,
    //path to wasm-file
    pub wasm_path: String,
}

impl MountPathConfig {
    pub fn new(dest_dir: String) -> Result<MountPathConfig, i32> {

        let index = parse_index(&dest_dir)?;

        let Some(wasm_path) = find_wasm(&index) else {
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


fn parse_index(dest_dir: &str) -> Result<Arc<Vec<Node>>, i32> {
    let index_path = Path::new(dest_dir).join("index.html");
    let index_html = match std::fs::read_to_string(&index_path) {
        Ok(data) => data,
        Err(err) => {
            log::error!("File read error: file={index_path:?}, error={err}");
            return Err(-1);
        }
    };

    let document = html_query_parser::parse(&index_html);
    Ok(Arc::new(trim_list(document)))
}

fn trim_node(node: Node) -> Node {
    match node {
        Node::Element { name, attrs, children } => {
            let name = name.trim();
            let attrs = attrs
                .iter()
                .filter_map(|(key, value)| {
                    let key = key.trim();

                    if key != "" {
                        Some((key,value.trim()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            
            Node::new_element(name, attrs, trim_list(children))
        },
        rest => rest,
    }
}

fn trim_list(list: Vec<Node>) -> Vec<Node> {
    list
        .into_iter()
        .map(trim_node)
        .collect::<Vec<_>>()
}

fn find_wasm_node(node: &Node) -> Option<String> {
    if let Node::Element { attrs, children, .. } = node {
        if let Some(path) = attrs.get("data-vertigo-run-wasm") {
            return Some(path.clone());
        }

        return find_wasm_nodes(children);
    }

    None
}

fn find_wasm_nodes(nodes: &[Node]) -> Option<String> {
    for node in nodes.iter() {
        if let Some(path) = find_wasm_node(node) {
            return Some(path);
        }
    }

    None
}

pub fn find_wasm(nodes: &Arc<Vec<Node>>) -> Option<String> {
    find_wasm_nodes(nodes.as_slice())
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

