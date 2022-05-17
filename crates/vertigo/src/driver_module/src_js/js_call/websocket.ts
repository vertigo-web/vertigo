import { Guard, ListItemType } from "../arguments";
import { DriverWebsocket } from "../module/websocket/websocket";

export const initWebsocketModule = (websocket: DriverWebsocket) => (method: string, args: Array<ListItemType>): number => {
    if (method === 'register_callback') {
        const [host, callback_id, ...rest] = args;

        if (
            Guard.isString(host) &&
            Guard.isNumber(callback_id) &&
            rest.length === 0
        ) {
            websocket.websocket_register_callback(host, callback_id);
            return 0;
        }

        console.error('js-call -> module -> websocket -> register_callback: incorrect parameters', args);
        return 0;
    }

    if (method === 'unregister_callback') {
        const [callback_id, ...rest] = args;

        if (
            Guard.isNumber(callback_id) &&
            rest.length === 0
        ) {
            websocket.websocket_unregister_callback(callback_id);
            return 0;
        }

        console.error('js-call -> module -> websocket -> unregister_callback: incorrect parameters', args);
        return 0;
    }

    if (method === 'send_message') {
        const [callback_id, message, ...rest] = args;

        if (
            Guard.isNumber(callback_id) &&
            Guard.isString(message) && 
            rest.length === 0
        ) {
            websocket.websocket_send_message(callback_id, message);
            return 0;
        }

        console.error('js-call -> module -> websocket -> send_message: incorrect parameters', args);
        return 0;
    }

    console.error('js-call -> module -> websocket: incorrect parameters', args);
    return 0;
};
