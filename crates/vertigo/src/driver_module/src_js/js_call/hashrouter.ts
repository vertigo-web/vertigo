import { Guard, ListItemType } from "../arguments";
import { HashRouter } from "../module/hashrouter";
import { ModuleControllerType } from "../wasm_init";
import { ExportType } from "../wasm_module";

export const initHashrouter = (getWasm: () => ModuleControllerType<ExportType>, hashRouter: HashRouter) => (method: string, args: Array<ListItemType>): number => {
    if (method === 'push') {
        const [new_hash, ...rest] = args;

        if (Guard.isString(new_hash) && rest.length === 0) {
            hashRouter.push(new_hash);
            return 0;
        }

        console.error('js-call -> module -> hashrouter -> push: incorrect parameters', args);
        return 0;
    }

    if (method === 'get') {
        if (args.length === 0) {
            const hash = hashRouter.get();

            const params = getWasm().newList();
            params.push_string(hash);
            return params.saveToBuffer();
        }

        console.error('js-call -> module -> hashrouter -> get: incorrect parameters', args);
        return 0;
    }

    console.error('js-call -> module -> hashrouter: incorrect parameters', args);
    return 0;
};
