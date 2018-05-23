export interface AssetSchemeValues {
    metadata: string;
    parameters: any[];
    amount: number;
}

export class AssetScheme {
    metadata: string;
    parameters: any[];
    amount: number;

    constructor(data: AssetSchemeValues) {
        this.metadata = data.metadata;
        this.parameters = data.parameters;
        this.amount = data.amount;
    }
}