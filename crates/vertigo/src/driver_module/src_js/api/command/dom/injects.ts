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
/// new window, download) or on a link with a `target` or `download` attribute. So are links out
/// of the app, which it has no page for (see [`pathInApp`]).
export function hydrateLink(node: Element, appLocation: AppLocation) {
    node.addEventListener('click', (e) => {
        const href = node.getAttribute('href');
        if (href === null || href.startsWith('#')) {
            return;
        }

        if (!isPlainClick(e) || node.hasAttribute('download') || !opensInPlace(node)) {
            return;
        }

        const path = pathInApp(node, href, appLocation.mountPoint);
        if (path === null) {
            return;
        }

        e.preventDefault();
        appLocation.set('History', 'Push', path);
        // The app renders the new page before `set` returns. Only a part still waiting for its
        // data isn't there yet - a fragment pointing into it waits for it (see
        // [`scrollToPendingFragment`]).
        scrollToFragment(path);
    })
}

/// The address the link leads to, the way the history router keeps it (`/post?edit=1#comment-5`),
/// or `null` when it leads out of the app: to another origin or scheme (`mailto:`, `tel:`,
/// `javascript:`), outside the path the app is mounted at, or to a page marked `rel="external"`.
function pathInApp(node: Element, href: string, mountPoint: string): string | null {
    // The app at `/` can't tell a page of another app on the site (`/panel/`) from its own, so a
    // link to one says it (`~=` - one of the words in `rel`, `i` - in any letter case)
    if (node.matches('[rel~="external" i]')) {
        return null;
    }

    let url: URL;
    try {
        url = new URL(href, document.baseURI);
    } catch {
        // not a valid address (`http://[`) - the app has nothing to render for it
        return null;
    }

    // Another origin or scheme - `pushState` would refuse it. `protocol` and `host` rather than
    // `origin`, as a `blob:` URL has the page's origin too
    if (url.protocol !== window.location.protocol || url.host !== window.location.host) {
        return null;
    }

    // An app mounted at `/panel` leaves `/` and `/other/` to the browser - those pages belong to
    // whatever else the site serves
    const mount = mountPoint.replace(/\/+$/, '');
    if (mount !== '' && url.pathname !== mount && !url.pathname.startsWith(`${mount}/`)) {
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

/// How long a followed link's fragment waits for its element to arrive with the page's data.
const FRAGMENT_WAIT_MS = 10_000;

/// A fragment followed before the new page had its element (f. ex. for `/post#comments`) - see [`scrollToPendingFragment`].
let pending: { path: string, fragment: string, until: number, scrollY: number } | null = null;

/// Where a followed link lands: on the element its fragment names, otherwise at the top - where
/// the page also waits for an element that hasn't arrived yet.
function scrollToFragment(path: string) {
    const hash = path.indexOf('#');
    const fragment = hash === -1 ? '' : path.slice(hash + 1);
    const target = fragmentTarget(fragment);

    pending = null;
    if (target !== null) {
        target.scrollIntoView();
        return;
    }

    window.scrollTo(0, 0);
    // `#top` with nothing named so means the top of the page, as in the browser
    if (fragment !== '' && decodeFragment(fragment).toLowerCase() !== 'top') {
        pending = { path, fragment, until: Date.now() + FRAGMENT_WAIT_MS, scrollY: window.scrollY };
    }
}

/// Run after every render. Scrolls to the element a followed link's fragment names once it is
/// on the page - unless the reader has scrolled or left the address in the meantime, or it took
/// longer than [`FRAGMENT_WAIT_MS`].
export function scrollToPendingFragment() {
    if (pending === null) {
        return;
    }

    // The way to the top only goes up, even when the page scrolls smoothly
    // (`scroll-behavior: smooth`) - so the page lower than it has been means the reader scrolled.
    const here = window.location.pathname + window.location.search + window.location.hash;
    if (here !== pending.path || window.scrollY > pending.scrollY || Date.now() > pending.until) {
        pending = null;
        return;
    }
    pending.scrollY = window.scrollY;

    const target = fragmentTarget(pending.fragment);
    if (target !== null) {
        pending = null;
        target.scrollIntoView();
    }
}

/// The element a fragment names, the way the browser finds it.
function fragmentTarget(fragment: string): Element | null {
    return fragment === ''
        ? null
        : namedElement(fragment) ?? namedElement(decodeFragment(fragment));
}

/// The element with this `id`, otherwise the first `<a>` with this `name`.
function namedElement(name: string): Element | null {
    return document.getElementById(name)
        ?? Array.from(document.getElementsByName(name)).find((element) => element.tagName === 'A')
        ?? null;
}

function decodeFragment(fragment: string): string {
    try {
        return decodeURIComponent(fragment);
    } catch {
        // a malformed escape (`#%E0`) - nothing to decode, the browser would not find it either
        return fragment;
    }
}
