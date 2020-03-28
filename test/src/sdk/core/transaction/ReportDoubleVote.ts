import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface ReportDoubleVoteActionJSON {
    message1: number[];
    message2: number[];
}

export class ReportDoubleVote extends Transaction {
    private readonly message1: Buffer;
    private readonly message2: Buffer;

    public constructor(
        message1: Buffer,
        message2: Buffer,
        networkId: NetworkId
    ) {
        super(networkId);
        this.message1 = message1;
        this.message2 = message2;
    }

    public type(): string {
        return "reportDoubleVote";
    }

    protected actionToEncodeObject(): any[] {
        return [0x25, this.message1, this.message2];
    }

    protected actionToJSON(): ReportDoubleVoteActionJSON {
        return {
            message1: [...this.message1],
            message2: [...this.message2]
        };
    }
}
