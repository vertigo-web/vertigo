// Pictures made smaller for `vertigo::image` (`image.rs`): the browser reads the file, turned
// as its camera wrote into it, draws it scaled on a canvas nobody sees - it is never put into
// the page - and encodes that. So an app carries no decoders and no encoder in its wasm.
//
// A chunk of its own, loaded with the first picture (`image_lazy.ts`).

import type { JsJsonType } from "../../jsjson";

/// How the browser tells wasm of a picture, through the callback its call came with: an
/// `ImageAnswer` of `dev/command.rs`.
export type Tell = (callback: number, answer: JsJsonType) => void;

/// A call of `vertigo::image`, as `ImageOp` in `dev/command.rs` is sent.
export type ImageOp = {
    Resize: {
        callback: number,
        data: Uint8Array<ArrayBuffer>,
        max_width: number,
        max_height: number,
        mime: string,
        quality: number,
        background: string,
        keep_fitting: boolean,
    }
};

/// The answer that something failed, with what the browser said.
const failed = (error: unknown): JsJsonType => ({
    Failed: { message: error instanceof Error ? error.message : String(error) }
});

export class Images {
    public constructor(private readonly tell: Tell) {}

    public run(op: ImageOp) {
        const resize = op.Resize;
        this.resize(resize).then(
            (answer) => { this.tell(resize.callback, answer); },
            (error: unknown) => { this.tell(resize.callback, failed(error)); },
        );
    }

    private async resize(op: ImageOp['Resize']): Promise<JsJsonType> {
        // Without a type: the browser tells the format by the bytes
        const picture = await createImageBitmap(new Blob([op.data]));

        try {
            const { width: original_width, height: original_height } = picture;
            // Never bigger than it is
            const scale = Math.min(1, op.max_width / original_width, op.max_height / original_height);

            if (op.keep_fitting && scale === 1) {
                return { Resized: { width: original_width, height: original_height, original_width, original_height, data: null } };
            }

            const width = Math.max(1, Math.round(original_width * scale));
            const height = Math.max(1, Math.round(original_height * scale));

            const canvas = document.createElement('canvas');
            canvas.width = width;
            canvas.height = height;

            const context = canvas.getContext('2d');
            if (context === null) {
                throw new Error('the browser gives no canvas to draw the picture on');
            }

            // A JPEG has no transparency: what shows through is the canvas, black by itself
            if (op.mime === 'image/jpeg') {
                context.fillStyle = op.background;
                context.fillRect(0, 0, width, height);
            }
            context.imageSmoothingQuality = 'high';
            context.drawImage(picture, 0, 0, width, height);

            const encoded = await new Promise<Blob>((resolve, reject) => {
                canvas.toBlob((blob) => {
                    if (blob === null) {
                        reject(new Error('the browser could not encode the picture'));
                    } else {
                        resolve(blob);
                    }
                }, op.mime, op.quality);
            });

            return {
                Resized: { width, height, original_width, original_height, data: new Uint8Array(await encoded.arrayBuffer()) }
            };
        } finally {
            // The pixels, which are many megabytes of a photo, at once and not when the
            // browser gets to collecting them
            picture.close();
        }
    }
}
