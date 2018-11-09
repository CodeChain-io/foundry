"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const buffer_1 = require("buffer");
const codechain_primitives_1 = require("codechain-primitives");
const utils_1 = require("../../utils");
const Asset_1 = require("../Asset");
const AssetScheme_1 = require("../AssetScheme");
const H256_1 = require("../H256");
const AssetMintOutput_1 = require("./AssetMintOutput");
const RLP = require("rlp");
/**
 * Creates a new asset type and that asset itself.
 *
 * The owner of the new asset created can be assigned by a lock script hash and parameters.
 *  - A metadata is a string that explains the asset's type.
 *  - Amount defines the quantity of asset to be created. If set as null, it
 *  will be set as the maximum value of a 64-bit unsigned integer by default.
 *  - If registrar exists, the registrar must be the Signer of the Parcel when
 *  sending the created asset through AssetTransferTransaction.
 */
class AssetMintTransaction {
    /**
     * @param data.networkId A network ID of the transaction.
     * @param data.shardId A shard ID of the transaction.
     * @param data.metadata A metadata of the asset.
     * @param data.output.lockScriptHash A lock script hash of the output.
     * @param data.output.parameters Parameters of the output.
     * @param data.output.amount Asset amount of the output.
     * @param data.registrar A registrar of the asset.
     */
    constructor(data) {
        this.type = "assetMint";
        const { networkId, shardId, metadata, output, registrar } = data;
        this.networkId = networkId;
        this.shardId = shardId;
        this.metadata = metadata;
        this.output = new AssetMintOutput_1.AssetMintOutput(output);
        this.registrar = registrar;
    }
    /**
     * Create an AssetMintTransaction from an AssetMintTransaction JSON object.
     * @param data An AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction.
     */
    static fromJSON(data) {
        const { data: { networkId, shardId, metadata, output, registrar } } = data;
        return new this({
            networkId,
            shardId,
            metadata,
            output: AssetMintOutput_1.AssetMintOutput.fromJSON(output),
            registrar: registrar === null
                ? null
                : codechain_primitives_1.PlatformAddress.fromString(registrar)
        });
    }
    /**
     * Convert to an AssetMintTransaction JSON object.
     * @returns An AssetMintTransaction JSON object.
     */
    toJSON() {
        const { networkId, shardId, metadata, output, registrar } = this;
        return {
            type: this.type,
            data: {
                networkId,
                shardId,
                metadata,
                output: output.toJSON(),
                registrar: registrar === null ? null : registrar.toString()
            }
        };
    }
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject() {
        const { networkId, shardId, metadata, output: { lockScriptHash, parameters, amount }, registrar } = this;
        return [
            3,
            networkId,
            shardId,
            metadata,
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => buffer_1.Buffer.from(parameter)),
            amount != null ? [amount.toEncodeObject()] : [],
            registrar ? [registrar.getAccountId().toEncodeObject()] : []
        ];
    }
    /**
     * Convert to RLP bytes.
     */
    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }
    /**
     * Get the hash of an AssetMintTransaction.
     * @returns A transaction hash.
     */
    hash() {
        return new H256_1.H256(utils_1.blake256(this.rlpBytes()));
    }
    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    getMintedAsset() {
        const { lockScriptHash, parameters, amount } = this.output;
        // FIXME: need U64 to be implemented or use U256
        if (amount == null) {
            throw Error("not implemented");
        }
        return new Asset_1.Asset({
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
    getAssetScheme() {
        const { networkId, shardId, metadata, output: { amount }, registrar } = this;
        // FIXME: need U64 to be implemented or use U256
        if (amount == null) {
            throw Error("not implemented");
        }
        return new AssetScheme_1.AssetScheme({
            networkId,
            shardId,
            metadata,
            amount,
            registrar,
            pool: []
        });
    }
    /**
     * Get the address of the asset scheme. An asset scheme address equals to an
     * asset type value.
     * @returns An asset scheme address which is H256.
     */
    getAssetSchemeAddress() {
        const { shardId } = this;
        const blake = utils_1.blake256WithKey(this.hash().value, new Uint8Array([
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff,
            0xff
        ]));
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `5300${shardPrefix}`;
        return new H256_1.H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
    /**
     * Get the asset address of the output.
     * @returns An asset address which is H256.
     */
    getAssetAddress() {
        const { shardId } = this;
        const blake = utils_1.blake256WithKey(this.hash().value, new Uint8Array([
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00
        ]));
        const shardPrefix = convertU16toHex(shardId);
        const prefix = `4100${shardPrefix}`;
        return new H256_1.H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }
}
exports.AssetMintTransaction = AssetMintTransaction;
function convertU16toHex(id) {
    const hi = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
