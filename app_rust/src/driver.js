export function add(a, b) {
    return a + b;
}

export function consoleLog(message) {
    console.info("Heja ho, tu konsola", message);
}

let app = 4;
let callbackList = [];

export function startDriverLoop(callback) {
    callbackList.push(callback);

    console.info("Rozpoczynam synchronizowanie");

    setInterval(() => {
        app++;
        console.info("synchronize");

        for (const item of callbackList) {
            item(app);
        }
    }, 1000);
}
