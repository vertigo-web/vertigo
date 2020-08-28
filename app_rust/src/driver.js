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

    window.button33.addEventListener('click', () => {
        console.info("klik w przycisk");
        emmiter.trigger();
    }, false);
}
