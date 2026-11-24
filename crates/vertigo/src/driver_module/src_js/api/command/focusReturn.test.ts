// The focus given back to the element that had it, once what took it is off the page - and not
// when it went to another element meanwhile.

import { FocusPage, FocusReturns } from "./focusReturn";

const assert = (condition: boolean, message: string) => {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        throw new Error(`Assertion failed: ${message}`);
    }
};

/// A page with the focus on `activeElement`, the body when it's on no element
class Page implements FocusPage {
    public readonly body: object = {};
    public activeElement: object | null = this.body;
}

type Element = {
    isConnected: boolean,
    focus: (options?: FocusOptions) => void,
    /// What it was focused with last
    focusOptions?: FocusOptions,
};

/// An element on `page`, which takes the focus when focused
const element = (page: Page): Element => {
    const it: Element = {
        isConnected: true,
        focus: (options) => {
            page.activeElement = it;
            it.focusOptions = options;
        },
    };
    return it;
};

/// Takes `element` off `page`, which moves the focus to no element if it had it, as a browser does
const remove = (page: Page, element: Element) => {
    element.isConnected = false;
    if (page.activeElement === element) {
        page.activeElement = page.body;
    }
};

/// What `FocusReturns` runs after the changes to the page, run when the test makes them
const queued: Array<() => void> = [];
const afterChanges = () => {
    for (const run of queued.splice(0)) {
        run();
    }
};

const page = new Page();
const returns = new FocusReturns(() => page, (run) => { queued.push(run); });

// A dialog opened from a button gives the focus back to it when it's closed
{
    const button = element(page);
    button.focus();
    returns.remember(1);
    const close = element(page);
    close.focus();

    returns.giveBack(1);
    assert(page.activeElement === close, 'the focus goes back only after the changes to the page');
    remove(page, close);
    afterChanges();
    assert(page.activeElement === button, 'the focus lost with the dialog goes back to its button');
    assert(button.focusOptions?.preventScroll === true, 'without scrolling the page to it');
}

// A popup closed by a click on a field beside it leaves the focus on the field
{
    const button = element(page);
    button.focus();
    returns.remember(2);
    const field = element(page);
    field.focus();

    returns.giveBack(2);
    afterChanges();
    assert(page.activeElement === field, 'the focus on another element stays there');
}

// The button is gone with the dialog
{
    const button = element(page);
    button.focus();
    returns.remember(3);
    const close = element(page);
    close.focus();

    returns.giveBack(3);
    remove(page, close);
    remove(page, button);
    afterChanges();
    assert(page.activeElement === page.body, 'the focus goes to no element that is off the page');
}

// Nothing had the focus
{
    page.activeElement = page.body;
    returns.remember(4);
    returns.giveBack(4);
    assert(queued.length === 0, 'with no element remembered nothing is given the focus');
    returns.giveBack(4);
    returns.giveBack(404);
    assert(queued.length === 0, 'giving back what is not remembered does nothing');
}

// A browser telling the element taken off the page still has the focus
{
    const button = element(page);
    button.focus();
    returns.remember(5);
    const close = element(page);
    close.focus();

    returns.giveBack(5);
    close.isConnected = false;
    afterChanges();
    assert(page.activeElement === button, 'the focus on an element off the page counts as lost');
}

// A dialog opened from a dialog, closed one after the other
{
    const settings = element(page);
    settings.focus();
    returns.remember(6);
    const confirm = element(page);
    confirm.focus();
    returns.remember(7);
    const yes = element(page);
    yes.focus();

    returns.giveBack(7);
    remove(page, yes);
    afterChanges();
    assert(page.activeElement === confirm, 'the dialog on top gives the focus to the one under it');

    returns.giveBack(6);
    remove(page, confirm);
    afterChanges();
    assert(page.activeElement === settings, 'then the one under gives it back to its button');
}

// A popup closed in its `blur` listener by a Tab, with `later` as in the browser. The browser
// takes the focus off the element before the listener, and gives it to the next one after it,
// once the microtasks queued in it ran - unless one of them moved the focus meanwhile.
const tabbedAwayFromAPopup = async () => {
    const blurPage = new Page();
    const blurReturns = new FocusReturns(() => blurPage);
    const opener = element(blurPage);
    opener.focus();
    blurReturns.remember(8);
    const item = element(blurPage);
    item.focus();
    const next = element(blurPage);

    // The focus off the item, then the listener: the popup goes with the item
    blurPage.activeElement = blurPage.body;
    blurReturns.giveBack(8);
    item.isConnected = false;
    // The browser after the listener
    queueMicrotask(() => {
        if (blurPage.activeElement === blurPage.body) {
            next.focus();
        }
    });

    await new Promise((resolve) => setTimeout(resolve, 0));
    assert(blurPage.activeElement === next, 'the focus Tab moved on from a popup closed on blur stays there');
};

void tabbedAwayFromAPopup();
