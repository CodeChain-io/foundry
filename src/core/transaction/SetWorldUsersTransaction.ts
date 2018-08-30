import { PlatformAddress } from "../../key/PlatformAddress";
import { blake256 } from "../../utils";
import { H256 } from "../H256";

const RLP = require("rlp");

type NetworkId = string;

export interface SetWorldUsersData {
    networkId: NetworkId;
    shardId: number;
    worldId: number;
    nonce: number;
    users: PlatformAddress[];
}

/**
 * Change the users of the world
 */
export class SetWorldUsersTransaction {
    public static fromJSON(obj: any) {
        const {
            data: { networkId, shardId, worldId, nonce, users }
        } = obj;
        return new this({
            networkId,
            shardId,
            worldId,
            nonce,
            users
        });
    }
    public readonly networkId: NetworkId;
    public readonly shardId: number;
    public readonly worldId: number;
    public readonly nonce: number;
    public readonly users: PlatformAddress[];

    public readonly type = "setWorldUsers";

    constructor(data: SetWorldUsersData) {
        const { networkId, shardId, worldId, nonce, users } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.worldId = worldId;
        this.nonce = nonce;
        this.users = users;
    }

    public toJSON() {
        const { networkId, shardId, worldId, nonce, users } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                worldId,
                nonce,
                users
            }
        };
    }

    public toEncodeObject() {
        const { networkId, shardId, worldId, nonce, users } = this;
        return [2, networkId, shardId, worldId, nonce, users];
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
