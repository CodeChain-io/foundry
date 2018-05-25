import { H160 } from ".";

export interface AssetSchemeValues {
    metadata: string;
    amount: number;
    registrar: H160 | null;
}

export class AssetScheme {
    metadata: string;
    amount: number;
    registrar: H160 | null;

    constructor(data: AssetSchemeValues) {
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
