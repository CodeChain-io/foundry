import { PlatformAddress } from "../../key/PlatformAddress";
import { H256 } from "../H256";
import { blake256 } from "../../utils";

const RLP = require("rlp");

type NetworkId = string;

export type SetWorldUsersData = {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    nonce: number;
    users: PlatformAddress[];
};

/**
 * Change the users of the world
 */
export class SetWorldUsersTransaction {
    readonly networkId: NetworkId;
    readonly shardId: number;
    readonly worldId: number;
    readonly nonce: number;
    readonly users: PlatformAddress[];

    readonly type = "setWorldUsers";

    constructor(data: SetWorldUsersData) {
        const { networkId, shardId, worldId, nonce, users } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.worldId = worldId;
        this.nonce = nonce;
        this.users = users;
    }

    static fromJSON(obj: any) {
        const { data: { networkId, shardId, worldId, nonce, users } } = obj;
        return new this({
            networkId,
            shardId,
            worldId,
            nonce,
            users,
        });
    }

    toJSON() {
        const { networkId, shardId, worldId, nonce, users } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                worldId,
                nonce,
                users,
            }
        };
    }

    toEncodeObject() {
        const { networkId, shardId, worldId, nonce, users } = this;
        return [
            2,
            networkId,
            shardId,
            worldId,
            nonce,
            users
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
