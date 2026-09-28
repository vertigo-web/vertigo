import { AppLocation } from "../../location/AppLocation";

/// Used by hydration, which claims an existing element and so has to look at its tag. The
/// command stream knows the tag from its dictionary index and calls [`hydrateLink`] directly.
export function injects(node: Element, appLocation: AppLocation) {
    if (node.tagName.toLowerCase() === 'a') {
        hydrateLink(node, appLocation);
    }
}

/// Follows a link to the app's own page without reloading it, and scrolls the way the browser
/// would: to the element named in the fragment (`/post?edit=1#comment-5`), otherwise to the top.
///
/// Clicks the browser gives a meaning of its own are left to it - with a modifier key (new tab,
/// new window, download) or on a link with a `target` or `download` attribute.
export function hydrateLink(node: Element, appLocation: AppLocation) {
    node.addEventListener('click', (e) => {
        const href = node.getAttribute('href');
        if (href === null || href.startsWith('#')) {
            return;
        }

        if (!isPlainClick(e) || node.hasAttribute('download') || !opensInPlace(node)) {
            return;
        }

        const path = pathInApp(href);
        if (path === null) {
            return;
        }

        e.preventDefault();
        appLocation.set('History', 'Push', path);
        // The app renders the new page before `set` returns. Only a part still waiting for its
        // data isn't there yet - a fragment pointing into it ends up at the top.
        scrollToFragment(path);
    })
}

/// The address the link leads to, the way the history router keeps it (`/post?edit=1#comment-5`),
/// or `null` when it leads out of the app: to another origin, or to another scheme (`mailto:`,
/// `tel:`, `javascript:`) - `pushState` would refuse those.
function pathInApp(href: string): string | null {
    let url: URL;
    try {
        url = new URL(href, document.baseURI);
    } catch {
        // not a valid address (`http://[`) - the app has nothing to render for it
        return null;
    }

    // `protocol` and `host` rather than `origin` - a `blob:` URL has the page's origin too
    if (url.protocol !== window.location.protocol || url.host !== window.location.host) {
        return null;
    }

    return url.pathname + url.search + url.hash;
}

function isPlainClick(event: Event): boolean {
    return event instanceof MouseEvent && event.button === 0 && !event.ctrlKey && !event.metaKey && !event.shiftKey && !event.altKey;
}

function opensInPlace(node: Element): boolean {
    const target = node.getAttribute('target');
    return target === null || target === '' || target === '_self';
}

/// Like the browser, tries the fragment as written first and percent-decoded second
/// (`#za%C5%BC%C3%B3%C5%82%C4%87` finds `id="zażółć"`).
function scrollToFragment(href: string) {
    const hash = href.indexOf('#');
    const fragment = hash === -1 ? '' : href.slice(hash + 1);
    const target = fragment === ''
        ? null
        : document.getElementById(fragment) ?? document.getElementById(decodeFragment(fragment));

    if (target === null) {
        window.scrollTo(0, 0);
    } else {
        target.scrollIntoView();
    }
}

function decodeFragment(fragment: string): string {
    try {
        return decodeURIComponent(fragment);
    } catch {
        // a malformed escape (`#%E0`) - nothing to decode, the browser would not find it either
        return fragment;
    }
}
