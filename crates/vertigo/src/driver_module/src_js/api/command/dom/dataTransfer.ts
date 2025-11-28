interface FileItemType {
    name: string,
    data: Uint8Array,
}

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
                files.push(file
                    .arrayBuffer()
                    .then((data): FileItemType => ({
                        name: file.name,
                        data: new Uint8Array(data),
                    }))
                );
            }
        }
    }
    return files;
}
