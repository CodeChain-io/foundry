import { U64 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface SelfNominateActionJSON {
    deposit: string;
    metadata: number[];
}

export class SelfNominate extends Transaction {
    private readonly deposit: U64;
    private readonly metadata: Buffer;

    public constructor(deposit: U64, metadata: Buffer, networkId: NetworkId) {
        super(networkId);
        this.deposit = deposit;
        this.metadata = metadata;
    }

    public type(): string {
        return "selfNominate";
    }

    protected actionToEncodeObject(): any[] {
        return [0x24, this.deposit.toEncodeObject(), this.metadata];
    }

    protected actionToJSON(): SelfNominateActionJSON {
        return {
            deposit: this.deposit.toJSON(),
            metadata: [...this.metadata]
        };
    }
}
