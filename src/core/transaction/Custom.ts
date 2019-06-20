import { U64 } from "codechain-primitives";

import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface CustomActionJSON {
    handlerId: string;
    bytes: number[];
}

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

    protected actionToJSON(): CustomActionJSON {
        const { handlerId, bytes } = this;
        return {
            handlerId: handlerId.toJSON(),
            bytes: [...bytes]
        };
    }
}
