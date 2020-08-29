//@ts-check

export function consoleLog(message) {
    console.info("Heja ho, tu konsola", message);
}

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

let emmiter = new EventEmmiter();

export function startDriverLoop(callback) {
    emmiter.add(callback);

    console.info("Rozpoczynam synchronizowanie");

    const root = document.createElement('div');

    const wrapper = document.createElement('div');
    wrapper.setAttribute('style', 'padding: 5px; background-color: #e0e0e0;');

    const button = document.createElement('button');

    const text = document.createTextNode('click');

    button.append(text);
    wrapper.appendChild(button);
    root.appendChild(wrapper);
    document.body.appendChild(root);

    root.addEventListener('click', (event) => {
        console.info('trigger....', event);
        emmiter.trigger();
    });
}

/**
 * @param {number} id 
 * @param {string} name 
 */
export function createNode(id, name) {
    console.info('createNode', id, name);
}

/**
 * @param {number} id 
 * @param {string} value 
 */
export function createText(id, value) {
    console.info('createNode', id, value);
}

/**
 * @param {number} id 
 * @param {string} value 
 */
export function createComment(id, value) {
    console.info('createComment', id, value);
}

/**
 * @param {number} id 
 * @param {string} key
 * @param {string} value 
 */
export function setAttr(id, key, value) {
    console.info('setAttr', id, key, value);
}

/**
 * @param {number} id 
 * @param {string} name 
 */
export function removeAttr(id, name) {
    console.info('removeAttr', id, name);
}

/**
 * @param {number} id 
 */
export function remove(id) {
    console.info('remove', id);
}

/**
 * @param {number} id 
 */
export function removeAllChild(id) {
    console.info('removeAllChild', id);
}

/**
 * @param {number} parent
 * @param {number} child
 */
export function insertAsFirstChild(parent, child) {
    console.info('insertAsFirstChild', parent, child);
}

/**
 * @param {number} refId
 * @param {number} child
 */
export function insertBefore(refId, child) {
    console.info('insertAsFirstChild', refId, child);
}

/**
 * @param {number} refId
 * @param {number} child
 */
export function insertAfter(refId, child) {
    console.info('insertAsFirstChild', refId, child);
}

/**
 * @param {number} parent
 * @param {number} child
 */
export function addChild(parent, child) {
    console.info('insertAsFirstChild', parent, child);
}