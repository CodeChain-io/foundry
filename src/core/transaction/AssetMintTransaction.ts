import { Buffer } from "buffer";

import { PlatformAddress } from "../../key/classes";

import { H160 } from "../H160";
import { H256 } from "../H256";
import { blake256, blake256WithKey } from "../../utils";
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
    registrar: PlatformAddress | H160 | string | null;
    nonce: number;
};

/**
 * Creates a new asset type and that asset itself.
 *
 * The owner of the new asset created can be assigned by a lock script hash and parameters.
 *  - A metadata is a string that explains the asset's type.
 *  - Amount defines the quantity of asset to be created. If set as null, it
 *  will be set as the maximum value of a 64-bit unsigned integer by default.
 *  - If registrar exists, the registrar must be the Signer of the Parcel when
 *  sending the created asset through AssetTransferTransaction.
 *  - A transaction hash can be changed by changing nonce.
 *  - If an identical transaction hash already exists, then the change fails. In
 *  this situation, a transaction can be created again by arbitrarily changing
 *  the nonce.
 */
export class AssetMintTransaction {
    readonly networkId: number;
    readonly shardId: number;
    readonly metadata: string;
    readonly output: {
        lockScriptHash: H256;
        parameters: Buffer[];
        amount: number | null;
    };
    readonly registrar: H160 | null;
    readonly nonce: number;
    readonly type = "assetMint";

    /**
     * @param data.networkId A network ID of the transaction.
     * @param data.shardId A shard ID of the transaction.
     * @param data.metadata A metadata of the asset.
     * @param data.output.lockScriptHash A lock script hash of the output.
     * @param data.output.parameters Parameters of the output.
     * @param data.output.amount Asset amount of the output.
     * @param data.registrar A registrar of the asset.
     * @param data.nonce A nonce of the transaction.
     */
    constructor(data: AssetMintTransactionData) {
        const { networkId, shardId, metadata, output, registrar, nonce } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.metadata = metadata;
        this.output = output;
        this.registrar = registrar === null ? null : PlatformAddress.ensureAccount(registrar);
        this.nonce = nonce;
    }

    /**
     * Create an AssetMintTransaction from an AssetMintTransaction JSON object.
     * @param data An AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction.
     */
    static fromJSON(data: any) {
        const { data: { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar, nonce } } = data;
        return new this({
            networkId,
            shardId,
            metadata,
            output: {
                lockScriptHash: new H256(lockScriptHash),
                parameters: parameters.map((p: Array<number>) => Buffer.from(p)),
                amount: amount === null ? null : amount,
            },
            registrar: registrar === null ? null : new H160(registrar),
            nonce,
        });
    }

    /**
     * Convert to an AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction JSON object.
     */
    toJSON() {
        const { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar, nonce } = this;
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
            }
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        const { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar, nonce } = this;
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

    /**
     * Convert to RLP bytes.
     */
    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of an AssetMintTransaction.
     * @returns A transaction hash.
     */
    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    getMintedAsset(): Asset {
        const { lockScriptHash, parameters, amount } = this.output;
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

    /**
     * Get the asset scheme of this transaction.
     * @return An AssetScheme.
     */
    getAssetScheme(): AssetScheme {
        const { networkId, shardId, metadata, output: { amount }, registrar } = this;
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

    /**
     * Get the address of the asset scheme. An asset scheme address equals to an
     * asset type value.
     * @returns An asset scheme address which is H256.
     */
    getAssetSchemeAddress(): H256 {
        const { shardId } = this;
        const blake = blake256WithKey(this.hash().value, new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ]));
        const shardPrefix = convertU16toHex(shardId);
        const worldPrefix = "0000";
        const prefix = `5300${shardPrefix}${worldPrefix}`;
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }

    /**
     * Get the asset address of the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress(): H256 {
        const { shardId } = this;
        const blake = blake256WithKey(this.hash().value, new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]));
        const shardPrefix = convertU16toHex(shardId);
        const worldPrefix = "0000";
        const prefix = `4100${shardPrefix}${worldPrefix}`;
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
}

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xFF).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xFF).toString(16)).slice(-2);
    return hi + lo;
}
