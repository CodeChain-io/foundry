import { PlatformAddress, U64 } from "../classes";
import { Parcel } from "../Parcel";
import { NetworkId } from "../types";

export class Pay extends Parcel {
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

    protected action(): string {
        return "pay";
    }
}
