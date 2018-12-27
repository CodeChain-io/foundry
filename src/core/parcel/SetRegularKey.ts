import { H512 } from "../classes";
import { Parcel } from "../Parcel";
import { NetworkId } from "../types";

export class SetRegularKey extends Parcel {
    private readonly key: H512;
    public constructor(key: H512, networkId: NetworkId) {
        super(networkId);
        this.key = key;
    }

    protected actionToEncodeObject(): any[] {
        return [3, this.key.toEncodeObject()];
    }

    protected actionToJSON(): any {
        return {
            key: this.key.toJSON()
        };
    }

    protected action(): string {
        return "setRegularKey";
    }
}
