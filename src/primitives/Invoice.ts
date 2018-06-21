const RLP = require("rlp");

/**
 * Used to know whether a transaction succeeded or failed.
 */
export class Invoice {
    private success: boolean;

    constructor(success: boolean) {
        this.success = !!success;
    }

    static fromJSON(data: any) {
        return new this(data === "Success");
    }

    toJSON() {
        return this.success ? "Success" : "Failed";
    }

    static fromBytes(buffer: Buffer): Invoice {
        const bytes = Array.from(buffer.values());
        if (bytes.length !== 1 || bytes[0] > 0x01) {
            throw `Invalid RLP for Invoice: ${bytes}`;
        }
        return new Invoice(RLP.decode(buffer)[0]);
    }

    toEncodeObject(): boolean {
        return this.success;
    }

    rlpBytes(): Buffer {
        return Buffer.from([this.toEncodeObject() ? 0x01 : 0x00]);
    }
}
