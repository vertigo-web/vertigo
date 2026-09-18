import { AppLocation } from "../../location/AppLocation";

/// Applies the behaviours that an element gets from us rather than from the markup.
///
/// Checks the tag itself, because `NodeAdopt` hands over a node built by the server and says
/// nothing about what it is. `createNode` does know the tag, and passes `isAnchor` so that it
/// only calls here when there is something to apply.
export function injects(node: Element, appLocation: AppLocation) {
    if (node.tagName.toLowerCase() === 'a') {
        hydrateLink(node, appLocation);
    }
}

function hydrateLink(node: Element, appLocation: AppLocation) {
    node.addEventListener('click', (e) => {
        let href = node.getAttribute('href');
        if (href === null) {
            return;
        }

        if (href.startsWith('#') || href.startsWith('http://') || href.startsWith('https://') || href.startsWith('//')) {
            return;
        }

        e.preventDefault();
        appLocation.set('History', 'Push', href);
        window.scrollTo(0, 0);
    })
}
