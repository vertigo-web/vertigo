import { JsJsonType } from "../jsjson";

/*
{type: 14, value: 'FetchCacheGet'}
*/

//TODO - dodać typy

export const exec_command = (arg: JsJsonType): JsJsonType => {

    console.info('exec_command: Do przetworzenia', arg);

    const cache = document.documentElement.getAttribute('data-fetch-cache');

    return {
        data: cache
    };
};

