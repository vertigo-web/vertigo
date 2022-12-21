///https://javascript.info/arraybuffer-binary-arrays#dataview

const decoder = new TextDecoder("utf-8");
const encoder = new TextEncoder();

export class BufferCursor {
    private dataView: DataView;
    private pointer: number = 0;

    constructor(
        private getUint8Memory: () => Uint8Array,
        private ptr: number,
        private size: number,
    ) {
        this.dataView = new DataView(
            this.getUint8Memory().buffer,
            this.ptr,
            this.size
        );
    }

    public getByte(): number {
        const value = this.dataView.getUint8(this.pointer);
        this.pointer += 1;
        return value;
    }

    public setByte(byte: number) {
        this.dataView.setUint8(this.pointer, byte);
        this.pointer += 1;
    }

    public getU16(): number {
        const value = this.dataView.getUint16(this.pointer);
        this.pointer += 2;
        return value;
    }

    public setU16(value: number) {
        this.dataView.setUint16(this.pointer, value);
        this.pointer += 2;
    }

    public getU32(): number {
        const value = this.dataView.getUint32(this.pointer);
        this.pointer += 4;
        return value;
    }

    public setU32(value: number) {
        this.dataView.setUint32(this.pointer, value);
        this.pointer += 4;
    }

    public getI32(): number {
        const value = this.dataView.getInt32(this.pointer);
        this.pointer += 4;
        return value;
    }

    public setI32(value: number) {
        this.dataView.setInt32(this.pointer, value);
        this.pointer += 4;
    }

    public getU64(): bigint {
        const value = this.dataView.getBigUint64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public setU64(value: bigint) {
        this.dataView.setBigUint64(this.pointer, value);
        this.pointer += 8;
    }

    public getI64(): bigint {
        const value = this.dataView.getBigInt64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public setI64(value: bigint) {
        this.dataView.setBigInt64(this.pointer, value);
        this.pointer += 8;
    }

    public getF64(): number {
        const value = this.dataView.getFloat64(this.pointer);
        this.pointer += 8;
        return value;
    }

    public setF64(value: number) {
        this.dataView.setFloat64(this.pointer, value);
        this.pointer += 8;
    }

    public getBuffer(): Uint8Array {
        const size = this.getU32();
        const result = this
            .getUint8Memory()
            .subarray(
                this.ptr + this.pointer,
                this.ptr + this.pointer + size
            );

        this.pointer += size;
        return result;
    }

    public setBuffer(buffer: Uint8Array) {
        const size = buffer.length;
        this.setU32(size);

        const subbuffer = this
            .getUint8Memory()
            .subarray(
                this.ptr + this.pointer,
                this.ptr + this.pointer + size
            );

        subbuffer.set(buffer);

        this.pointer += size;
    }

    public getString(): string {
        return decoder.decode(this.getBuffer());
    }

    public setString(value: string) {
        const buffer = encoder.encode(value);
        this.setBuffer(buffer);
    }
}

export const getStringSize = (value: string): number => {
    return new TextEncoder().encode(value).length;
};

