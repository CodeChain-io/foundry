import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";

import { U64 } from "../U64";

export interface AssetTransferOutputJSON {
    lockScriptHash: string;
    parameters: string[];
    assetType: string;
    shardId: number;
    quantity: string;
}

export interface AssetTransferOutputData {
    lockScriptHash: H160;
    parameters: Buffer[];
    assetType: H160;
    shardId: number;
    quantity: U64;
}

export interface AssetTransferOutputAddressData {
    recipient: AssetTransferAddress;
    assetType: H160;
    shardId: number;
    quantity: U64;
}

/**
 * An AssetTransferOutput consists of:
 *  - A lock script hash and parameters, which mark ownership of the asset.
 *  - An asset type and quantity, which indicate the asset's type and quantity.
 */
export class AssetTransferOutput {
    /**
     * Create an AssetTransferOutput from an AssetTransferOutput JSON object.
     * @param data An AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput.
     */
    public static fromJSON(data: AssetTransferOutputJSON) {
        const {
            lockScriptHash,
            parameters,
            assetType,
            shardId,
            quantity
        } = data;
        return new AssetTransferOutput({
            lockScriptHash: H160.ensure(lockScriptHash),
            parameters: parameters.map((p: string) => Buffer.from(p, "hex")),
            assetType: H160.ensure(assetType),
            shardId,
            quantity: U64.ensure(quantity)
        });
    }
    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly assetType: H160;
    public readonly shardId: number;
    public readonly quantity: U64;

    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.assetType An asset type of the output.
     * @param data.shardId A shard ID of the output.
     * @param data.quantity An asset quantity of the output.
     */
    constructor(
        data: AssetTransferOutputData | AssetTransferOutputAddressData
    ) {
        if ("recipient" in data) {
            // FIXME: Clean up by abstracting the standard scripts
            const { type, payload } = data.recipient;
            if ("pubkeys" in payload) {
                throw Error("Multisig payload is not supported yet");
            }
            switch (type) {
                case 0x00: // LOCK_SCRIPT_HASH ONLY
                    this.lockScriptHash = payload;
                    this.parameters = [];
                    break;
                case 0x01: // P2PKH
                    this.lockScriptHash = P2PKH.getLockScriptHash();
                    this.parameters = [Buffer.from(payload.value, "hex")];
                    break;
                case 0x02: // P2PKHBurn
                    this.lockScriptHash = P2PKHBurn.getLockScriptHash();
                    this.parameters = [Buffer.from(payload.value, "hex")];
                    break;
                default:
                    throw Error(
                        `Unexpected type of AssetTransferAddress: ${type}, ${
                            data.recipient
                        }`
                    );
            }
        } else {
            const { lockScriptHash, parameters } = data;
            this.lockScriptHash = lockScriptHash;
            this.parameters = parameters;
        }
        const { assetType, shardId, quantity } = data;
        this.assetType = assetType;
        this.shardId = shardId;
        this.quantity = quantity;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            lockScriptHash,
            parameters,
            assetType,
            shardId,
            quantity
        } = this;
        return [
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            assetType.toEncodeObject(),
            shardId,
            quantity.toEncodeObject()
        ];
    }

    /**
     * Convert to an AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput JSON object.
     */
    public toJSON(): AssetTransferOutputJSON {
        const {
            lockScriptHash,
            parameters,
            assetType,
            shardId,
            quantity
        } = this;
        return {
            lockScriptHash: lockScriptHash.toJSON(),
            parameters: parameters.map((p: Buffer) => p.toString("hex")),
            assetType: assetType.toJSON(),
            shardId,
            quantity: quantity.toJSON()
        };
    }
}
