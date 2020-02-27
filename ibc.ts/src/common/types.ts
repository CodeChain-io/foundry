export interface IBCQueryResult<T> {
    data: T | null;
    proof: string;
}

export type IBCHeader = string;
