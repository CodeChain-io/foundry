import { Address, U64 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export interface DelegateCCSActionJSON {
    address: string;
    quantity: string;
}

export class DelegateCCS extends Transaction {
    private readonly address: Address;
    private readonly quantity: U64;

    public constructor(address: Address, quantity: U64, networkId: NetworkId) {
        super(networkId);
        this.address = address;
        this.quantity = quantity;
    }

    public type(): string {
        return "delegateCCS";
    }

    protected actionToEncodeObject(): any[] {
        return [
            0x22,
            this.address.getPubKey().toEncodeObject(),
            this.quantity.toEncodeObject()
        ];
    }

    protected actionToJSON(): DelegateCCSActionJSON {
        return {
            address: this.address.value,
            quantity: this.quantity.toJSON()
        };
    }
}
