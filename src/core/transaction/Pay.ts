import { PlatformAddress, U64 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export class Pay extends Transaction {
    private readonly receiver: PlatformAddress;
    private readonly amount: U64;

    public constructor(
        receiver: PlatformAddress,
        amount: U64,
        networkId: NetworkId
    ) {
        super(networkId);
        this.receiver = receiver;
        this.amount = amount;
    }

    public action(): string {
        return "pay";
    }

    protected actionToEncodeObject(): any[] {
        return [
            2,
            this.receiver.getAccountId().toEncodeObject(),
            this.amount.toEncodeObject()
        ];
    }

    protected actionToJSON(): any {
        return {
            receiver: this.receiver.value,
            amount: this.amount.toEncodeObject()
        };
    }
}
