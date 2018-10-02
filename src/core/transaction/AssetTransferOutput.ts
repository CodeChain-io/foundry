import { Buffer } from "buffer";
import { AssetTransferAddress, H160 } from "codechain-primitives";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";

import { H256 } from "../H256";

export interface AssetTransferOutputData {
    lockScriptHash: H160;
    parameters: Buffer[];
    assetType: H256;
    amount: number;
}

export interface AssetTransferOutputAddressData {
    recipient: AssetTransferAddress;
    assetType: H256;
    amount: number;
}

/**
 * An AssetTransferOutput consists of:
 *  - A lock script hash and parameters, which mark ownership of the asset.
 *  - An asset type and amount, which indicate the asset's type and quantity.
 */
export class AssetTransferOutput {
    /**
     * Create an AssetTransferOutput from an AssetTransferOutput JSON object.
     * @param data An AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput.
     */
    public static fromJSON(data: AssetTransferOutputData) {
        const { lockScriptHash, parameters, assetType, amount } = data;
        return new this({
            lockScriptHash: H160.ensure(lockScriptHash),
            parameters: parameters.map((p: number[] | Buffer) =>
                Buffer.from(p)
            ),
            assetType: H256.ensure(assetType),
            amount
        });
    }
    public readonly lockScriptHash: H160;
    public readonly parameters: Buffer[];
    public readonly assetType: H256;
    public readonly amount: number;

    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.assetType An asset type of the output.
     * @param data.amount An asset amount of the output.
     */
    constructor(
        data: AssetTransferOutputData | AssetTransferOutputAddressData
    ) {
        if ("recipient" in data) {
            // FIXME: Clean up by abstracting the standard scripts
            const { type, payload } = data.recipient;
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
        const { assetType, amount } = data;
        this.assetType = assetType;
        this.amount = amount;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this;
        return [
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            assetType.toEncodeObject(),
            amount
        ];
    }

    /**
     * Convert to an AssetTransferOutput JSON object.
     * @returns An AssetTransferOutput JSON object.
     */
    public toJSON() {
        const { lockScriptHash, parameters, assetType, amount } = this;
        return {
            lockScriptHash: lockScriptHash.value,
            parameters: parameters.map(parameter => [...parameter]),
            assetType: assetType.value,
            amount
        };
    }

    /**
     * Get the shard ID.
     * @returns A shard ID.
     */
    public shardId(): number {
        const { assetType } = this;
        return parseInt(assetType.value.slice(4, 8), 16);
    }
}
