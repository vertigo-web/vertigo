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
    scrolledTo = false;
    scrollIntoView() { this.scrolledTo = true; }
}

/// Elements of the page, by id.
const elements: Map<string, MockTarget> = new Map();
let scrolledToTop = false;

(globalThis as any).document = {
    baseURI: 'https://app.test/post/5',
    getElementById: (id: string) => elements.get(id) ?? null,
};
(globalThis as any).window = {
    location: new URL('https://app.test/post/5'),
    scrollTo: () => { scrolledToTop = true; },
};
(globalThis as any).MouseEvent = MockMouseEvent;

import { hydrateLink } from "./injects";

// --- TEST RUNNER ---
function assert(condition: boolean, message: string) {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
}

/// Clicks a link built with `attributes`. `render` runs when the app is told the new address -
/// the moment it renders the new page.
function click(attributes: Record<string, string>, event: Partial<MouseEvent> = {}, render = () => { }) {
    elements.clear();
    scrolledToTop = false;

    const link = new MockLink(attributes);
    const pushed: Array<string> = [];
    const appLocation = {
        set: (_target: string, _mode: string, href: string) => {
            pushed.push(href);
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

    assert(comment.scrolledTo, "scrolled to the element named in the fragment");
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
    assert(heading.scrolledTo, "found by the decoded id");

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

testPlainLinkGoesToTop();
testFragmentOfTheNewPage();
testFragmentWithoutElement();
testEncodedFragment();
testModifiedClicksAreLeftToTheBrowser();
testTargetAndDownloadAreLeftToTheBrowser();
testOtherLinksAreLeftToTheBrowser();
testAddressesResolvedLikeTheBrowser();
