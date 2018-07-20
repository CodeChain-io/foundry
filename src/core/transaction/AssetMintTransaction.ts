import { Buffer } from "buffer";

import { H160 } from "../H160";
import { H256 } from "../H256";
import { blake256WithKey, blake256 } from "../../utils";
import { Asset } from "../Asset";
import { AssetScheme } from "../AssetScheme";

const RLP = require("rlp");

export type AssetMintTransactionData = {
    networkId: number;
    shardId: number;
    metadata: string;
    output: {
        lockScriptHash: H256;
        parameters: Buffer[];
        amount: number | null;
    };
    registrar: H160 | null;
    nonce: number;
};

/**
 * Creates a new asset type and that asset itself.
 *
 * The owner of the new asset created can be assigned by lockScriptHash and parameters.
 * - metadata is a string that explains the asset's type.
 * - amount defines the quantity of asset to be created. If set as null, it will be set as the maximum value of a 64-bit unsigned integer by default.
 * - If registrar exists, the registrar must be the Signer of the Parcel when sending the created asset through AssetTransferTransaction.
 * - Transaction hash can be changed by changing nonce.
 * - If an identical transaction hash already exists, then the change fails. In this situation, a transaction can be created again by arbitrarily changing the nonce.
 */
export class AssetMintTransaction {
    private readonly data: AssetMintTransactionData;
    private readonly type = "assetMint";

    constructor(data: AssetMintTransactionData) {
        this.data = data;
    }

    static fromJSON(obj: any) {
        const { data: { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar, nonce } } = obj;
        return new this({
            networkId,
            shardId,
            metadata,
            output: {
                lockScriptHash: new H256(lockScriptHash),
                parameters,
                amount: amount === null ? null : amount,
            },
            registrar: registrar === null ? null : new H160(registrar),
            nonce,
        });
    }

    toJSON() {
        const { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar, nonce } = this.data;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                metadata,
                output: {
                    lockScriptHash: lockScriptHash.value,
                    parameters: parameters.map(parameter => Buffer.from(parameter)),
                    amount,
                },
                registrar: registrar === null ? null : registrar.value,
                nonce,
                hash: this.hash().value,
            }
        };
    }

    toEncodeObject() {
        const { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar, nonce } = this.data;
        return [
            3,
            networkId,
            shardId,
            metadata,
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            amount ? [amount] : [],
            registrar ? [registrar.toEncodeObject()] : [],
            nonce
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    getMintedAsset(): Asset {
        const { lockScriptHash, parameters, amount } = this.data.output;
        // FIXME: need U64 to be implemented or use U256
        if (amount === null) {
            throw "not implemented";
        }
        return new Asset({
            assetType: this.getAssetSchemeAddress(),
            lockScriptHash,
            parameters,
            amount,
            transactionHash: this.hash(),
            transactionOutputIndex: 0
        });
    }

    getAssetScheme(): AssetScheme {
        const { networkId, shardId, metadata, output: { amount }, registrar } = this.data;
        // FIXME: need U64 to be implemented or use U256
        if (amount === null) {
            throw "not implemented";
        }
        return new AssetScheme({
            networkId,
            shardId,
            metadata,
            amount,
            registrar
        });
    }

    getAssetSchemeAddress(): H256 {
        const { shardId } = this.data;
        const blake = blake256WithKey(this.hash().value, new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ]));
        const shardPrefix = convertU32toHex(shardId);
        const prefix = `53000000${shardPrefix}`;
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }

    getAssetAddress(): H256 {
        const { shardId } = this.data;
        const blake = blake256WithKey(this.hash().value, new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]));
        const shardPrefix = convertU32toHex(shardId);
        const prefix = `41000000${shardPrefix}`;
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
}

function convertU32toHex(id: number) {
    const shardId0: string = ("0" + ((id >> 24) & 0xFF).toString(16)).slice(-2);
    const shardId1: string = ("0" + ((id >> 16) & 0xFF).toString(16)).slice(-2);
    const shardId2: string = ("0" + ((id >> 8) & 0xFF).toString(16)).slice(-2);
    const shardId3: string = ("0" + ((id >> 0) & 0xFF).toString(16)).slice(-2);
    return shardId0 + shardId1 + shardId2 + shardId3;
}
