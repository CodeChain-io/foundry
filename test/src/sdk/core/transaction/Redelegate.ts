import { Address, U64 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface RedelegateActionJSON {
    prevDelegator: string;
    nextDelegator: string;
    quantity: string;
}

export class Redelegate extends Transaction {
    private readonly prevDelegator: Address;
    private readonly nextDelegator: Address;
    private readonly quantity: U64;

    public constructor(
        prevDelegator: Address,
        nextDelegator: Address,
        quantity: U64,
        networkId: NetworkId
    ) {
        super(networkId);
        this.prevDelegator = prevDelegator;
        this.nextDelegator = nextDelegator;
        this.quantity = quantity;
    }

    public type(): string {
        return "redelegate";
    }

    protected actionToEncodeObject(): any[] {
        return [
            0x26,
            this.prevDelegator.getPubKey().toEncodeObject(),
            this.nextDelegator.getPubKey().toEncodeObject(),
            this.quantity.toEncodeObject()
        ];
    }

    protected actionToJSON(): RedelegateActionJSON {
        return {
            prevDelegator: this.prevDelegator.value,
            nextDelegator: this.nextDelegator.value,
            quantity: this.quantity.toJSON()
        };
    }
}
