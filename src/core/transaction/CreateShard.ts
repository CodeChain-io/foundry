import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export class CreateShard extends Transaction {
    public constructor(networkId: NetworkId) {
        super(networkId);
    }

    public type(): string {
        return "createShard";
    }

    protected actionToEncodeObject(): any[] {
        return [4];
    }

    protected actionToJSON(): any {
        return {};
    }
}
