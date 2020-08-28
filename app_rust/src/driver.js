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
