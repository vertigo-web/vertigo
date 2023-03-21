//@ts-check

/**
 * @param {string} url
 */
function start_watch(url) {
    /** @type string | null */
    let last_message = null;

    /** @type HTMLElement | null */
    let building_message = null;

    /** @param {string} message  */
    function processMessage(message) {
        console.info(`Watch message: ${message}`);

        if (message === 'Building') {
            console.info('Watch - show building');
            showBuilding(buildInProgressMessage);
            return;
        }

        if (message === 'Errors') {
            console.info('Watch - show errors');
            showBuilding(buildErrorsMessage);
            return;
        }

        if (last_message === null) {
            console.info(`Watch - init version ${message}`);
            last_message = message;
            return;
        }

        console.info("Watch - reload");
        window.location.reload();
    }

    /**
     * @param {() => HTMLElement} createDomMessage
     */
    function showBuilding(createDomMessage) {
        if (building_message !== null) {
            building_message.remove();
        }

        const div = createDomMessage();
        document.documentElement.appendChild(div);
        building_message = div;
    }

    const events = new EventSource(url);
    events.onmessage = function(event) {
        const message = event.data;

        if (typeof message === 'string') {
            processMessage(message);
        } else {
            console.error('Invalid message type received. Expected string, but received ' + typeof message + '.');
        }
    };
}

/**
 * @param {string} name
 * @param {Array<[string, string]>} chunks
 * @returns {HTMLElement}
 */
function createElement(name, chunks) {
    const div = document.createElement(name);

    if (chunks.length > 0) {
        const css = chunks.map(([key, value]) => `${key}:${value}`).join(';');
        div.setAttribute('style', css);
    }

    return div;
}

/**
 * Builds an HTML message indicating that an action is in progress.
 * @returns {HTMLElement} HTML message indicating that an action is in progress.
 */
function buildInProgressMessage() {
    const div = createElement('div', [
        ['position', 'fixed'],
        ['top', '0'],
        ['right', '0'],
        ['bottom', '0'],
        ['left', '0'],
        ['display', 'flex'],
        ['align-items', 'center'],
        ['justify-content', 'center'],
        ['background-color', '#00000080']
    ]);

    const inner = createElement('div', [
        ['width', '300px'],
        ['height', '50px'],
        ['line-height', '50px'],
        ['background-color', 'white'],
        ['text-align', 'center'],
        ['border', '1px solid #e0e0e0']
    ]);

    const text = document.createTextNode('Build in progress ...');

    div.appendChild(inner);
    inner.appendChild(text);

    return div;
}

/**
 * Builds an HTML message indicating that an action is in progress.
 * @returns {HTMLElement} HTML message indicating that an action is in progress.
 */
function buildErrorsMessage() {
    const div = createElement('div', [
        ['position', 'fixed'],
        ['top', '0'],
        ['right', '0'],
        ['bottom', '0'],
        ['left', '0'],
        ['display', 'flex'],
        ['align-items', 'center'],
        ['justify-content', 'center'],
        ['background-color', '#00000080']
    ]);

    const inner = createElement('div', [
        ['width', '300px'],
        ['line-height', '50px'],
        ['background-color', 'white'],
        ['text-align', 'center'],
        ['color', 'red'],
        ['border', '1px solid red'],
        ['padding', '0 20px'],
        ['display', 'flex'],
        ['flex-direction', 'column'],
    ]);

    const messageDiv = document.createElement('div');
    const messageText = document.createTextNode('Compilation failed. More details can be found in the console.');
    messageDiv.appendChild(messageText);

    const escDiv = createElement('div', [
        ['align-self', 'end'],
        ['cursor', 'pointer'],
    ]);
    const escText = document.createTextNode('[X]');
    escDiv.appendChild(escText);
    escDiv.addEventListener('click', () => {
        while(div.firstChild) div.removeChild(div.firstChild);
        div.setAttribute('style', '{ display: none }')
    });

    inner.appendChild(escDiv);
    inner.appendChild(messageDiv);
    div.appendChild(inner);

    return div;
}
