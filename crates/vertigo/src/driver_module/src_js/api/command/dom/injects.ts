import { AppLocation } from "../../location/AppLocation";

/// Used by hydration, which claims an existing element and so has to look at its tag. The
/// command stream knows the tag from its dictionary index and calls [`hydrateLink`] directly.
export function injects(node: Element, appLocation: AppLocation) {
    if (node.tagName.toLowerCase() === 'a') {
        hydrateLink(node, appLocation);
    }
}

export function hydrateLink(node: Element, appLocation: AppLocation) {
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
