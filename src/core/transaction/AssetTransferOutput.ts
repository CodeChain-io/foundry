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
    private readonly data: AssetTransferOutputData;

    constructor(data: AssetTransferOutputData) {
        this.data = data;
    }

    toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this.data;
        return [lockScriptHash.toEncodeObject(), parameters, assetType.toEncodeObject(), amount];
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
        const { lockScriptHash, parameters, assetType, amount } = this.data;
        return {
            lockScriptHash: lockScriptHash.value,
            parameters,
            assetType: assetType.value,
            amount,
        };
    }

    shardId(): number {
        const { assetType } = this.data;
        return parseInt(assetType.value.slice(8, 16), 16);
    }
}
