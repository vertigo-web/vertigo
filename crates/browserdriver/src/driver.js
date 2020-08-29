//@ts-check

export function consoleLog(message) {
    console.info("Heja ho, tu konsola", message);
}

// const showLog = false;

// const showMessage = (...args) => {
//     if (showLog) {
//         console.log.call(console, args);
//     }
// }

class EventEmmiter {
    /**
     * @property {Array<() => void>} callbackList
     */

    constructor() {
        this.callbackList = [];
    }

    /**
     * @param {() => void} callback 
     */
    add(callback) {
        this.callbackList.push(callback);
    }

    trigger() {
        for (const item of this.callbackList) {
            item();
        }
    }
}

/**
 * @param {HTMLElement} root 
 */
function createMock(root) {
    const wrapper = document.createElement('div');
    wrapper.setAttribute('style', 'padding: 5px; background-color: #e0e0e0;');
    const button = document.createElement('button');
    const text = document.createTextNode('click');
    button.append(text);
    wrapper.appendChild(button);
    root.appendChild(wrapper);
}

class State {

    /**
     * @property {EventEmmiter} _emmiter
     * @property {Map<BigInt, HTMLElement | Text | Comment} _nodes
     */

    constructor() {
        this._emmiter = new EventEmmiter();
        this._nodes = new Map();
    }

    /**
     * @param {() => void} callback 
     */
    addEvent(callback) {
        this._emmiter.add(callback);
    }

    trigger() {
        this._emmiter.trigger();
    }

    /**
     * @param {BigInt} id 
     * @param {HTMLElement | Text | Comment} node 
     */
    setChild(id, node) {
        this._nodes.set(id, node);
    }

    /**
     * @param {BigInt} id
     * @returns {HTMLElement | Text | Comment}
     */
    getChild(id) {
        const node = this._nodes.get(id);
        if (node) {
            return node;
        }

        throw Error(`Not found node: ${id}`);
    }

    /**
     * @param {BigInt} id
     * @returns {HTMLElement}
     */
    getChildNode(id) {
        const node = this._nodes.get(id);
        if (node instanceof HTMLElement) {
            return node;
        }

        throw Error(`Not found HTMLElement: ${id}`);
    }
    
}

const state = new State();

export function startDriverLoop(callback) {
    state.addEvent(callback);

    console.info("startDriverLoop - js");

    const root = document.createElement('div');
    //createMock(root);
    document.body.appendChild(root);

    state.setChild(BigInt(1), root);

    root.addEventListener('click', (event) => {
        console.info('trigger....', event);
        state.trigger();
    });
}

/**
 * @param {BigInt} id 
 * @param {string} name 
 */
export function createNode(id, name) {
    const node = document.createElement(name);
    state.setChild(id, node);

}

/**
 * @param {BigInt} id 
 * @param {string} value 
 */
export function createText(id, value) {
    const text = document.createTextNode(value);
    state.setChild(id, text);
}

/**
 * @param {BigInt} id 
 * @param {string} value 
 */
export function createComment(id, value) {
    const comment = document.createComment(value);
    state.setChild(id, comment);
}

/**
 * @param {BigInt} id 
 * @param {string} key
 * @param {string} value 
 */
export function setAttr(id, key, value) {
    const node = state.getChildNode(id);
    node.setAttribute(key, value);
}

/**
 * @param {BigInt} id 
 * @param {string} name 
 */
export function removeAttr(id, name) {
    const node = state.getChildNode(id);
    node.removeAttribute(name);
}

/**
 * @param {BigInt} id 
 */
export function remove(id) {
    const node = state.getChild(id);
    node.parentElement.removeChild(node);
}

/**
 * @param {BigInt} id 
 */
export function removeAllChild(id) {
    console.info('removeAllChild', id);
    throw Error('TODO removeAllChild');                         //TODO
}

/**
 * @param {BigInt} parent
 * @param {BigInt} child
 */
export function insertAsFirstChild(parent, child) {
    const parentNode = state.getChildNode(parent);
    const childNode = state.getChild(child);
    parentNode.insertBefore(childNode, null);
}

/**
 * @param {BigInt} refId
 * @param {BigInt} child
 */
export function insertBefore(refId, child) {
    const refNode = state.getChild(refId);
    const childNode = state.getChild(child);
    refNode.parentNode.insertBefore(childNode, refNode);
}

/**
 * @param {BigInt} refId
 * @param {BigInt} child
 */
export function insertAfter(refId, child) {
    const refNode = state.getChild(refId);
    const childNode = state.getChild(child);

    const next = refNode.nextSibling;

    if (next) {
        refNode.parentNode.insertBefore(childNode, next);
    } else {
        refNode.parentNode.appendChild(childNode);
    }
}

/**
 * @param {BigInt} parent
 * @param {BigInt} child
 */
export function addChild(parent, child) {
    const parentNode = state.getChild(parent);
    const childNode = state.getChild(child);
    parentNode.appendChild(childNode);
}