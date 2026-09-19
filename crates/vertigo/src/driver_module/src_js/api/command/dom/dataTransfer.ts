import { JsJsonType } from "../../../jsjson";

interface FileItemType {
    name: string,
    data: Uint8Array,
}

/// What both file paths ultimately send: one `[[name, bytes], ...]` payload.
export type Send = (value: JsJsonType) => JsJsonType;

export const readFile = (file: File): Promise<FileItemType> =>
    file.arrayBuffer().then((data): FileItemType => ({
        name: file.name,
        data: new Uint8Array(data),
    }));

export function getFiles(items: DataTransferItemList): Array<Promise<FileItemType>> {
    const files: Array<Promise<FileItemType>> = [];

    for (let i = 0; i < items.length; i++) {
        const item = items[i];

        if (item === undefined) {
            console.error('dom -> drop -> item - undefined');
        } else {
            const file = item.getAsFile();

            if (file === null) {
                console.error(`dom -> drop -> index:${i} -> It's not a file`);
            } else {
                files.push(readFile(file));
            }
        }
    }
    return files;
}

/// The tail shared by `drop` and `changeFile`.
///
/// `label` is a parameter only so the two callers keep their own wording in the failure line,
/// which is what they printed before this was factored out.
export const sendFiles = (files: Array<Promise<FileItemType>>, send: Send, label: string) => {
    Promise.all(files).then((loaded) => {
        const params = [];

        for (const file of loaded) {
            // Uint8Array -> array of numbers, which is what JsJson carries.
            params.push([file.name, Array.from(file.data)]);
        }

        send([params]);
    }).catch((error) => {
        console.error(label, error);
    });
};
