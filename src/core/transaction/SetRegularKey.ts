import { H512 } from "../classes";
import { Transaction } from "../Transaction";
import { NetworkId } from "../types";

export class SetRegularKey extends Transaction {
    private readonly key: H512;
    public constructor(key: H512, networkId: NetworkId) {
        super(networkId);
        this.key = key;
    }

    public action(): string {
        return "setRegularKey";
    }

    protected actionToEncodeObject(): any[] {
        return [3, this.key.toEncodeObject()];
    }

    protected actionToJSON(): any {
        return {
            key: this.key.toJSON()
        };
    }
}
