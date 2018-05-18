import { H160, H256 } from "../index";

export type AssetMintTransactionData = {
    metadata: string;
    lockScriptHash: H256;
    parameters: Buffer[];
    amount: number | null;
    registrar: H160 | null;
};

export class AssetMintTransaction {
    private data: AssetMintTransactionData;

    constructor(data: AssetMintTransactionData) {
        this.data = data;
    }

    toEncodeObject() {
        const { metadata, lockScriptHash, parameters, amount, registrar } = this.data;
        return [
            3,
            metadata,
            lockScriptHash.toEncodeObject(),
            parameters,
            amount ? [amount] : [],
            registrar ? [registrar] : []
        ];
    }
}
