///https://javascript.info/arraybuffer-binary-arrays#dataview
const decoder = new TextDecoder("utf-8");
const encoder = new TextEncoder();
class BufferCursor {
    getUint8Memory;
    ptr;
    size;
    dataView;
    pointer = 0;
    constructor(getUint8Memory, ptr, size) {
        this.getUint8Memory = getUint8Memory;
        this.ptr = ptr;
        this.size = size;
        this.getUint8Memory()[3] = 56;
        this.dataView = new DataView(this.getUint8Memory().buffer, this.ptr, this.size);
    }
    getByte() {
        const value = this.dataView.getUint8(this.pointer);
        this.pointer += 1;
        return value;
    }
    setByte(byte) {
        this.dataView.setUint8(this.pointer, byte);
        this.pointer += 1;
    }
    getU16() {
        const value = this.dataView.getUint16(this.pointer);
        this.pointer += 2;
        return value;
    }
    setU16(value) {
        this.dataView.setUint16(this.pointer, value);
        this.pointer += 2;
    }
    getU32() {
        const value = this.dataView.getUint32(this.pointer);
        this.pointer += 4;
        return value;
    }
    setU32(value) {
        this.dataView.setUint32(this.pointer, value);
        this.pointer += 4;
    }
    getI32() {
        const value = this.dataView.getInt32(this.pointer);
        this.pointer += 4;
        return value;
    }
    setI32(value) {
        this.dataView.setInt32(this.pointer, value);
        this.pointer += 4;
    }
    getU64() {
        const value = this.dataView.getBigUint64(this.pointer);
        this.pointer += 8;
        return value;
    }
    setU64(value) {
        this.dataView.setBigUint64(this.pointer, value);
        this.pointer += 8;
    }
    getI64() {
        const value = this.dataView.getBigInt64(this.pointer);
        this.pointer += 8;
        return value;
    }
    setI64(value) {
        this.dataView.setBigInt64(this.pointer, value);
        this.pointer += 8;
    }
    getBuffer() {
        const size = this.getU32();
        const result = this
            .getUint8Memory()
            .subarray(this.ptr + this.pointer, this.ptr + this.pointer + size);
        this.pointer += size;
        return result;
    }
    setBuffer(buffer) {
        const size = buffer.length;
        this.setU32(size);
        const subbugger = this
            .getUint8Memory()
            .subarray(this.ptr + this.pointer, this.ptr + this.pointer + size);
        subbugger.set(buffer);
        this.pointer += size;
    }
    getString() {
        return decoder.decode(this.getBuffer());
    }
    setString(value) {
        const buffer = encoder.encode(value);
        this.setBuffer(buffer);
    }
}
//https://github.com/unsplash/unsplash-js/pull/174
// export type AnyJson = boolean | number | string | null | JsonArray | JsonMap;
// export interface JsonMap { [key: string]: AnyJson }
// export interface JsonArray extends Array<AnyJson> {}
const argumentsDecodeItem = (cursor) => {
    const typeParam = cursor.getByte();
    if (typeParam === 1) {
        return {
            type: 'u32',
            value: cursor.getU32()
        };
    }
    if (typeParam === 2) {
        return {
            type: 'u32',
            value: cursor.getI32()
        };
    }
    if (typeParam === 3) {
        return {
            type: 'u64',
            value: cursor.getU64()
        };
    }
    if (typeParam === 4) {
        return {
            type: 'i64',
            value: cursor.getI64()
        };
    }
    if (typeParam === 5) {
        return true;
    }
    if (typeParam === 6) {
        return false;
    }
    if (typeParam === 7) {
        return null;
    }
    if (typeParam === 8) {
        return undefined;
    }
    if (typeParam === 9) {
        return cursor.getBuffer();
    }
    if (typeParam === 10) {
        return cursor.getString();
    }
    if (typeParam === 11) {
        const out = [];
        const listSize = cursor.getU16();
        for (let i = 0; i < listSize; i++) {
            out.push(argumentsDecodeItem(cursor));
        }
        return out;
    }
    if (typeParam === 12) {
        const out = {};
        const listSize = cursor.getU16();
        for (let i = 0; i < listSize; i++) {
            const key = cursor.getString();
            const value = argumentsDecodeItem(cursor);
            out[key] = value;
        }
        return {
            type: 'object',
            value: out
        };
    }
    console.error('typeParam', typeParam);
    throw Error('Nieprawidłowe odgałęzienie');
};
const argumentsDecode = (getUint8Memory, ptr, size) => {
    try {
        const cursor = new BufferCursor(getUint8Memory, ptr, size);
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
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === 'i32' || value.type === 'u32';
        }
        return false;
    };
    Guard.isBigInt = (value) => {
        if (typeof value === 'object' && value !== null && 'type' in value) {
            return value.type === 'i64' || value.type === 'u64';
        }
        return false;
    };
})(Guard || (Guard = {}));
const assertNever = (_value) => {
    throw Error("assert never");
};
const getStringSize = (value) => {
    return new TextEncoder().encode(value).length;
};
const getSize = (value) => {
    if (value === true ||
        value === false ||
        value === null ||
        value === undefined) {
        return 1;
    }
    if (Guard.isString(value)) {
        return 1 + 4 + getStringSize(value);
    }
    if (Array.isArray(value)) {
        let sum = 1 + 2;
        for (const item of value) {
            sum += getSize(item);
        }
        return sum;
    }
    if (value instanceof Uint8Array) {
        return 1 + 4 + value.length;
    }
    if (value.type === 'i32' || value.type === 'u32') {
        return 5; //1 + 4
    }
    if (value.type === 'i64' || value.type === 'u64') {
        return 9; //1 + 8
    }
    if (value.type === 'object') {
        let sum = 1 + 2;
        for (const [key, propertyValue] of Object.entries(value.value)) {
            sum += getStringSize(key);
            sum += getSize(propertyValue);
        }
        return sum;
    }
    return assertNever();
};
const saveToBufferItem = (value, cursor) => {
    if (value === true) {
        cursor.setByte(5);
        return;
    }
    if (value === false) {
        cursor.setByte(6);
        return;
    }
    if (value === null) {
        cursor.setByte(7);
        return;
    }
    if (value === undefined) {
        cursor.setByte(8);
        return;
    }
    if (value instanceof Uint8Array) {
        cursor.setByte(9);
        cursor.setBuffer(value);
        return;
    }
    if (Guard.isString(value)) {
        cursor.setByte(10);
        cursor.setString(value);
        return;
    }
    if (Array.isArray(value)) {
        cursor.setByte(11);
        cursor.setU16(value.length);
        for (const item of value) {
            saveToBufferItem(item, cursor);
        }
        return;
    }
    if (value.type === 'u32') {
        cursor.setByte(1);
        cursor.setU32(value.value);
        return;
    }
    if (value.type === 'i32') {
        cursor.setByte(2);
        cursor.setI32(value.value);
        return;
    }
    if (value.type === 'u64') {
        cursor.setByte(3);
        cursor.setU64(value.value);
        return;
    }
    if (value.type === 'i64') {
        cursor.setByte(4);
        cursor.setI64(value.value);
        return;
    }
    if (value.type === 'object') {
        const list = [];
        for (const [key, propertyValue] of Object.entries(value.value)) {
            list.push([key, propertyValue]);
        }
        cursor.setByte(12);
        cursor.setU16(list.length);
        for (const [key, propertyValue] of list) {
            cursor.setString(key);
            saveToBufferItem(propertyValue, cursor);
        }
        return;
    }
    return assertNever();
};
const saveToBuffer = (getUint8Memory, alloc, value) => {
    if (value === undefined) {
        return 0;
    }
    const size = getSize(value);
    const ptr = alloc(size);
    const cursor = new BufferCursor(getUint8Memory, ptr, size);
    saveToBufferItem(value, cursor);
    return ptr;
};
const convertFromJsValue = (value) => {
    if (value === true) {
        return true;
    }
    if (value === false) {
        return false;
    }
    if (value === null) {
        return null;
    }
    if (value === undefined) {
        return undefined;
    }
    if (value instanceof Uint8Array) {
        return value;
    }
    if (Guard.isString(value)) {
        return value;
    }
    if (Array.isArray(value)) {
        const newList = [];
        for (const item of value) {
            newList.push(convertFromJsValue(item));
        }
        return newList;
    }
    if (value.type === 'u32' || value.type === 'i32') {
        return value.value;
    }
    if (value.type === 'u64' || value.type === 'i64') {
        return value.value;
    }
    if (value.type === 'object') {
        const result = {};
        for (const [key, propertyValue] of Object.entries(value.value)) {
            result[key] = convertFromJsValue(propertyValue);
        }
        return result;
    }
    return assertNever();
};
const convertToJsValue = (value) => {
    if (typeof value === 'string') {
        return value;
    }
    if (value === true || value === false || value === undefined || value === null) {
        return value;
    }
    if (typeof value === 'number') {
        if (-(2 ** 31) <= value && value < 2 ** 31) {
            return {
                type: 'i32',
                value
            };
        }
        return {
            type: 'i64',
            value: BigInt(value)
        };
    }
    if (typeof value === 'bigint') {
        return {
            type: 'i64',
            value
        };
    }
    console.error('convertToJsValue', value);
    throw Error('TODO');
};

const fetchModule = async (wasmBinPath, imports) => {
    if (typeof WebAssembly.instantiateStreaming === 'function') {
        const stream = fetch(wasmBinPath);
        try {
            const module = await WebAssembly.instantiateStreaming(stream, imports);
            return module;
        }
        catch (err) {
            console.warn("`WebAssembly.instantiateStreaming` failed. This could happen if your server does not serve wasm with `application/wasm` MIME type, but check the original error too. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", err);
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
    const decodeArguments = (ptr, size) => argumentsDecode(getUint8Memory, ptr, size);
    const valueSaveToBuffer = (value) => saveToBuffer(getUint8Memory, exports.alloc, value);
    const wasm_callback = (callback_id, value) => {
        const value_ptr = valueSaveToBuffer(value);
        let result_ptr_and_size = exports.wasm_callback(callback_id, value_ptr);
        if (result_ptr_and_size === 0n) {
            return undefined;
        }
        const size = result_ptr_and_size % (2n ** 32n);
        const ptr = result_ptr_and_size >> 32n;
        if (ptr >= 2n ** 32n) {
            console.error(`Overflow of a variable with a pointer result_ptr_and_size=${result_ptr_and_size}`);
        }
        const response = decodeArguments(Number(ptr), Number(size));
        exports.free(Number(ptr));
        return response;
    };
    return {
        exports,
        decodeArguments,
        getUint8Memory,
        wasm_callback,
        valueSaveToBuffer,
    };
};

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

class Interval {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    interval_set = (duration, callback_id) => {
        const timer_id = setInterval(() => {
            this.getWasm().wasm_callback(callback_id, undefined);
        }, duration);
        return timer_id;
    };
    interval_clear = (timer_id) => {
        clearInterval(timer_id);
    };
    timeout_set = (duration, callback_id) => {
        const timeout_id = setTimeout(() => {
            this.getWasm().wasm_callback(callback_id, undefined);
        }, duration);
        return timeout_id;
    };
    timeout_clear = (timer_id) => {
        clearTimeout(timer_id);
    };
}

class HashRouter {
    getWasm;
    callback;
    constructor(getWasm) {
        this.getWasm = getWasm;
        this.callback = new Map();
    }
    add = (callback_id) => {
        const callback = () => {
            this.getWasm().wasm_callback(callback_id, this.get());
        };
        window.addEventListener("hashchange", callback);
        this.callback.set(callback_id, callback);
    };
    remove = (callback_id) => {
        const callback = this.callback.get(callback_id);
        if (callback === undefined) {
            console.error(`HashRouter - The callback with id is missing = ${callback_id}`);
            return;
        }
        this.callback.delete(callback_id);
        window.removeEventListener('hashchange', callback);
    };
    push = (new_hash) => {
        location.hash = new_hash;
    };
    get() {
        return decodeURIComponent(location.hash.substr(1));
    }
}

class Fetch {
    getWasm;
    constructor(getWasm) {
        this.getWasm = getWasm;
    }
    fetch_send_request = (callback_id, method, url, headers, body) => {
        const wasm = this.getWasm();
        const headers_record = JSON.parse(headers);
        fetch(url, {
            method,
            body,
            headers: Object.keys(headers_record).length === 0 ? undefined : headers_record,
        })
            .then((response) => response.text()
            .then((responseText) => {
            wasm.wasm_callback(callback_id, [
                true,
                { type: 'u32', value: response.status },
                responseText //body
            ]);
        })
            .catch((err) => {
            console.error('fetch error (2)', err);
            const responseMessage = new String(err).toString();
            wasm.wasm_callback(callback_id, [
                false,
                { type: 'u32', value: response.status },
                responseMessage //body
            ]);
        }))
            .catch((err) => {
            console.error('fetch error (1)', err);
            const responseMessage = new String(err).toString();
            wasm.wasm_callback(callback_id, [
                false,
                { type: 'u32', value: 0 },
                responseMessage //body
            ]);
        });
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
                wasm.wasm_callback(callback_id, true);
                return;
            }
            if (message.type === 'message') {
                wasm.wasm_callback(callback_id, message.message);
                return;
            }
            if (message.type === 'close') {
                this.socket.delete(callback_id);
                wasm.wasm_callback(callback_id, false);
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
    callbacks;
    constructor(getWasm) {
        this.getWasm = getWasm;
        this.nodes = new MapNodes();
        this.texts = new MapNodes();
        this.callbacks = new Map();
        document.addEventListener('dragover', (ev) => {
            // console.log('File(s) in drop zone');
            ev.preventDefault();
        });
    }
    debugNodes(...ids) {
        const result = {};
        for (const id of ids) {
            const value = this.nodes.getItem(BigInt(id));
            result[id] = value;
        }
        console.info('debug nodes', result);
    }
    mount_node(root_id) {
        this.nodes.get("append_to_body", root_id, (root) => {
            document.body.appendChild(root);
        });
    }
    create_node(id, name) {
        const node = createElement(name);
        node.setAttribute('data-id', id.toString());
        this.nodes.set(id, node);
    }
    set_attribute(id, name, value) {
        this.nodes.get("set_attribute", id, (node) => {
            if (node instanceof Element) {
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
            }
            else {
                console.error("set_attribute error");
            }
        });
    }
    remove_node(id) {
        this.nodes.delete("remove_node", id, (node) => {
            node.remove();
        });
    }
    create_text(id, value) {
        const text = document.createTextNode(value);
        this.texts.set(id, text);
    }
    remove_text(id) {
        this.texts.delete("remove_node", id, (text) => {
            text.remove();
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
    callback_mousedown(event, callback_id) {
        event.preventDefault();
        this.getWasm().wasm_callback(callback_id, undefined);
    }
    callback_input(event, callback_id) {
        const target = event.target;
        if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement) {
            this.getWasm().wasm_callback(callback_id, target.value);
            return;
        }
        console.warn('event input ignore', target);
    }
    callback_mouseenter(_event, callback_id) {
        // event.preventDefault();
        this.getWasm().wasm_callback(callback_id, undefined);
    }
    callback_mouseleave(_event, callback_id) {
        // event.preventDefault();
        this.getWasm().wasm_callback(callback_id, undefined);
    }
    callback_drop(event, callback_id) {
        event.preventDefault();
        if (event instanceof DragEvent) {
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
                        const params = [];
                        for (const file of files) {
                            params.push([
                                file.name,
                                file.data,
                            ]);
                        }
                        this.getWasm().wasm_callback(callback_id, params);
                    }).catch((error) => {
                        console.error('callback_drop -> promise.all -> ', error);
                    });
                }
                else {
                    console.error('No files to send');
                }
            }
        }
        else {
            console.warn('event drop ignore', event);
        }
    }
    callback_keydown(event, callback_id) {
        if (event instanceof KeyboardEvent) {
            const result = this.getWasm().wasm_callback(callback_id, [
                event.key,
                event.code,
                event.altKey,
                event.ctrlKey,
                event.shiftKey,
                event.metaKey
            ]);
            if (result === true) {
                event.preventDefault();
                event.stopPropagation();
            }
            return;
        }
        console.warn('keydown ignore', event);
    }
    callback_add(id, event_name, callback_id) {
        let callback = (event) => {
            if (event_name === 'mousedown') {
                return this.callback_mousedown(event, callback_id);
            }
            if (event_name === 'input') {
                return this.callback_input(event, callback_id);
            }
            if (event_name === 'mouseenter') {
                return this.callback_mouseenter(event, callback_id);
            }
            if (event_name === 'mouseleave') {
                return this.callback_mouseleave(event, callback_id);
            }
            if (event_name === 'keydown') {
                return this.callback_keydown(event, callback_id);
            }
            if (event_name === 'hook_keydown') {
                return this.callback_keydown(event, callback_id);
            }
            if (event_name === 'drop') {
                return this.callback_drop(event, callback_id);
            }
            console.error(`No support for the event ${event_name}`);
        };
        if (this.callbacks.has(callback_id)) {
            console.error(`There was already a callback added with the callback_id=${callback_id}`);
            return;
        }
        this.callbacks.set(callback_id, callback);
        if (event_name === 'hook_keydown') {
            document.addEventListener('keydown', callback, false);
        }
        else {
            this.nodes.get('callback_add', id, (node) => {
                node.addEventListener(event_name, callback, false);
            });
        }
    }
    callback_remove(id, event_name, callback_id) {
        const callback = this.callbacks.get(callback_id);
        this.callbacks.delete(callback_id);
        if (callback === undefined) {
            console.error(`The callback is missing with the id=${callback_id}`);
            return;
        }
        if (event_name === 'hook_keydown') {
            document.removeEventListener('keydown', callback);
        }
        else {
            this.nodes.get('callback_remove', id, (node) => {
                node.removeEventListener(event_name, callback);
            });
        }
    }
    dom_bulk_update = (value) => {
        const setFocus = new Set();
        try {
            const commands = JSON.parse(value);
            for (const command of commands) {
                try {
                    this.bulk_update_command(command);
                }
                catch (error) {
                    console.error('bulk_update - item', error, command);
                }
                if (command.type === 'set_attr' && command.name.toLocaleLowerCase() === 'autofocus') {
                    setFocus.add(command.id);
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
        if (command.type === 'remove_text') {
            this.remove_text(BigInt(command.id));
            return;
        }
        if (command.type === 'insert_css') {
            this.insert_css(command.selector, command.value);
            return;
        }
        if (command.type === 'create_comment') {
            const comment = document.createComment(command.value);
            this.nodes.set(BigInt(command.id), comment);
            return;
        }
        if (command.type === 'remove_comment') {
            this.nodes.delete("remove_comment", BigInt(command.id), (comment) => {
                comment.remove();
            });
            return;
        }
        if (command.type === 'callback_add') {
            this.callback_add(BigInt(command.id), command.event_name, BigInt(command.callback_id));
            return;
        }
        if (command.type === 'callback_remove') {
            this.callback_remove(BigInt(command.id), command.event_name, BigInt(command.callback_id));
            return;
        }
        return assertNeverCommand(command);
    }
}

class ApiBrowser {
    cookie;
    interval;
    hashRouter;
    fetch;
    websocket;
    dom;
    constructor(getWasm) {
        this.cookie = new Cookies();
        this.interval = new Interval(getWasm);
        this.hashRouter = new HashRouter(getWasm);
        this.fetch = new Fetch(getWasm);
        this.websocket = new DriverWebsocket(getWasm);
        this.dom = new DriverDom(getWasm);
    }
}

class JsNode {
    api;
    nodes;
    texts;
    wsk;
    constructor(api, nodes, texts, wsk) {
        this.api = api;
        this.nodes = nodes;
        this.texts = texts;
        this.wsk = wsk;
    }
    getByProperty(path, property) {
        try {
            //@ts-expect-error
            const nextCurrentPointer = this.wsk[property];
            return new JsNode(this.api, this.nodes, this.texts, nextCurrentPointer);
        }
        catch (error) {
            console.error('A problem with get', {
                path,
                property,
                error
            });
            return null;
        }
    }
    toValue() {
        return convertToJsValue(this.wsk);
    }
    next(path, command) {
        if (Array.isArray(command)) {
            const [commandName, ...args] = command;
            if (commandName === 'api') {
                return this.nextApi(path, args);
            }
            if (commandName === 'root') {
                return this.nextRoot(path, args);
            }
            if (commandName === 'get') {
                return this.nextGet(path, args);
            }
            if (commandName === 'set') {
                return this.nextSet(path, args);
            }
            if (commandName === 'call') {
                return this.nextCall(path, args);
            }
            if (commandName === 'get_props') {
                return this.nextGetProps(path, args);
            }
            console.error('JsNode.next - wrong commandName', commandName);
            return null;
        }
        console.error('JsNode.next - array was expected', { path, command });
        return null;
    }
    nextApi(path, args) {
        if (args.length === 0) {
            return new JsNode(this.api, this.nodes, this.texts, this.api);
        }
        console.error('nextApi: wrong parameter', { path, args });
        return null;
    }
    nextRoot(path, args) {
        const [firstName, ...rest] = args;
        if (Guard.isString(firstName) && rest.length === 0) {
            if (firstName === 'window') {
                return new JsNode(this.api, this.nodes, this.texts, window);
            }
            if (firstName === 'document') {
                return new JsNode(this.api, this.nodes, this.texts, document);
            }
            console.error(`JsNode.nextRoot: Global name not found -> ${firstName}`, { path, args });
            return null;
        }
        if (Guard.isNumber(firstName) && rest.length === 0) {
            const domId = firstName.value;
            const node = this.nodes.getItem(BigInt(domId));
            if (node !== undefined) {
                return new JsNode(this.api, this.nodes, this.texts, node);
            }
            const text = this.texts.getItem(BigInt(domId));
            if (text !== undefined) {
                return new JsNode(this.api, this.nodes, this.texts, text);
            }
            console.error(`JsNode.nextRoot: No node with id=${domId}`, { path, args });
            return null;
        }
        console.error('JsNode.nextRoot: wrong parameter', { path, args });
        return null;
    }
    nextGet(path, args) {
        const [property, ...getArgs] = args;
        if (Guard.isString(property) && getArgs.length === 0) {
            return this.getByProperty(path, property);
        }
        console.error('JsNode.nextGet - wrong parameters', { path, args });
        return null;
    }
    nextSet(path, args) {
        const [property, value, ...setArgs] = args;
        if (Guard.isString(property) && setArgs.length === 0) {
            try {
                //@ts-expect-error
                this.wsk[property] = convertFromJsValue(value);
                return new JsNode(this.api, this.nodes, this.texts, undefined);
            }
            catch (error) {
                console.error('A problem with set', {
                    path,
                    property,
                    error
                });
                return null;
            }
        }
        console.error('JsNode.nextSet - wrong parameters', { path, args });
        return null;
    }
    nextCall(path, args) {
        const [property, ...callArgs] = args;
        if (Guard.isString(property)) {
            try {
                let paramsJs = callArgs.map(convertFromJsValue);
                //@ts-expect-error
                const result = this.wsk[property](...paramsJs);
                return new JsNode(this.api, this.nodes, this.texts, result);
            }
            catch (error) {
                console.error('A problem with call', {
                    path,
                    property,
                    error
                });
                return null;
            }
        }
        console.error('JsNode.nextCall - wrong parameters', { path, args });
        return null;
    }
    nextGetProps(path, args) {
        const result = {};
        for (const property of args) {
            if (Guard.isString(property)) {
                const value = this.getByProperty(path, property);
                if (value === null) {
                    return null;
                }
                result[property] = value.toValue();
            }
            else {
                console.error('JsNode.nextGetProps - wrong parameters', { path, args, property });
                return null;
            }
        }
        return new JsNode(this.api, this.nodes, this.texts, result);
    }
}

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
        const apiBrowser = new ApiBrowser(getWasm);
        //@ts-expect-error
        window.$vertigoApi = apiBrowser;
        wasmModule = await wasmInit(wasmBinPath, {
            mod: {
                panic_message: (ptr, size) => {
                    const decoder = new TextDecoder("utf-8");
                    const m = getWasm().getUint8Memory().subarray(ptr, ptr + size);
                    const message = decoder.decode(m);
                    console.error('PANIC', message);
                },
                dom_access: (ptr, size) => {
                    let args = getWasm().decodeArguments(ptr, size);
                    if (Array.isArray(args)) {
                        const path = args;
                        let wsk = new JsNode(apiBrowser, apiBrowser.dom.nodes, apiBrowser.dom.texts, null);
                        for (const pathItem of path) {
                            const newWsk = wsk.next(path, pathItem);
                            if (newWsk === null) {
                                return 0;
                            }
                            wsk = newWsk;
                        }
                        return getWasm().valueSaveToBuffer(wsk.toValue());
                    }
                    console.error('dom_access - wrong parameters', args);
                    return 0;
                },
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
