class Cookies {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    get = (cname_ptr, cname_len) => {
        const wasm = this.getWasm();
        const cname = wasm.decodeText(cname_ptr, cname_len);
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
                return wasm.pushString(decodeURIComponent(cookieValue));
            }
        }
        wasm.pushString("");
    };
    set = (cname_ptr, cname_len, cvalue_ptr, cvalue_len, expires_in) => {
        const wasm = this.getWasm();
        const cname = wasm.decodeText(cname_ptr, cname_len);
        const cvalue = wasm.decodeText(cvalue_ptr, cvalue_len);
        const cvalueEncoded = cvalue == null ? "" : encodeURIComponent(cvalue);
        const d = new Date();
        d.setTime(d.getTime() + (Number(expires_in) * 1000));
        let expires = "expires=" + d.toUTCString();
        document.cookie = cname + "=" + cvalueEncoded + ";" + expires + ";path=/;samesite=strict";
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
                this.getWasm().pushString(event.key);
                this.getWasm().pushString(event.code);
                const stopPropagate = this.getWasm().exports.dom_keydown(id === undefined ? 0n : id, 
                // event.key,
                // event.code,
                event.altKey === true ? 1 : 0, event.ctrlKey === true ? 1 : 0, event.shiftKey === true ? 1 : 0, event.metaKey === true ? 1 : 0);
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
                    if (target instanceof HTMLInputElement) {
                        this.getWasm().pushString(target.value);
                        this.getWasm().exports.dom_oninput(id);
                        return;
                    }
                    if (target instanceof HTMLTextAreaElement) {
                        this.getWasm().pushString(target.value);
                        this.getWasm().exports.dom_oninput(id);
                        return;
                    }
                    return;
                }
            }
            console.warn('mouseover ignore', target);
        }, false);
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
            while (true) {
                const firstChild = node.firstChild;
                if (firstChild) {
                    new_node.appendChild(firstChild);
                }
                else {
                    this.all.delete(node);
                    this.all.set(new_node, id);
                    this.nodes.set(id, new_node);
                    return;
                }
            }
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
    dom_bulk_update = (value_ptr, value_len) => {
        const value = this.getWasm().decodeText(value_ptr, value_len);
        try {
            const commands = JSON.parse(value);
            for (const command of commands) {
                this.bulk_update_command(command);
            }
        }
        catch (error) {
            console.warn('buil_update - check in: https://jsonformatter.curiousconcept.com/');
            console.warn('bulk_update - param', value);
            console.error('bulk_update - incorrectly json data', error);
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
            this.set_attribute(BigInt(command.id), command.key, command.value);
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
    websocket_register_callback = (host_ptr, //string,
    host_len, callback_id) => {
        const wasm = this.getWasm();
        const host = wasm.decodeText(host_ptr, host_len);
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
                wasm.pushString(message.message);
                wasm.exports.websocket_callback_message(callback_id);
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
    websocket_send_message = (callback_id, message_ptr, //string,
    message_len) => {
        const message = this.getWasm().decodeText(message_ptr, message_len);
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
    fetch_send_request = (request_id, method_ptr, method_len, url_ptr, url_len, headers_ptr, headers_len, body_ptr, body_len) => {
        const wasm = this.getWasm();
        const method = wasm.decodeText(method_ptr, method_len);
        const url = wasm.decodeText(url_ptr, url_len);
        const headers = wasm.decodeText(headers_ptr, headers_len);
        const body = wasm.decodeTextNull(body_ptr, body_len);
        const headers_record = JSON.parse(headers);
        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) => response.text()
            .then((responseText) => {
            wasm.pushString(responseText);
            wasm.exports.fetch_callback(request_id, 1, response.status);
        })
            .catch((err) => {
            console.error('fetch error (2)', err);
            const responseMessage = new String(err).toString();
            wasm.pushString(responseMessage);
            wasm.exports.fetch_callback(request_id, 0, response.status);
        }))
            .catch((err) => {
            console.error('fetch error (1)', err);
            const responseMessage = new String(err).toString();
            wasm.pushString(responseMessage);
            wasm.exports.fetch_callback(request_id, 0, 0);
        });
    };
}

class HashRouter {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
        window.addEventListener("hashchange", () => {
            this.hashrouter_get_hash_location();
            this.getWasm().exports.hashrouter_hashchange_callback();
        }, false);
    }
    hashrouter_get_hash_location = () => {
        const currentHash = location.hash.substr(1);
        this.getWasm().pushString(currentHash);
    };
    hashrouter_push_hash_location = (/*new_hash: string*/ new_hash_ptr, new_hash_length) => {
        location.hash = this.getWasm().decodeText(new_hash_ptr, new_hash_length);
    };
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
    const decodeTextNull = (ptr, length) => {
        if (length === 0n) {
            return null;
        }
        const m = getUint8Memory().subarray(Number(ptr), Number(ptr) + Number(length));
        var decoder = new TextDecoder("utf-8");
        return decoder.decode(m.slice(0, Number(length)));
    };
    const decodeText = (ptr, length) => {
        if (length === 0n) {
            return '';
        }
        const m = getUint8Memory().subarray(Number(ptr), Number(ptr) + Number(length));
        var decoder = new TextDecoder("utf-8");
        return decoder.decode(m.slice(0, Number(length)));
    };
    //@ts-expect-error
    const exports = module_instance.instance.exports;
    const cachedTextEncoder = new TextEncoder();
    const pushString = (arg) => {
        if (arg.length === 0) {
            exports.alloc_empty_string();
            return;
        }
        const buf = cachedTextEncoder.encode(arg);
        const ptr = Number(exports.alloc(BigInt(buf.length)));
        getUint8Memory().subarray(ptr, ptr + buf.length).set(buf);
    };
    return {
        exports,
        decodeText,
        decodeTextNull,
        pushString
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
        const cookies = new Cookies(getWasm);
        const interval = new Interval(getWasm);
        const hashRouter = new HashRouter(getWasm);
        const fetchModule = new Fetch(getWasm);
        const websocket = new DriverWebsocket(getWasm);
        const dom = new DriverDom(getWasm);
        const console_error_1 = (arg1_ptr, arg1_len) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            console.error(arg1);
        };
        const console_debug_4 = (arg1_ptr, arg1_len, arg2_ptr, arg2_len, arg3_ptr, arg3_len, arg4_ptr, arg4_len) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.debug(arg1, arg2, arg3, arg4);
        };
        const console_log_4 = (arg1_ptr, arg1_len, arg2_ptr, arg2_len, arg3_ptr, arg3_len, arg4_ptr, arg4_len) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.log(arg1, arg2, arg3, arg4);
        };
        const console_info_4 = (arg1_ptr, arg1_len, arg2_ptr, arg2_len, arg3_ptr, arg3_len, arg4_ptr, arg4_len) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.info(arg1, arg2, arg3, arg4);
        };
        const console_warn_4 = (arg1_ptr, arg1_len, arg2_ptr, arg2_len, arg3_ptr, arg3_len, arg4_ptr, arg4_len) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.warn(arg1, arg2, arg3, arg4);
        };
        const console_error_4 = (arg1_ptr, arg1_len, arg2_ptr, arg2_len, arg3_ptr, arg3_len, arg4_ptr, arg4_len) => {
            const wasm = getWasm();
            const arg1 = wasm.decodeText(arg1_ptr, arg1_len);
            const arg2 = wasm.decodeText(arg2_ptr, arg2_len);
            const arg3 = wasm.decodeText(arg3_ptr, arg3_len);
            const arg4 = wasm.decodeText(arg4_ptr, arg4_len);
            console.error(arg1, arg2, arg3, arg4);
        };
        wasmModule = await wasmInit(wasmBinPath, {
            mod: {
                console_error_1,
                console_debug_4,
                console_log_4,
                console_info_4,
                console_warn_4,
                console_error_4,
                cookie_get: cookies.get,
                cookie_set: cookies.set,
                interval_set: interval.interval_set,
                interval_clear: interval.interval_clear,
                timeout_set: interval.timeout_set,
                timeout_clear: interval.timeout_clear,
                instant_now,
                hashrouter_get_hash_location: hashRouter.hashrouter_get_hash_location,
                hashrouter_push_hash_location: hashRouter.hashrouter_push_hash_location,
                fetch_send_request: fetchModule.fetch_send_request,
                websocket_register_callback: websocket.websocket_register_callback,
                websocket_unregister_callback: websocket.websocket_unregister_callback,
                websocket_send_message: websocket.websocket_send_message,
                dom_bulk_update: dom.dom_bulk_update,
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
