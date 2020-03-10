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

export interface ChannelEnd {
    state: "INIT" | "TRYOPEN" | "OPEN" | "CLOSED";
    ordering: "ORDERED" | "UNORDERED";
    counterpartyPortIdentifier: string;
    counterpartyChannelIdentifier: string;
    connectionHops: string[];
    version: string;
}
