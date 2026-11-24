/// What of an element the focus goes back to: an `HTMLElement` or an `SVGElement`.
export type Focusable = Pick<HTMLElement, 'focus' | 'isConnected'>;

/// What of the page tells where the focus is: `document`.
export type FocusPage = {
    /// The element with the focus, the body or `null` when it's on no element
    readonly activeElement: object | null;
    readonly body: object | null;
};

const isFocusable = (element: object): element is Focusable =>
    'focus' in element && typeof element.focus === 'function' && 'isConnected' in element;

/// The elements that had the focus, each given it back once what took it is gone, see
/// `Driver::remember_focus`.
export class FocusReturns {
    private readonly remembered: Map<number, Focusable> = new Map();

    /// `later` runs what it gets once the browser is done with the event the call asking for it
    /// came from, as a task does: after the changes to the page the call made, which the wasm
    /// sends before it returns, and after the focus moved by a Tab or a click. Not a microtask:
    /// one queued in a `blur` listener runs with the focus on no element yet, and moving it then
    /// keeps the browser from giving it to the element it was going to.
    constructor(
        private readonly page: () => FocusPage,
        private readonly later: (run: () => void) => void = (run) => { setTimeout(run, 0); },
    ) {}

    /// Remembers the element with the focus as `id`, nothing when the focus is on no element.
    public remember = (id: number) => {
        const { activeElement, body } = this.page();
        if (activeElement !== null && activeElement !== body && isFocusable(activeElement)) {
            this.remembered.set(id, activeElement);
        }
    }

    /// Gives the focus back to the element remembered as `id`, if the focus got lost meanwhile:
    /// it's on no element, as when the element that had it was taken off the page. The element
    /// remembered has to be on the page still.
    public giveBack = (id: number) => {
        const element = this.remembered.get(id);
        this.remembered.delete(id);
        if (element === undefined) {
            return;
        }

        this.later(() => {
            const { activeElement, body } = this.page();
            const lost = activeElement === null
                || activeElement === body
                || (isFocusable(activeElement) && !activeElement.isConnected);

            if (lost && element.isConnected) {
                // Without scrolling the page to it, as a `<dialog>` closed gives the focus back
                element.focus({ preventScroll: true });
            }
        });
    }
}
