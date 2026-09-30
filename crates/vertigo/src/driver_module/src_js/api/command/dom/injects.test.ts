// --- MOCKS ---
class MockLink {
    attributes: Map<string, string> = new Map();
    listener: ((e: any) => void) | null = null;

    constructor(attributes: Record<string, string>) {
        for (const [name, value] of Object.entries(attributes)) {
            this.attributes.set(name, value);
        }
    }
    getAttribute(name: string) { return this.attributes.get(name) ?? null; }
    hasAttribute(name: string) { return this.attributes.has(name); }
    addEventListener(_event: string, listener: (e: any) => void) { this.listener = listener; }

    /// Only the selector for `rel="external"`, matched the way the browser does - one of the
    /// words, in any letter case.
    matches(selector: string) {
        if (selector !== '[rel~="external" i]') {
            throw new Error(`MockLink can't match ${selector}`);
        }
        return (this.attributes.get('rel') ?? '').toLowerCase().split(/\s+/).includes('external');
    }
}

class MockMouseEvent {
    button = 0;
    ctrlKey = false;
    metaKey = false;
    shiftKey = false;
    altKey = false;
    preventDefault() { }
}

class MockTarget {
    scrolledTo = 0;
    /// Upper case for an HTML element, as `tagName` is in the browser.
    constructor(public tagName = 'DIV') { }
    scrollIntoView() { this.scrolledTo += 1; }
}

/// Elements of the page, by id and by `name`.
const elements: Map<string, MockTarget> = new Map();
const named: Map<string, Array<MockTarget>> = new Map();
let scrolledToTop = false;

(globalThis as any).document = {
    baseURI: 'https://app.test/post/5',
    getElementById: (id: string) => elements.get(id) ?? null,
    getElementsByName: (name: string) => named.get(name) ?? [],
};
(globalThis as any).window = {
    location: new URL('https://app.test/post/5'),
    scrollY: 0,
    scrollTo: () => {
        scrolledToTop = true;
        (globalThis as any).window.scrollY = 0;
    },
};
(globalThis as any).MouseEvent = MockMouseEvent;

import { hydrateLink, scrollToPendingFragment } from "./injects";

// --- TEST RUNNER ---
function assert(condition: boolean, message: string) {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
}

/// Opens the page the links are clicked on. It has no `<base>`, so its address is the base URL too.
function openPage(address: string) {
    (globalThis as any).document.baseURI = address;
    (globalThis as any).window.location = new URL(address);
}

/// Clicks a link built with `attributes` in an app mounted at `mountPoint`. `render` runs when
/// the app is told the new address - the moment it renders the new page.
function click(attributes: Record<string, string>, event: Partial<MouseEvent> = {}, render = () => { }, mountPoint = '/') {
    elements.clear();
    named.clear();
    scrolledToTop = false;
    // the link is on the page opened with `openPage`, scrolled somewhere down
    (globalThis as any).window.location = new URL((globalThis as any).document.baseURI);
    (globalThis as any).window.scrollY = 700;

    const link = new MockLink(attributes);
    const pushed: Array<string> = [];
    const appLocation = {
        mountPoint,
        set: (_target: string, _mode: string, href: string) => {
            pushed.push(href);
            // what `pushState` does to the address bar
            (globalThis as any).window.location = new URL(href, (globalThis as any).document.baseURI);
            render();
        },
    } as any;
    hydrateLink(link as any, appLocation);

    let prevented = false;
    link.listener?.(Object.assign(new MockMouseEvent(), event, {
        preventDefault: () => { prevented = true; },
    }));
    return { pushed, prevented };
}

// --- TESTS ---

function testPlainLinkGoesToTop() {
    console.log("\n--- Test links 1: without a fragment the new page starts at the top ---");
    const { pushed, prevented } = click({ href: '/list?page=2' });

    assert(prevented, "the browser does not reload the page");
    assert(pushed.length === 1 && pushed[0] === '/list?page=2', "the app gets the new address");
    assert(scrolledToTop, "scrolled to the top");
}

function testFragmentOfTheNewPage() {
    console.log("\n--- Test links 2: the fragment points into the new page ---");
    const comment = new MockTarget();
    // The element only appears when the app renders the new page
    click({ href: '/post?edit=5#comment-5' }, {}, () => elements.set('comment-5', comment));

    assert(comment.scrolledTo === 1, "scrolled to the element named in the fragment");
    assert(!scrolledToTop, "not to the top");
}

function testFragmentWithoutElement() {
    console.log("\n--- Test links 3: a fragment the new page doesn't have ---");
    click({ href: '/post#missing' });
    assert(scrolledToTop, "falls back to the top");

    click({ href: '/post#' });
    assert(scrolledToTop, "an empty fragment means the top");
}

function testEncodedFragment() {
    console.log("\n--- Test links 4: percent-encoded fragment ---");
    const heading = new MockTarget();
    click({ href: '/post#za%C5%BC%C3%B3%C5%82%C4%87' }, {}, () => elements.set('zażółć', heading));
    assert(heading.scrolledTo === 1, "found by the decoded id");

    click({ href: '/post#%E0' });
    assert(scrolledToTop, "a malformed escape doesn't throw and falls back to the top");
}

function testModifiedClicksAreLeftToTheBrowser() {
    console.log("\n--- Test links 5: clicks with a modifier key or another button ---");
    const events: Array<[string, Partial<MouseEvent>]> = [
        ['ctrl', { ctrlKey: true }],
        ['meta', { metaKey: true }],
        ['shift', { shiftKey: true }],
        ['alt', { altKey: true }],
        ['middle button', { button: 1 }],
    ];

    for (const [name, event] of events) {
        const { pushed, prevented } = click({ href: '/post' }, event);
        assert(!prevented && pushed.length === 0, `${name}+click is left to the browser`);
    }
}

function testTargetAndDownloadAreLeftToTheBrowser() {
    console.log("\n--- Test links 6: target and download ---");
    const newTab = click({ href: '/post', target: '_blank' });
    assert(!newTab.prevented && newTab.pushed.length === 0, "target=_blank opens a new tab");

    const download = click({ href: '/export.csv', download: '' });
    assert(!download.prevented && download.pushed.length === 0, "download downloads");

    const self = click({ href: '/post', target: '_self' });
    assert(self.prevented && self.pushed.length === 1, "target=_self stays in the app");
}

function testOtherLinksAreLeftToTheBrowser() {
    console.log("\n--- Test links 7: other origins, other schemes and bare fragments ---");
    const hrefs = [
        'https://example.com/', 'http://example.com/', '//example.com/', 'http://app.test/post',
        'mailto:someone@example.com', 'tel:+48123456789', 'javascript:void(0)',
        'blob:https://app.test/0b0e5f0c', 'http://[', '#section',
    ];
    for (const href of hrefs) {
        const { pushed, prevented } = click({ href });
        assert(!prevented && pushed.length === 0, `${href} is left to the browser`);
    }
}

function testAddressesResolvedLikeTheBrowser() {
    console.log("\n--- Test links 8: the app gets the address resolved against the page ---");
    const cases: Array<[string, string]> = [
        ['https://app.test/list?page=2', '/list?page=2'],
        ['//app.test/list', '/list'],
        ['edit', '/post/edit'],
        ['?page=2', '/post/5?page=2'],
        ['../about#team', '/about#team'],
    ];

    for (const [href, path] of cases) {
        const { pushed, prevented } = click({ href });
        assert(prevented && pushed.length === 1 && pushed[0] === path, `${href} opens ${path} in the app`);
    }
}

function testRelExternal() {
    console.log("\n--- Test links 9: rel=\"external\" leaves the app ---");
    for (const rel of ['external', 'nofollow external', 'EXTERNAL']) {
        const { pushed, prevented } = click({ href: '/panel/', rel });
        assert(!prevented && pushed.length === 0, `rel="${rel}" is left to the browser`);
    }

    const { pushed, prevented } = click({ href: '/post', rel: 'nofollow' });
    assert(prevented && pushed.length === 1, "another rel stays in the app");
}

function testMountPoint() {
    console.log("\n--- Test links 10: an app mounted at /panel ---");
    openPage('https://app.test/panel/comments/');

    for (const mountPoint of ['/panel', '/panel/']) {
        const inApp = ['/panel/settings/', '/panel', '?page=2', 'users/'];
        for (const href of inApp) {
            const { pushed, prevented } = click({ href }, {}, () => { }, mountPoint);
            assert(prevented && pushed.length === 1, `${href} stays in the app mounted at ${mountPoint}`);
        }

        const outside = ['/', '/panels-on-the-roof/', '../../other/'];
        for (const href of outside) {
            const { pushed, prevented } = click({ href }, {}, () => { }, mountPoint);
            assert(!prevented && pushed.length === 0, `${href} is left to the browser by the app mounted at ${mountPoint}`);
        }
    }

    openPage('https://app.test/post/5');
}

function testNamedAnchor() {
    console.log("\n--- Test links 11: an old-style anchor, <a name> ---");
    const anchor = new MockTarget('A');
    click({ href: '/support/#4b' }, {}, () => named.set('4b', [new MockTarget('INPUT'), anchor]));
    assert(anchor.scrolledTo === 1, "found by the name of an <a>, not of another element");

    const byId = new MockTarget();
    const byName = new MockTarget('A');
    click({ href: '/support/#paypal' }, {}, () => {
        elements.set('paypal', byId);
        named.set('paypal', [byName]);
    });
    assert(byId.scrolledTo === 1 && byName.scrolledTo === 0, "the id comes first, as in the browser");

    const svgLink = new MockTarget('a');
    click({ href: '/support/#logo' }, {}, () => named.set('logo', [svgLink]));
    assert(svgLink.scrolledTo === 0 && scrolledToTop, "an SVG <a> doesn't count");
}

function testFragmentOfDataArrivingLater() {
    console.log("\n--- Test links 12: the element comes with the page's data, after the click ---");
    const comments = new MockTarget();
    click({ href: '/post/7#comments' });
    assert(scrolledToTop && comments.scrolledTo === 0, "the new page starts at the top");

    scrollToPendingFragment();
    assert(comments.scrolledTo === 0, "a render without the element leaves the page where it is");

    elements.set('comments', comments);
    scrollToPendingFragment();
    assert(comments.scrolledTo === 1, "the render that brings the element scrolls to it");

    scrollToPendingFragment();
    assert(comments.scrolledTo === 1, "and only that one - later renders don't scroll again");

    const anchor = new MockTarget('A');
    click({ href: '/support/#4b' });
    named.set('4b', [anchor]);
    scrollToPendingFragment();
    assert(anchor.scrolledTo === 1, "an <a name> arriving later counts too");
}

function testPendingFragmentGivesUp() {
    console.log("\n--- Test links 13: waiting for the element ends ---");
    const later = new MockTarget();

    click({ href: '/post/7#comments' });
    (globalThis as any).window.scrollY = 250;
    elements.set('comments', later);
    scrollToPendingFragment();
    assert(later.scrolledTo === 0, "when the reader has scrolled in the meantime");

    click({ href: '/post/7#comments' });
    (globalThis as any).window.location = new URL('https://app.test/other');
    elements.set('comments', later);
    scrollToPendingFragment();
    assert(later.scrolledTo === 0, "when the reader has left the address");

    const now = Date.now;
    try {
        click({ href: '/post/7#comments' });
        const clickedAt = now();
        Date.now = () => clickedAt + 10_001;
        elements.set('comments', later);
        scrollToPendingFragment();
        assert(later.scrolledTo === 0, "after 10 seconds");
    } finally {
        Date.now = now;
    }

    click({ href: '/post/7#comments' });
    const other = new MockTarget();
    click({ href: '/post/8#other' }, {}, () => elements.set('other', other));
    elements.set('comments', later);
    scrollToPendingFragment();
    assert(other.scrolledTo === 1 && later.scrolledTo === 0, "when another link has been followed");
}

function testTopFragment() {
    console.log("\n--- Test links 14: #top ---");
    const top = new MockTarget();
    click({ href: '/post#top' });
    assert(scrolledToTop, "without an element named so, the top of the page");
    elements.set('top', top);
    scrollToPendingFragment();
    assert(top.scrolledTo === 0, "and nothing to wait for");

    click({ href: '/post#top' }, {}, () => elements.set('top', top));
    assert(top.scrolledTo === 1, "an element named so comes first");
}

function testSmoothScrollToTheTop() {
    console.log("\n--- Test links 15: scroll-behavior: smooth ---");
    // The way to the top takes a few frames, and renders come in between
    const jump = (globalThis as any).window.scrollTo;
    (globalThis as any).window.scrollTo = () => { scrolledToTop = true; };
    try {
        const comments = new MockTarget();
        click({ href: '/post/7#comments' });
        (globalThis as any).window.scrollY = 400;
        scrollToPendingFragment();
        (globalThis as any).window.scrollY = 0;
        elements.set('comments', comments);
        scrollToPendingFragment();
        assert(comments.scrolledTo === 1, "the fragment waits while the page is on its way to the top");

        const later = new MockTarget();
        click({ href: '/post/7#comments' });
        (globalThis as any).window.scrollY = 0;
        scrollToPendingFragment();
        (globalThis as any).window.scrollY = 150;
        elements.set('comments', later);
        scrollToPendingFragment();
        assert(later.scrolledTo === 0, "but not once the reader scrolls down from there");
    } finally {
        (globalThis as any).window.scrollTo = jump;
    }
}

testPlainLinkGoesToTop();
testFragmentOfTheNewPage();
testFragmentWithoutElement();
testEncodedFragment();
testModifiedClicksAreLeftToTheBrowser();
testTargetAndDownloadAreLeftToTheBrowser();
testOtherLinksAreLeftToTheBrowser();
testAddressesResolvedLikeTheBrowser();
testRelExternal();
testMountPoint();
testNamedAnchor();
testFragmentOfDataArrivingLater();
testPendingFragmentGivesUp();
testTopFragment();
testSmoothScrollToTheTop();
