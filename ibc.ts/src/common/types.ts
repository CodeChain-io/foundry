export interface IBCQueryResult<T> {
    data: T | null;
    proof: string;
}

export type IBCHeader = string;

export interface ConnectionEnd {
    state: "INIT" | "TRYOPEN" | "OPEN";
    counterpartyConnectionIdentifier: string;
    counterpartyPrefix: string;
    clientIdentifier: string;
    counterpartyClientIdentifier: string;
}
