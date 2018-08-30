import { PlatformAddress } from "../../key/PlatformAddress";
import { blake256 } from "../../utils";
import { H256 } from "../H256";

const RLP = require("rlp");

type NetworkId = string;

export interface CreateWorldData {
    networkId: NetworkId;
    shardId: number;
    nonce: number;
    owners: PlatformAddress[];
}

/**
 * Creates a world
 *
 * - Transaction hash can be changed by changing nonce.
 * - If an identical transaction hash already exists, then the change fails. In this situation, a transaction can be created again by arbitrarily changing the nonce.
 */
export class CreateWorldTransaction {
    public static fromJSON(obj: any) {
        const {
            data: { networkId, shardId, nonce, owners }
        } = obj;
        return new this({
            networkId,
            shardId,
            nonce,
            owners
        });
    }
    public readonly networkId: NetworkId;
    public readonly shardId: number;
    public readonly nonce: number;
    public readonly owners: PlatformAddress[];

    public readonly type = "createWorld";

    constructor(data: CreateWorldData) {
        const { networkId, shardId, nonce, owners } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.nonce = nonce;
        this.owners = owners;
    }

    public toJSON() {
        const { networkId, shardId, nonce, owners } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                nonce,
                owners
            }
        };
    }

    public toEncodeObject() {
        const { networkId, shardId, nonce, owners } = this;
        return [
            1,
            networkId,
            shardId,
            nonce,
            owners.map(owner => owner.getAccountId().toEncodeObject())
        ];
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }
}
