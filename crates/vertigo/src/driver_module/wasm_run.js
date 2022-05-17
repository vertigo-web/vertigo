class Cookies {
    get = (cname) => {
        for (const cookie of document.cookie.split(';')) {
            if (cookie === "")
                continue;
            const cookieChunk = cookie.trim().split('=');
            if (cookieChunk.length !== 2) {
                console.warn(`Cookies.get: Incorrect number of cookieChunk => ${cookieChunk.length} in ${cookie}`);
                continue;
            }
            const cookieName = cookieChunk[0];
            const cookieValue = cookieChunk[1];
            if (cookieName === undefined || cookieValue === undefined) {
                console.warn(`Cookies.get: Broken cookie part => ${cookie}`);
                continue;
            }
            if (cookieName === cname) {
                return decodeURIComponent(cookieValue);
            }
        }
        return '';
    };
    set = (cname, cvalue, expires_in) => {
        const cvalueEncoded = cvalue == null ? "" : encodeURIComponent(cvalue);
        const d = new Date();
        d.setTime(d.getTime() + (Number(expires_in) * 1000));
        let expires = "expires=" + d.toUTCString();
        document.cookie = `${cname}=${cvalueEncoded};${expires};path=/;samesite=strict"`;
    };
}

class MapNodes {
    data;
    constructor() {
        this.data = new Map();
    }
    set(key, value) {
        this.data.set(key, value);
    }
    getItem(key) {
        return this.data.get(key);
    }
    mustGetItem(key) {
        const item = this.data.get(key);
        if (item === undefined) {
            throw Error(`item not found=${key}`);
        }
        return item;
    }
    get(label, key, callback) {
        const item = this.data.get(key);
        if (item === undefined) {
            console.error(`${label}->get: Item id not found = ${key}`);
        }
        else {
            callback(item);
        }
    }
    get2(label, key1, key2, callback) {
        const node1 = this.data.get(key1);
        const node2 = this.data.get(key2);
        if (node1 === undefined) {
            console.error(`${label}->get: Item id not found = ${key1}`);
            return;
        }
        if (node2 === undefined) {
            console.error(`${label}->get: Item id not found = ${key2}`);
            return;
        }
        callback(node1, node2);
    }
    delete(label, key, callback) {
        const item = this.data.get(key);
        this.data.delete(key);
        if (item === undefined) {
            console.error(`${label}->delete: Item id not found = ${key}`);
        }
        else {
            this.data.delete(key);
            callback(item);
        }
    }
}

const createElement = (name) => {
    if (name == "path" || name == "svg") {
        return document.createElementNS("http://www.w3.org/2000/svg", name);
    }
    else {
        return document.createElement(name);
    }
};
const assertNeverCommand = (data) => {
    console.error(data);
    throw Error('unknown command');
};
class DriverDom {
    getWasm;
    nodes;
    texts;
    all;
    constructor(getWasm) {
        this.getWasm = getWasm;
        this.nodes = new MapNodes();
        this.texts = new MapNodes();
        this.all = new Map();
        document.addEventListener('mousedown', (event) => {
            const target = event.target;
            if (target instanceof Element) {
                const id = this.all.get(target);
                if (id !== undefined) {
                    this.getWasm().exports.dom_mousedown(id);
                    return;
                }
            }
            console.warn('mousedown ignore', target);
        }, false);
        document.addEventListener('mouseover', (event) => {
            const target = event.target;
            if (target instanceof Element) {
                const id = this.all.get(target);
                if (id === undefined) {
                    this.getWasm().exports.dom_mouseover(0n);
                    return;
                }
                this.getWasm().exports.dom_mouseover(id);
                return;
            }
            console.warn('mouseover ignore', target);
        }, false);
        document.addEventListener('keydown', (event) => {
            const target = event.target;
            if (target instanceof Element && event instanceof KeyboardEvent) {
                const id = this.all.get(target);
                const new_params = this.getWasm().newList();
                if (id === undefined) {
                    new_params.push_null();
                }
                else {
                    new_params.push_u64(id);
                }
                new_params.push_string(event.key);
                new_params.push_string(event.code);
                new_params.push_bool(event.altKey);
                new_params.push_bool(event.ctrlKey);
                new_params.push_bool(event.shiftKey);
                new_params.push_bool(event.metaKey);
                const new_params_id = new_params.freeze();
                const stopPropagate = this.getWasm().exports.dom_keydown(new_params_id);
                if (stopPropagate > 0) {
                    event.preventDefault();
                    event.stopPropagation();
                }
                return;
            }
            console.warn('keydown ignore', target);
        }, false);
        document.addEventListener('input', (event) => {
            const target = event.target;
            if (target instanceof Element) {
                const id = this.all.get(target);
                if (id !== undefined) {
                    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
                        const new_params = this.getWasm().newList();
                        new_params.push_u64(id);
                        new_params.push_string(target.value);
                        const new_params_id = new_params.freeze();
                        this.getWasm().exports.dom_oninput(new_params_id);
                        return;
                    }
                    console.warn('input ignore', target);
                    return;
                }
            }
            console.warn('input ignore', target);
        }, false);
        document.addEventListener('dragover', (ev) => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
        document.addEventListener('drop', (event) => {
            event.preventDefault();
            const dom_id = this.getIdByTarget(event.target);
            if (dom_id === null) {
                console.warn('drop ignore', event.target);
                return;
            }
            if (event.dataTransfer === null) {
                console.error('dom -> drop -> dataTransfer null');
            }
            else {
                const files = [];
                for (let i = 0; i < event.dataTransfer.items.length; i++) {
                    const item = event.dataTransfer.items[i];
                    if (item === undefined) {
                        console.error('dom -> drop -> item - undefined');
                    }
                    else {
                        const file = item.getAsFile();
                        if (file === null) {
                            console.error(`dom -> drop -> index:${i} -> It's not a file`);
                        }
                        else {
                            files.push(file
                                .arrayBuffer()
                                .then((data) => ({
                                name: file.name,
                                data: new Uint8Array(data),
                            })));
                        }
                    }
                }
                if (files.length) {
                    Promise.all(files).then((files) => {
                        const params = this.getWasm().newList();
                        params.push_u64(dom_id);
                        params.push_list((params_files) => {
                            for (const file of files) {
                                params_files.push_list((params_details) => {
                                    params_details.push_string(file.name);
                                    params_details.push_buffer(file.data);
                                });
                            }
                        });
                        this.getWasm().exports.dom_ondropfile(params.freeze());
                    });
                }
                else {
                    console.error('No files to send');
                }
            }
        }, false);
    }
    getIdByTarget(target) {
        if (target instanceof Element) {
            const id = this.all.get(target);
            return id ?? null;
        }
        return null;
    }
    mount_node(root_id) {
        this.nodes.get("append_to_body", root_id, (root) => {
            document.body.appendChild(root);
        });
    }
    create_node(id, name) {
        const node = createElement(name);
        this.nodes.set(id, node);
        this.all.set(node, id);
    }
    rename_node(id, name) {
        this.nodes.get("rename_node", id, (node) => {
            const new_node = createElement(name);
            while (node.firstChild) {
                new_node.appendChild(node.firstChild);
            }
            if (node.parentElement !== null) {
                node.parentElement.insertBefore(new_node, node);
                node.parentElement.removeChild(node);
            }
            this.all.delete(node);
            this.all.set(new_node, id);
            this.nodes.set(id, new_node);
        });
    }
    set_attribute(id, name, value) {
        this.nodes.get("set_attribute", id, (node) => {
            node.setAttribute(name, value);
            if (name == "value") {
                if (node instanceof HTMLInputElement) {
                    node.value = value;
                    return;
                }
                if (node instanceof HTMLTextAreaElement) {
                    node.value = value;
                    node.defaultValue = value;
                    return;
                }
            }
        });
    }
    remove_attribute(id, name) {
        this.nodes.get("remove_attribute", id, (node) => {
            node.removeAttribute(name);
        });
    }
    remove_node(id) {
        this.nodes.delete("remove_node", id, (node) => {
            this.all.delete(node);
            const parent = node.parentElement;
            if (parent !== null) {
                parent.removeChild(node);
            }
        });
    }
    create_text(id, value) {
        const text = document.createTextNode(value);
        this.texts.set(id, text);
        this.all.set(text, id);
    }
    remove_text(id) {
        this.texts.delete("remove_node", id, (text) => {
            this.all.delete(text);
            const parent = text.parentElement;
            if (parent !== null) {
                parent.removeChild(text);
            }
        });
    }
    update_text(id, value) {
        this.texts.get("set_attribute", id, (text) => {
            text.textContent = value;
        });
    }
    get_node(label, id, callback) {
        const node = this.nodes.getItem(id);
        if (node !== undefined) {
            callback(node);
            return;
        }
        const text = this.texts.getItem(id);
        if (text !== undefined) {
            callback(text);
            return;
        }
        console.error(`${label}->get_node: Item id not found = ${id}`);
        return;
    }
    insert_before(parent, child, ref_id) {
        this.nodes.get("insert_before", parent, (parentNode) => {
            this.get_node("insert_before child", child, (childNode) => {
                if (ref_id === null || ref_id === undefined) {
                    parentNode.insertBefore(childNode, null);
                }
                else {
                    this.get_node('insert_before ref', ref_id, (ref_node) => {
                        parentNode.insertBefore(childNode, ref_node);
                    });
                }
            });
        });
    }
    insert_css(selector, value) {
        const style = document.createElement('style');
        const content = document.createTextNode(`${selector} { ${value} }`);
        style.appendChild(content);
        document.head.appendChild(style);
    }
    dom_bulk_update = (value) => {
        const setFocus = new Set();
        try {
            const commands = JSON.parse(value);
            for (const command of commands) {
                this.bulk_update_command(command);
                if (command.type === 'set_attr' && command.name.toLocaleLowerCase() === 'autofocus') {
                    setFocus.add(command.id);
                }
                else if (command.type === 'remove_attr' && command.name.toLocaleLowerCase() === 'autofocus') {
                    setFocus.delete(command.id);
                }
            }
        }
        catch (error) {
            console.warn('buil_update - check in: https://jsonformatter.curiousconcept.com/');
            console.warn('bulk_update - param', value);
            console.error('bulk_update - incorrectly json data', error);
        }
        if (setFocus.size > 0) {
            setTimeout(() => {
                for (const id of setFocus) {
                    this.nodes.get(`set focus ${id}`, BigInt(id), (node) => {
                        if (node instanceof HTMLElement) {
                            node.focus();
                        }
                        else {
                            console.error('setfocus: HTMLElement expected');
                        }
                    });
                }
            }, 0);
        }
    };
    bulk_update_command(command) {
        if (command.type === 'remove_node') {
            this.remove_node(BigInt(command.id));
            return;
        }
        if (command.type === 'insert_before') {
            this.insert_before(BigInt(command.parent), BigInt(command.child), command.ref_id === null ? null : BigInt(command.ref_id));
            return;
        }
        if (command.type === 'mount_node') {
            this.mount_node(BigInt(command.id));
            return;
        }
        if (command.type === 'create_node') {
            this.create_node(BigInt(command.id), command.name);
            return;
        }
        if (command.type === 'rename_node') {
            this.rename_node(BigInt(command.id), command.new_name);
            return;
        }
        if (command.type === 'create_text') {
            this.create_text(BigInt(command.id), command.value);
            return;
        }
        if (command.type === 'update_text') {
            this.update_text(BigInt(command.id), command.value);
            return;
        }
        if (command.type === 'set_attr') {
            this.set_attribute(BigInt(command.id), command.name, command.value);
            return;
        }
        if (command.type === 'remove_attr') {
            this.remove_attribute(BigInt(command.id), command.name);
            return;
        }
        if (command.type === 'remove_text') {
            this.remove_text(BigInt(command.id));
            return;
        }
        if (command.type === 'insert_css') {
            this.insert_css(command.selector, command.value);
            return;
        }
        return assertNeverCommand(command);
    }
    dom_get_bounding_client_rect_x = (node_id) => {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().x;
    };
    dom_get_bounding_client_rect_y = (node_id) => {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().y;
    };
    dom_get_bounding_client_rect_width = (node_id) => {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().width;
    };
    dom_get_bounding_client_rect_height = (node_id) => {
        return this.nodes.mustGetItem(node_id).getBoundingClientRect().height;
    };
    dom_scroll_top = (node_id) => {
        return this.nodes.mustGetItem(node_id).scrollTop;
    };
    dom_set_scroll_top = (node_id, value) => {
        this.nodes.mustGetItem(node_id).scrollTop = value;
    };
    dom_scroll_left = (node_id) => {
        return this.nodes.mustGetItem(node_id).scrollLeft;
    };
    dom_set_scroll_left = (node_id, value) => {
        return this.nodes.mustGetItem(node_id).scrollLeft = value;
    };
    dom_scroll_width = (node_id) => {
        return this.nodes.mustGetItem(node_id).scrollWidth;
    };
    dom_scroll_height = (node_id) => {
        return this.nodes.mustGetItem(node_id).scrollHeight;
    };
}

class EventEmmiter {
    events;
    constructor() {
        this.events = new Set();
    }
    on(callback) {
        let isActive = true;
        const onExec = (param) => {
            if (isActive) {
                callback(param);
            }
        };
        this.events.add(onExec);
        return () => {
            isActive = false;
            this.events.delete(onExec);
        };
    }
    trigger(param) {
        const eventsCopy = Array.from(this.events.values());
        for (const itemCallbackToRun of eventsCopy) {
            try {
                itemCallbackToRun(param);
            }
            catch (err) {
                console.error(err);
            }
        }
    }
    get size() {
        return this.events.size;
    }
}

const createPromiseValue = () => {
    let resolve = null;
    let reject = null;
    const promise = new Promise((localResolve, localReject) => {
        resolve = localResolve;
        reject = localReject;
    });
    if (resolve === null) {
        throw Error('createPromiseValue - resolve is null');
    }
    if (reject === null) {
        throw Error('createPromiseValue - reject is null');
    }
    const promiseValue = {
        resolve,
        reject,
    };
    return [promiseValue, promise];
};
class PromiseBoxRace {
    promiseResolveReject = null;
    promise;
    constructor() {
        const [promiseResolveReject, promise] = createPromiseValue();
        this.promiseResolveReject = promiseResolveReject;
        this.promise = promise;
    }
    resolve = (value) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;
        if (promiseResolveReject === null) {
            return;
        }
        promiseResolveReject.resolve(value);
    };
    reject = (err) => {
        const promiseResolveReject = this.promiseResolveReject;
        this.promiseResolveReject = null;
        if (promiseResolveReject === null) {
            return;
        }
        promiseResolveReject.reject(err);
    };
    isFulfilled = () => {
        return this.promiseResolveReject === null;
    };
}

const timeout = async (timeout) => {
    return new Promise((resolve) => {
        setTimeout(resolve, timeout);
    });
};
const reconnectDelay = async (label, timeout_retry) => {
    console.info(`${label} wait ${timeout_retry}ms`);
    await timeout(timeout_retry);
    console.info(`${label} go forth`);
};
class LogContext {
    host;
    constructor(host) {
        this.host = host;
    }
    formatLog = (message) => `Socket ${this.host} ==> ${message}`;
}
class SocketConnection {
    eventMessage;
    close;
    send;
    constructor(close, send) {
        this.eventMessage = new EventEmmiter();
        this.close = close;
        this.send = send;
    }
    static connect(log, host, timeout) {
        const result = new PromiseBoxRace();
        const done = new PromiseBoxRace();
        const socket = new WebSocket(host);
        let isClose = false;
        console.info(log.formatLog('starting ...'));
        const closeSocket = () => {
            if (isClose) {
                return;
            }
            console.info(log.formatLog('close'));
            isClose = true;
            result.resolve(null);
            done.resolve();
            socket.close();
        };
        const socketConnection = new SocketConnection(closeSocket, (message) => {
            if (isClose) {
                return;
            }
            socket.send(message);
        });
        setTimeout(() => {
            if (result.isFulfilled() === false) {
                console.error(log.formatLog(`timeout (${timeout}ms)`));
                closeSocket();
            }
        }, timeout);
        const onOpen = () => {
            console.info(log.formatLog('open'));
            result.resolve(socketConnection);
        };
        const onError = (error) => {
            console.error(log.formatLog('error'), error);
            closeSocket();
        };
        const onMessage = (event) => {
            if (isClose) {
                return;
            }
            const dataRaw = event.data;
            if (typeof dataRaw === 'string') {
                socketConnection.eventMessage.trigger(dataRaw);
                return;
            }
            console.error(log.formatLog('onMessage - expected string'), dataRaw);
        };
        socket.addEventListener('open', onOpen);
        socket.addEventListener('error', onError);
        socket.addEventListener('close', closeSocket);
        socket.addEventListener('message', onMessage);
        return {
            socket: result.promise,
            done: done.promise
        };
    }
    static startSocket(host, timeout_connection, timeout_retry, onMessage) {
        let isConnect = true;
        let socketConnection = null;
        const log = new LogContext(host);
        (async () => {
            while (isConnect) {
                const openSocketResult = SocketConnection.connect(log, host, timeout_connection);
                const socket = await openSocketResult.socket;
                if (socket === null) {
                    await reconnectDelay(log.formatLog('reconnect after error'), timeout_retry);
                    continue;
                }
                socketConnection = socket;
                onMessage({
                    type: 'socket',
                    socket
                });
                socket.eventMessage.on(message => {
                    onMessage({
                        type: 'message',
                        message
                    });
                });
                await openSocketResult.done;
                onMessage({
                    type: 'close'
                });
                if (!isConnect) {
                    console.info(log.formatLog('disconnect (1)'));
                    return;
                }
                await reconnectDelay(log.formatLog('reconnect after close'), timeout_retry);
            }
            console.info(log.formatLog('disconnect (2)'));
        })().catch((error) => {
            console.error(error);
        });
        return {
            send: (message) => {
                if (socketConnection === null) {
                    console.error('send fail - missing connection', message);
                }
                else {
                    socketConnection.send(message);
                }
            },
            dispose: () => {
                isConnect = false;
                socketConnection?.close();
            }
        };
    }
}

const assertNeverMessage = (data) => {
    console.error(data);
    throw Error('unknown message');
};
class DriverWebsocket {
    getWasm;
    controllerList;
    socket;
    constructor(getWasm) {
        this.getWasm = getWasm;
        this.controllerList = new Map();
        this.socket = new Map();
    }
    websocket_register_callback = (host, callback_id) => {
        const wasm = this.getWasm();
        let controller = SocketConnection.startSocket(host, 5000, //timeout connection 
        3000, //timeout reconnection
        (message) => {
            if (this.controllerList.has(callback_id) === false) {
                return;
            }
            if (message.type === 'socket') {
                this.socket.set(callback_id, message.socket);
                wasm.exports.websocket_callback_socket(callback_id);
                return;
            }
            if (message.type === 'message') {
                const new_params = wasm.newList();
                new_params.push_u32(callback_id);
                new_params.push_string(message.message);
                const new_params_id = new_params.freeze();
                wasm.exports.websocket_callback_message(new_params_id);
                return;
            }
            if (message.type === 'close') {
                wasm.exports.websocket_callback_close(callback_id);
                this.socket.delete(callback_id);
                return;
            }
            return assertNeverMessage(message);
        });
        this.controllerList.set(callback_id, controller);
    };
    websocket_unregister_callback = (callback_id) => {
        const controller = this.controllerList.get(callback_id);
        if (controller === undefined) {
            console.error('Expected controller');
            return;
        }
        controller.dispose();
        this.controllerList.delete(callback_id);
    };
    websocket_send_message = (callback_id, message) => {
        const socket = this.socket.get(callback_id);
        if (socket === undefined) {
            console.error(`Missing socket connection for callback_id=${callback_id}`);
        }
        else {
            socket.send(message);
        }
    };
}

class Fetch {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    fetch_send_request = (request_id, method, url, headers, body) => {
        const wasm = this.getWasm();
        const headers_record = JSON.parse(headers);
        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) => response.text()
            .then((responseText) => {
            const new_params = this.getWasm().newList();
            new_params.push_u32(request_id); //request_id
            new_params.push_bool(true); //ok
            new_params.push_u32(response.status); //http code
            new_params.push_string(responseText); //body
            let params_id = new_params.freeze();
            wasm.exports.fetch_callback(params_id);
        })
            .catch((err) => {
            console.error('fetch error (2)', err);
            const responseMessage = new String(err).toString();
            const new_params = this.getWasm().newList();
            new_params.push_u32(request_id); //request_id
            new_params.push_bool(false); //ok
            new_params.push_u32(response.status); //http code
            new_params.push_string(responseMessage); //body
            let params_id = new_params.freeze();
            wasm.exports.fetch_callback(params_id);
        }))
            .catch((err) => {
            console.error('fetch error (1)', err);
            const responseMessage = new String(err).toString();
            const new_params = this.getWasm().newList();
            new_params.push_u32(request_id); //request_id
            new_params.push_bool(false); //ok
            new_params.push_u32(0); //http code
            new_params.push_string(responseMessage); //body
            let params_id = new_params.freeze();
            wasm.exports.fetch_callback(params_id);
        });
    };
}

class HashRouter {
    constructor(getWasm) {
        window.addEventListener("hashchange", () => {
            const params = getWasm().newList();
            params.push_string(this.get());
            const listId = params.freeze();
            getWasm().exports.hashrouter_hashchange_callback(listId);
        }, false);
    }
    push = (new_hash) => {
        location.hash = new_hash;
    };
    get() {
        return location.hash.substr(1);
    }
}

const instant_now = () => {
    return Date.now();
};

class Interval {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    interval_set = (duration, callback_id) => {
        const timer_id = setInterval(() => {
            this.getWasm().exports.interval_run_callback(callback_id);
        }, Number(duration));
        return timer_id;
    };
    interval_clear = (timer_id) => {
        clearInterval(timer_id);
    };
    timeout_set = (duration, callback_id) => {
        const timeout_id = setTimeout(() => {
            this.getWasm().exports.timeout_run_callback(callback_id);
        }, duration);
        return timeout_id;
    };
    timeout_clear = (timer_id) => {
        clearTimeout(timer_id);
    };
}

///https://javascript.info/arraybuffer-binary-arrays#dataview
class BufferCursor {
    dataView;
    getUint8Memory;
    pointer = 0;
    constructor(dataView, getUint8Memory) {
        this.dataView = dataView;
        this.getUint8Memory = getUint8Memory;
    }
    getByte() {
        const value = this.dataView.getUint8(this.pointer);
        this.pointer += 1;
        return value;
    }
    getU16() {
        const value = this.dataView.getUint16(this.pointer);
        this.pointer += 2;
        return value;
    }
    getU32() {
        const value = this.dataView.getUint32(this.pointer);
        this.pointer += 4;
        return value;
    }
    getI32() {
        const value = this.dataView.getInt32(this.pointer);
        this.pointer += 4;
        return value;
    }
    getU64() {
        const value = this.dataView.getBigUint64(this.pointer);
        this.pointer += 8;
        return value;
    }
    getI64() {
        const value = this.dataView.getBigInt64(this.pointer);
        this.pointer += 8;
        return value;
    }
    getString() {
        const ptr = this.getU32();
        const size = this.getU32();
        const m = this.getUint8Memory().subarray(ptr, ptr + size);
        return decoder.decode(m);
    }
    getBuffer() {
        const ptr = this.getU32();
        const size = this.getU32();
        return this.getUint8Memory().subarray(ptr, ptr + size);
    }
}
const decoder = new TextDecoder("utf-8");
const argumentsDecodeItem = (cursor) => {
    const typeParam = cursor.getByte();
    if (typeParam === 1) {
        return '';
    }
    if (typeParam === 2 || typeParam === 3 || typeParam === 4) {
        return cursor.getString();
    }
    if (typeParam === 5) {
        return cursor.getU32();
    }
    if (typeParam === 6) {
        return cursor.getI32();
    }
    if (typeParam === 7) {
        return cursor.getU64();
    }
    if (typeParam === 8) {
        return cursor.getI64();
    }
    if (typeParam === 9) {
        return true;
    }
    if (typeParam === 10) {
        return false;
    }
    if (typeParam === 11) {
        return null;
    }
    if (typeParam === 12) {
        return undefined;
    }
    if (typeParam === 13) {
        const out = [];
        const listSize = cursor.getU16();
        for (let i = 0; i < listSize; i++) {
            out.push(argumentsDecodeItem(cursor));
        }
        return out;
    }
    if (typeParam === 14) {
        return cursor.getBuffer();
    }
    console.error('typeParam', typeParam);
    throw Error('Nieprawidłowe odgałęzienie');
};
const argumentsDecode = (getUint8Memory, ptr) => {
    try {
        const view = new DataView(getUint8Memory().buffer, ptr);
        const cursor = new BufferCursor(view, getUint8Memory);
        return argumentsDecodeItem(cursor);
    }
    catch (err) {
        console.error(err);
        return [];
    }
};
var Guard;
(function (Guard) {
    Guard.isString = (value) => {
        return typeof value === 'string';
    };
    Guard.isStringOrNull = (value) => {
        return value === null || typeof value === 'string';
    };
    Guard.isNumber = (value) => {
        return typeof value === 'number';
    };
    Guard.isBigInt = (value) => {
        return typeof value === 'bigint';
    };
})(Guard || (Guard = {}));
class ParamListBuilder {
    getUint8Memory;
    exportsModule;
    listId;
    constructor(getUint8Memory, exportsModule) {
        this.getUint8Memory = getUint8Memory;
        this.exportsModule = exportsModule;
        this.listId = exportsModule.arguments_new_list();
    }
    debug() {
        this.exportsModule.arguments_debug(this.listId);
    }
    push_string(value) {
        if (value.length === 0) {
            this.exportsModule.arguments_push_string_empty(this.listId);
        }
        else {
            const encoder = new TextEncoder();
            const buf = encoder.encode(value);
            let ptr = this.exportsModule.arguments_push_string_alloc(this.listId, buf.length);
            this.getUint8Memory().subarray(ptr, ptr + buf.length).set(buf);
        }
    }
    push_buffer(buf) {
        const ptr = this.exportsModule.arguments_push_buffer_alloc(this.listId, buf.length);
        this.getUint8Memory().subarray(ptr, ptr + buf.length).set(buf);
    }
    push_u32(value) {
        this.exportsModule.arguments_push_u32(this.listId, value);
    }
    push_i32(value) {
        this.exportsModule.arguments_push_i32(this.listId, value);
    }
    push_u64(value) {
        this.exportsModule.arguments_push_u64(this.listId, value);
    }
    push_i64(value) {
        this.exportsModule.arguments_push_i64(this.listId, value);
    }
    push_null() {
        this.exportsModule.arguments_push_null(this.listId);
    }
    push_bool(value) {
        if (value) {
            this.exportsModule.arguments_push_true(this.listId);
        }
        else {
            this.exportsModule.arguments_push_false(this.listId);
        }
    }
    push_list(build) {
        const sub_params = new ParamListBuilder(this.getUint8Memory, this.exportsModule);
        build(sub_params);
        this.exportsModule.arguments_push_sublist(this.listId, sub_params.listId);
    }
    freeze() {
        this.exportsModule.arguments_freeze(this.listId);
        return this.listId;
    }
}

const fetchModule = async (wasmBinPath, imports) => {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
        console.info('fetchModule by WebAssembly.instantiateStreaming');
        try {
            const module = await WebAssembly.instantiateStreaming(fetch(wasmBinPath), imports);
            return module;
        }
        catch (err) {
            console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", err);
        }
    }
    console.info('fetchModule by WebAssembly.instantiate');
    const resp = await fetch(wasmBinPath);
    const binary = await resp.arrayBuffer();
    const module_instance = await WebAssembly.instantiate(binary, imports);
    return module_instance;
};
const wasmInit = async (wasmBinPath, imports) => {
    const module_instance = await fetchModule(wasmBinPath, imports);
    let cachegetUint8Memory = new Uint8Array(1);
    const getUint8Memory = () => {
        if (module_instance.instance.exports.memory instanceof WebAssembly.Memory) {
            if (cachegetUint8Memory.buffer !== module_instance.instance.exports.memory.buffer) {
                console.info('getUint8Memory: reallocate the Uint8Array for a new size', module_instance.instance.exports.memory.buffer.byteLength);
                cachegetUint8Memory = new Uint8Array(module_instance.instance.exports.memory.buffer);
            }
            return cachegetUint8Memory;
        }
        else {
            throw Error('Missing memory');
        }
    };
    //@ts-expect-error
    const exports = module_instance.instance.exports;
    const decodeArguments = (ptr) => argumentsDecode(getUint8Memory, ptr);
    const newList = () => new ParamListBuilder(getUint8Memory, exports);
    return {
        exports,
        decodeArguments,
        getUint8Memory,
        newList
    };
};

const consoleLog = (method, args) => {
    if (method === 'debug' || method === 'info' || method === 'log' || method === 'warn' || method === 'error') {
        console[method](...args);
        return 0;
    }
    console.error('js-call -> module -> consoleLog: incorrect parameters', args);
    return 0;
};

const initCookie = (getWasm, cookies) => (method, args) => {
    if (method === 'get') {
        const [name, ...rest] = args;
        if (Guard.isString(name) && rest.length === 0) {
            const value = cookies.get(name);
            const params = getWasm().newList();
            params.push_string(value);
            return params.freeze();
        }
        console.error('js-call -> module -> cookie -> get: incorrect parameters', args);
        return 0;
    }
    if (method === 'set') {
        const [name, value, expires_in, ...rest] = args;
        if (Guard.isString(name) &&
            Guard.isString(value) &&
            Guard.isBigInt(expires_in) &&
            rest.length === 0) {
            cookies.set(name, value, expires_in);
            return 0;
        }
        console.error('js-call -> module -> cookie -> set: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> cookie: incorrect parameters', args);
    return 0;
};

const initDom = (dom) => (method, args) => {
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

const initFetch = (fetch) => (method, args) => {
    if (method === 'send') {
        const [requestId, httpMethod, url, headers, body, ...rest] = args;
        if (Guard.isNumber(requestId) &&
            Guard.isString(httpMethod) &&
            Guard.isString(url) &&
            Guard.isString(headers) &&
            Guard.isStringOrNull(body) &&
            rest.length === 0) {
            fetch.fetch_send_request(requestId, httpMethod, url, headers, body);
            return 0;
        }
        console.error('js-call -> module -> fetch -> send: incorrect parameters', args);
    }
    console.error('js-call -> module -> fetch: incorrect parameters', args);
    return 0;
};

const initHashrouter = (getWasm, hashRouter) => (method, args) => {
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
            return params.freeze();
        }
        console.error('js-call -> module -> hashrouter -> get: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> hashrouter: incorrect parameters', args);
    return 0;
};

const initWebsocketModule = (websocket) => (method, args) => {
    if (method === 'register_callback') {
        const [host, callback_id, ...rest] = args;
        if (Guard.isString(host) &&
            Guard.isNumber(callback_id) &&
            rest.length === 0) {
            websocket.websocket_register_callback(host, callback_id);
            return 0;
        }
        console.error('js-call -> module -> websocket -> register_callback: incorrect parameters', args);
        return 0;
    }
    if (method === 'unregister_callback') {
        const [callback_id, ...rest] = args;
        if (Guard.isNumber(callback_id) &&
            rest.length === 0) {
            websocket.websocket_unregister_callback(callback_id);
            return 0;
        }
        console.error('js-call -> module -> websocket -> unregister_callback: incorrect parameters', args);
        return 0;
    }
    if (method === 'send_message') {
        const [callback_id, message, ...rest] = args;
        if (Guard.isNumber(callback_id) &&
            Guard.isString(message) &&
            rest.length === 0) {
            websocket.websocket_send_message(callback_id, message);
            return 0;
        }
        console.error('js-call -> module -> websocket -> send_message: incorrect parameters', args);
        return 0;
    }
    console.error('js-call -> module -> websocket: incorrect parameters', args);
    return 0;
};

const js_call = (decodeArguments, getWasm, fetch, cookies, dom, hashRouter, websocket) => {
    const modules = {
        consoleLog: consoleLog,
        hashrouter: initHashrouter(getWasm, hashRouter),
        websocket: initWebsocketModule(websocket),
        cookie: initCookie(getWasm, cookies),
        fetch: initFetch(fetch),
        dom: initDom(dom),
    };
    return (ptr) => {
        const args = decodeArguments(ptr);
        // console.info('js_call', arg);        //for debug
        if (Array.isArray(args)) {
            const [modulePrefix, moduleName, newFunction, ...restNew] = args;
            if (modulePrefix === 'module' &&
                Guard.isString(moduleName) &&
                Guard.isString(newFunction)) {
                const toRun = modules[moduleName];
                if (toRun === undefined) {
                    console.error(`js-call: unknown module ${moduleName}`, args);
                    return 0;
                }
                return toRun(newFunction, restNew);
            }
            console.error('js-call: incorrect parameters', args);
            return 0;
        }
        console.error("js-call: List of parameters was expected: ", args);
        return 0;
    };
};

class WasmModule {
    wasm;
    constructor(wasm) {
        this.wasm = wasm;
    }
    start_application() {
        this.wasm.exports.start_application();
    }
    static async create(wasmBinPath) {
        let wasmModule = null;
        const getWasm = () => {
            if (wasmModule === null) {
                throw Error('Wasm is no initialized');
            }
            return wasmModule;
        };
        const cookies = new Cookies();
        const interval = new Interval(getWasm);
        const hashRouter = new HashRouter(getWasm);
        const fetchModule = new Fetch(getWasm);
        const websocket = new DriverWebsocket(getWasm);
        const dom = new DriverDom(getWasm);
        wasmModule = await wasmInit(wasmBinPath, {
            mod: {
                panic_message: (ptr, size) => {
                    const decoder = new TextDecoder("utf-8");
                    const m = getWasm().getUint8Memory().subarray(ptr, ptr + size);
                    const message = decoder.decode(m);
                    console.error('PANIC', message);
                },
                js_call: js_call((ptr) => getWasm().decodeArguments(ptr), getWasm, fetchModule, cookies, dom, hashRouter, websocket),
                interval_set: interval.interval_set,
                interval_clear: interval.interval_clear,
                timeout_set: interval.timeout_set,
                timeout_clear: interval.timeout_clear,
                instant_now,
                dom_get_bounding_client_rect_x: dom.dom_get_bounding_client_rect_x,
                dom_get_bounding_client_rect_y: dom.dom_get_bounding_client_rect_y,
                dom_get_bounding_client_rect_width: dom.dom_get_bounding_client_rect_width,
                dom_get_bounding_client_rect_height: dom.dom_get_bounding_client_rect_height,
                dom_scroll_top: dom.dom_scroll_top,
                dom_set_scroll_top: dom.dom_set_scroll_top,
                dom_scroll_left: dom.dom_scroll_left,
                dom_set_scroll_left: dom.dom_set_scroll_left,
                dom_scroll_width: dom.dom_scroll_width,
                dom_scroll_height: dom.dom_scroll_height,
            }
        });
        return new WasmModule(wasmModule);
    }
}

const runModule = async (wasmBinPath) => {
    console.info(`Wasm module: "${wasmBinPath}" -> start`);
    const wasmModule = await WasmModule.create(wasmBinPath);
    console.info(`Wasm module: "${wasmBinPath}" -> initialized`);
    wasmModule.start_application();
    console.info(`Wasm module: "${wasmBinPath}" -> launched start_application function`);
};

export { runModule };
