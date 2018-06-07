import { H256 } from ".";

export type AssetData = {
    assetType: H256;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number;
};

export class Asset {
    assetType: H256;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number;

    constructor(data: AssetData) {
        this.assetType = data.assetType;
        this.lockScriptHash = data.lockScriptHash;
        this.parameters = data.parameters;
        this.amount = data.amount;
    }

    static fromJSON(data: any) {
        // FIXME: use camelCase for all
        const { asset_type, lock_script_hash, parameters, amount } = data;
        return new Asset({
            assetType: new H256(asset_type),
            lockScriptHash: new H256(lock_script_hash),
            parameters,
            amount
        });
    }

    toJSON() {
        const { assetType, lockScriptHash, parameters, amount } = this;
        return {
            asset_type: assetType.value,
            lock_script_hash: lockScriptHash.value,
            parameters,
            amount,
        };
    }
}
