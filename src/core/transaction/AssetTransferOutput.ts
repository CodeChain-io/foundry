import { Buffer } from "buffer";

import { H256 } from "../H256";

export type AssetTransferOutputData = {
    lockScriptHash: H256;
    parameters: Buffer[];
    assetType: H256;
    amount: number;
};
/**
 * AssetTransferOutput consists of lockScriptHash and parameters, which mark ownership of the asset, and asset type and amount, which indicate the asset's type and quantity.
 */
export class AssetTransferOutput {
    readonly lockScriptHash: H256;
    readonly parameters: Buffer[];
    readonly assetType: H256;
    readonly amount: number;

    constructor(data: AssetTransferOutputData) {
        const { lockScriptHash, parameters, assetType, amount } = data;
        this.lockScriptHash = lockScriptHash;
        this.parameters = parameters;
        this.assetType = assetType;
        this.amount = amount;
    }

    toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this;
        return [
            lockScriptHash.toEncodeObject(),
            parameters.map(parameter => Buffer.from(parameter)),
            assetType.toEncodeObject(),
            amount
        ];
    }

    static fromJSON(data: any) {
        const { lockScriptHash, parameters, assetType, amount } = data;
        return new this({
            lockScriptHash: new H256(lockScriptHash),
            parameters,
            assetType: new H256(assetType),
            amount,
        });
    }

    toJSON() {
        const { lockScriptHash, parameters, assetType, amount } = this;
        return {
            lockScriptHash: lockScriptHash.value,
            parameters: parameters.map(parameter => Buffer.from(parameter)),
            assetType: assetType.value,
            amount,
        };
    }

    shardId(): number {
        const { assetType } = this;
        return parseInt(assetType.value.slice(4, 8), 16);
    }
}
