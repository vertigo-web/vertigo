use std::{collections::{HashMap}, sync::Arc};

use parking_lot::RwLock;
use reqwest::StatusCode;
use vertigo::{JsJsonSerialize, SsrFetchCache, VERTIGO_MOUNT_POINT_PLACEHOLDER, VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER};
use crate::serve::{html::{HtmlNode, element::AllElements, fetch_cache::FetchCache, html_element::HtmlElement}, mount_path::MountPathConfig, response_state::ResponseState};


pub fn build_response(
    all_elements: &AllElements,
    env: &HashMap<String, String>,
    mount_path: &MountPathConfig,
    status: StatusCode,
    fetch: &Arc<RwLock<FetchCache>>,
) -> ResponseState {
    let (mut root_html, css) = all_elements.get_response(false);

    let guard = fetch.read();
    let fetch_cache = SsrFetchCache::new(&guard.fetch_cache);
    let fetch_cache = fetch_cache.to_json().to_string();

    if let HtmlNode::Element(html) = &mut root_html {
        if html.name != "html" {
            // Not really possible
            return ResponseState::internal_error(format!(
                "Missing <html> element, found {} instead",
                html.name
            ));
        }

        html.add_attr("data-fetch-cache", fetch_cache);

        // Add custom env parameters
        for (env_name, env_value) in env {
            html.add_attr(format!("data-env-{env_name}"), env_value);
        }

        // Add dynamic values for public path
        html.add_attr(
            "data-env-vertigo-mount-point",
            mount_path.mount_point(),
        );
        html.add_attr(
            "data-env-vertigo-public-path",
            mount_path.dest_http_root(),
        );
    } else {
        return ResponseState::internal_error("Missing <html> element");
    }

    let head_exists = root_html.modify(&[("head", 0)], |head| {
        if mount_path.wasm_preload {
            let script_preconnect = HtmlElement::new("link")
                .attr("rel", "preload")
                .attr("href", mount_path.get_wasm_http_path())
                .attr("as", "script");

            head.add_child(script_preconnect);
        }
        head.add_child(css);
    });

    if !head_exists {
        log::info!("Missing <head> element");
    }

    let script = HtmlElement::new("script")
        .attr("type", "module")
        .attr(
            "data-vertigo-run-wasm",
            mount_path.get_wasm_http_path(),
        )
        .attr("src", mount_path.get_run_js_http_path());

    let body_exists = root_html.modify(&[("body", 0)], move |body| {
        body.add_child(script);
    });

    if body_exists {
        let mut body = root_html.convert_to_string(true).replace(
            VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER,
            &mount_path.dest_http_root(),
        );

        if mount_path.mount_point() != "/" {
            body = body.replace(
                VERTIGO_MOUNT_POINT_PLACEHOLDER,
                mount_path.mount_point(),
            );
        } else {
            body = body.replace(VERTIGO_MOUNT_POINT_PLACEHOLDER, "");
        }

        ResponseState::html(status, body)
    } else {
        ResponseState::internal_error("Missing <body> element")
    }
}
