import { Transaction } from "../Transaction";
import { NetworkId } from "../types";
import { U64 } from "../U64";

export class Custom extends Transaction {
    private readonly handlerId: U64;
    private readonly bytes: Buffer;

    constructor(
        params: { handlerId: U64; bytes: Buffer },
        networkId: NetworkId
    ) {
        super(networkId);

        this.handlerId = params.handlerId;
        this.bytes = params.bytes;
    }

    public type(): string {
        return "custom";
    }

    protected actionToEncodeObject(): any[] {
        const { handlerId, bytes } = this;
        return [0xff, handlerId.toEncodeObject(), bytes];
    }

    protected actionToJSON(): any {
        const { handlerId, bytes } = this;
        return {
            handlerId: handlerId.toJSON(),
            buffer: bytes
        };
    }
}
