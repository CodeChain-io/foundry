import { H160 } from "../H160";
import { H256 } from "../H256";
import { blake256 } from "../../utils";

const RLP = require("rlp");

export type SetWorldOwnersData = {
    networkId: number;
    shardId: number;
    worldId: number;
    nonce: number;
    owners: H160[];
};

/**
 * Change the owners of the world
 */
export class SetWorldOwnersTransaction {
    readonly networkId: number;
    readonly shardId: number;
    readonly worldId: number;
    readonly nonce: number;
    readonly owners: H160[];

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
