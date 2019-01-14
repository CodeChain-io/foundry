import { PlatformAddress, U64 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export class Pay extends Transaction {
    private readonly receiver: PlatformAddress;
    private readonly quantity: U64;

    public constructor(
        receiver: PlatformAddress,
        quantity: U64,
        networkId: NetworkId
    ) {
        super(networkId);
        this.receiver = receiver;
        this.quantity = quantity;
    }

    public type(): string {
        return "pay";
    }

    protected actionToEncodeObject(): any[] {
        return [
            2,
            this.receiver.getAccountId().toEncodeObject(),
            this.quantity.toEncodeObject()
        ];
    }

    protected actionToJSON(): any {
        return {
            receiver: this.receiver.value,
            quantity: this.quantity.toEncodeObject()
        };
    }
}
