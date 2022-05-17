import { ListItemType } from "../arguments";

export const consoleLog = (method: string, args: ListItemType[]): number => {
    if (method === 'debug' || method === 'info' || method === 'log' || method === 'warn' || method === 'error') {
        console[method](...args);
        return 0;
    }

    console.error('js-call -> module -> consoleLog: incorrect parameters', args);
    return 0;    
};
