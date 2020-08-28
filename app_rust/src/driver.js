export function add(a, b) {
    return a + b;
}

export function consoleLog(message) {
    console.info("Heja ho, tu konsola", message);
}

let app = 4;

export function startDriverLoop() {
    console.info("Rozpoczynam synchronizowanie");

    setInterval(() => {
        app++;
        console.info("synchronize", app);
    }, 1000);
}
