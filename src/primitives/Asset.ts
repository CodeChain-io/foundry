import { H256 } from ".";

export interface AssetValues {
    asset_type: H256;
    lock_script_hash: H256;
    parameters: Buffer[];
    amount: number;
}

export class Asset {
    asset_type: H256;
    lock_script_hash: H256;
    parameters: Buffer[];
    amount: number;

    constructor(data: AssetValues) {
        this.asset_type = data.asset_type;
        this.lock_script_hash = data.lock_script_hash;
        this.parameters = data.parameters
        this.amount = data.amount;
    }
}
