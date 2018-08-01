import { Buffer } from "buffer";

const RLP = require("rlp");

/**
 * An Invoice is used to know whether a transaction or a parcel succeeded or
 * failed.
 */
export class Invoice {
    readonly success: boolean;

    /**
     * @param success Whether a transaction or a parcel succeeded or failed.
     */
    constructor(success: boolean) {
        this.success = !!success;
    }

    // FIXME: any
    /**
     * Create an Invoice from an Invoice JSON object.
     * @param data An Invoice JSON object.
     * @returns An Invoice.
     */
    static fromJSON(data: any) {
        return new this(data === "Success");
    }

    /**
     * Convert to an Invoice JSON object.
     * @returns An Invoice JSON object.
     */
    toJSON() {
        return this.success ? "Success" : "Failed";
    }

    /**
     * Decode RLP bytes to an Invoice.
     * @param buffer RLP bytes.
     * @returns An Invoice.
     */
    static fromBytes(buffer: Buffer): Invoice {
        const bytes = Array.from(buffer.values());
        if (bytes.length !== 1 || bytes[0] > 0x01) {
            throw `Invalid RLP for Invoice: ${bytes}`;
        }
        return new Invoice(RLP.decode(buffer)[0]);
    }

    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): boolean {
        return this.success;
    }

    /**
     * Convert to RLP bytes
     * @returns RLP bytes
     */
    rlpBytes(): Buffer {
        return Buffer.from([this.toEncodeObject() ? 0x01 : 0x00]);
    }
}
