import { Parcel } from "../Parcel";
import { NetworkId } from "../types";

export class CreateShard extends Parcel {
    public constructor(networkId: NetworkId) {
        super(networkId);
    }

    protected actionToEncodeObject(): any[] {
        return [4];
    }

    protected actionToJSON(): any {
        return {};
    }

    protected action(): string {
        return "createShard";
    }
}
