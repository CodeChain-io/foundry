import { H160 } from ".";

export type AssetSchemeData = {
    metadata: string;
    amount: number;
    registrar: H160 | null;
};
/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
export class AssetScheme {
    metadata: string;
    amount: number;
    registrar: H160 | null;

    constructor(data: AssetSchemeData) {
        this.metadata = data.metadata;
        this.registrar = data.registrar;
        this.amount = data.amount;
    }

    static fromJSON(data: any) {
        return new AssetScheme(data);
    }

    toJSON() {
        const { metadata, amount, registrar } = this;
        return {
            metadata,
            amount,
            registrar: registrar === null ? null : registrar.value
        };
    }
}
