use std::{collections::HashMap, sync::Arc};

use crate::serve::{
    html::{HtmlNode, element::AllElements, fetch_cache::FetchCache, html_element::HtmlElement},
    mount_path::MountConfig,
    response_state::ResponseState,
    timings::SsrProbe,
};
use actix_web::http::StatusCode;
use parking_lot::RwLock;
use vertigo::{
    JsJsonSerialize,
    dev::{SsrFetchCache, VERTIGO_MOUNT_POINT_PLACEHOLDER, VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER},
};

pub fn build_response(
    all_elements: &AllElements,
    env: &HashMap<String, String>,
    mount_path: &MountConfig,
    status: StatusCode,
    fetch: &Arc<RwLock<FetchCache>>,
    probe: &SsrProbe,
) -> ResponseState {
    let tree_mark = probe.start();
    let (mut root_html, css) = all_elements.get_response(false);
    probe.html_tree(tree_mark);

    // Starts here rather than above so the two `internal_error` returns below leave it
    // unspent: on a malformed document there is no HTML to have injected into, and
    // recording a partial region would read as a suspiciously fast injection.
    let inject_mark = probe.start();

    if let HtmlNode::Element(html) = &mut root_html {
        if html.name != "html" {
            // Not really possible
            return ResponseState::internal_error(format!(
                "Missing <html> element, found {} instead",
                html.name
            ));
        }
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

    let body_exists = root_html.modify(&[("body", 0)], move |body| {
        // Generate SSR cache
        let fetch_cache = {
            let fetch_cache_guard = fetch.read();
            SsrFetchCache::new(&fetch_cache_guard.fetch_cache)
                .to_json()
                .convert_to_string()
        };

        // Create hidden div
        let mut data_div = HtmlElement::new("div")
            .attr("id", "v-metadata")
            .attr("hidden", "hidden")
            .attr("style", "display: none")
            .attr("data-fetch-cache", fetch_cache);

        // Add custom env parameters
        for (env_name, env_value) in env {
            data_div.add_attr(format!("data-env-{env_name}"), env_value);
        }

        // Add dynamic values for public path
        data_div.add_attr("data-env-vertigo-mount-point", mount_path.mount_point());
        data_div.add_attr("data-env-vertigo-public-path", mount_path.dest_http_root());

        // Add disable hydration flag
        data_div.add_attr(
            "data-env-disable-hydration",
            if mount_path.disable_hydration {
                "true"
            } else {
                "false"
            },
        );

        // WASM script starter
        let script = HtmlElement::new("script")
            .attr("type", "module")
            .attr("data-vertigo-run-wasm", mount_path.get_wasm_http_path())
            .attr("src", mount_path.get_run_js_http_path());

        body.add_child(script);

        body.add_child(data_div);
    });

    probe.html_inject(inject_mark);

    if body_exists {
        // The two substitutions are a full pass over the whole document each, so they
        // belong to serialisation rather than to injection.
        let string_mark = probe.start();

        let mut body = root_html.convert_to_string(true).replace(
            VERTIGO_PUBLIC_BUILD_PATH_PLACEHOLDER,
            &mount_path.dest_http_root(),
        );

        if mount_path.mount_point() != "/" {
            body = body.replace(VERTIGO_MOUNT_POINT_PLACEHOLDER, mount_path.mount_point());
        } else {
            body = body.replace(VERTIGO_MOUNT_POINT_PLACEHOLDER, "");
        }

        probe.html_string(string_mark);

        ResponseState::html(status, body)
    } else {
        ResponseState::internal_error("Missing <body> element")
    }
}
