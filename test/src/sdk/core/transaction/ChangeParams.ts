import { U64 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface ChangeParamsActionJSON {
    metadataSeq: string;
    params: (number | string)[];
    approvals: any[];
}

export class ChangeParams extends Transaction {
    private readonly metadataSeq: U64;
    private readonly params: (number | string)[];
    private readonly approvals: any[];

    public constructor(
        metadataSeq: U64,
        params: (number | string)[],
        approvals: any[],
        networkId: NetworkId
    ) {
        super(networkId);
        this.metadataSeq = metadataSeq;
        this.approvals = approvals;
        this.params = params;
    }

    public type(): string {
        return "changeParams";
    }

    protected actionToEncodeObject(): any[] {
        return [
            0xff,
            this.metadataSeq.toEncodeObject(),
            this.params,
            ...this.approvals
        ];
    }

    protected actionToJSON(): ChangeParamsActionJSON {
        return {
            metadataSeq: this.metadataSeq.toJSON(),
            params: this.params,
            approvals: this.approvals
        };
    }
}
