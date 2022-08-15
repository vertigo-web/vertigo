import { Guard, JsValueType } from "../arguments";
import { DriverDom } from "../module/dom/dom";

export const initDom = (dom: DriverDom) => (method: string, args: Array<JsValueType>): number => {
    if (method === 'dom_bulk_update') {
        const [value, ...rest] = args;

        if (Guard.isString(value) && rest.length === 0) {
            dom.dom_bulk_update(value);
            return 0;
        }

        console.error('js-call -> module -> dom -> dom_bulk_update: incorrect parameters', args);
        return 0;
    }

    console.error('js-call -> module -> dom: incorrect parameters', args);
    return 0;
};
