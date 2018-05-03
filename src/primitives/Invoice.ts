const RLP = require("rlp");

type TransactionOutcome = "Success" | "Failed";

export class Invoice {
    private outcome: TransactionOutcome;

    constructor(success: boolean) {
        this.outcome = success ? "Success" : "Failed";
    }

    static fromBytes(buffer: Buffer): Invoice {
        const bytes = Array.from(buffer.values());
        if (bytes.length !== 1 || bytes[0] > 0x01) {
            throw `Invalid RLP for Invoice: ${bytes}`;
        }
        return new Invoice(RLP.decode(buffer)[0]);
    }

    toEncodeObject(): boolean {
        return this.outcome === "Success";
    }

    rlpBytes(): Buffer {
        return Buffer.from([this.toEncodeObject() ? 0x01 : 0x00]);
    }
}
