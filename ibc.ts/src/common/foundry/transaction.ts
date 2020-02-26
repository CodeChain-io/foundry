import { Transaction } from "codechain-sdk/lib/core/classes";
import { NetworkId } from "codechain-sdk/lib/core/types";

export interface IBCActionJSON {
    bytes: Buffer;
}

export class IBC extends Transaction {
    private readonly bytes: Buffer;

    public constructor(networkId: NetworkId, bytes: Buffer) {
        super(networkId);
        this.bytes = bytes;
    }

    public type(): string {
        return "ibc";
    }

    protected actionToEncodeObject(): any[] {
        return [0x20, this.bytes];
    }

    // Since the result type is hard-coded in the SDK, we should use any type here.
    protected actionToJSON(): any {
        return {
            bytes: this.bytes
        };
    }
}
