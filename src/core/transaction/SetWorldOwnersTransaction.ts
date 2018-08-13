import { PlatformAddress } from "../../key/PlatformAddress";
import { H256 } from "../H256";
import { blake256 } from "../../utils";

const RLP = require("rlp");

type NetworkId = string;

export type SetWorldOwnersData = {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    nonce: number;
    owners: PlatformAddress[];
};

/**
 * Change the owners of the world
 */
export class SetWorldOwnersTransaction {
    readonly networkId: NetworkId;
    readonly shardId: number;
    readonly worldId: number;
    readonly nonce: number;
    readonly owners: PlatformAddress[];

    readonly type = "setWorldOwners";

    constructor(data: SetWorldOwnersData) {
        const { networkId, shardId, worldId, nonce, owners } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.worldId = worldId;
        this.nonce = nonce;
        this.owners = owners;
    }

    static fromJSON(obj: any) {
        const { data: { networkId, shardId, worldId, nonce, owners } } = obj;
        return new this({
            networkId,
            shardId,
            worldId,
            nonce,
            owners,
        });
    }

    toJSON() {
        const { networkId, shardId, worldId, nonce, owners } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                worldId,
                nonce,
                owners,
            }
        };
    }

    toEncodeObject() {
        const { networkId, shardId, worldId, nonce, owners } = this;
        return [
            2,
            networkId,
            shardId,
            worldId,
            nonce,
            owners
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
