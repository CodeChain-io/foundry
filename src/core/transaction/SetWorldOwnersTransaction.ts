import { PlatformAddress } from "../../key/PlatformAddress";
import { blake256 } from "../../utils";
import { H256 } from "../H256";

const RLP = require("rlp");

type NetworkId = string;

export interface SetWorldOwnersData {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    nonce: number;
    owners: PlatformAddress[];
}

/**
 * Change the owners of the world
 */
export class SetWorldOwnersTransaction {
    public static fromJSON(obj: any) {
        const {
            data: { networkId, shardId, worldId, nonce, owners }
        } = obj;
        return new this({
            networkId,
            shardId,
            worldId,
            nonce,
            owners
        });
    }
    public readonly networkId: NetworkId;
    public readonly shardId: number;
    public readonly worldId: number;
    public readonly nonce: number;
    public readonly owners: PlatformAddress[];

    public readonly type = "setWorldOwners";

    constructor(data: SetWorldOwnersData) {
        const { networkId, shardId, worldId, nonce, owners } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.worldId = worldId;
        this.nonce = nonce;
        this.owners = owners;
    }

    public toJSON() {
        const { networkId, shardId, worldId, nonce, owners } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                worldId,
                nonce,
                owners
            }
        };
    }

    public toEncodeObject() {
        const { networkId, shardId, worldId, nonce, owners } = this;
        return [2, networkId, shardId, worldId, nonce, owners];
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
