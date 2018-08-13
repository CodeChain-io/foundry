import { PlatformAddress } from "../../key/PlatformAddress";
import { H256 } from "../H256";
import { blake256 } from "../../utils";

const RLP = require("rlp");

type NetworkId = string;

export type CreateWorldData = {
    networkId: NetworkId;
    shardId: number;
    nonce: number;
    owners: PlatformAddress[];
};

/**
 * Creates a world
 *
 * - Transaction hash can be changed by changing nonce.
 * - If an identical transaction hash already exists, then the change fails. In this situation, a transaction can be created again by arbitrarily changing the nonce.
 */
export class CreateWorldTransaction {
    readonly networkId: NetworkId;
    readonly shardId: number;
    readonly nonce: number;
    readonly owners: PlatformAddress[];

    readonly type = "createWorld";

    constructor(data: CreateWorldData) {
        const { networkId, shardId, nonce, owners } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.nonce = nonce;
        this.owners = owners;
    }

    static fromJSON(obj: any) {
        const { data: { networkId, shardId, nonce, owners } } = obj;
        return new this({
            networkId,
            shardId,
            nonce,
            owners,
        });
    }

    toJSON() {
        const { networkId, shardId, nonce, owners } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                nonce,
                owners,
            }
        };
    }

    toEncodeObject() {
        const { networkId, shardId, nonce, owners } = this;
        return [
            1,
            networkId,
            shardId,
            nonce,
            owners.map(owner => owner.getAccountId().toEncodeObject())
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
