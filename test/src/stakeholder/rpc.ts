import RPC from "foundry-rpc";
import { SDK } from "../sdk";
import { Address } from "../sdk/core/classes";

export interface TermMetadata {
    lastTermFinishedBlockNumber: number;
    currentTermId: number;
}

export async function getTermMetadata(
    rpc: RPC,
    blockNumber?: number
): Promise<TermMetadata | null> {
    const result = await rpc.chain.getTermMetadata({ blockNumber });
    if (result === null) {
        return null;
    }
    if (
        Array.isArray(result) &&
        result.length === 2 &&
        typeof result[0] === "number" &&
        typeof result[1] === "number"
    ) {
        return {
            lastTermFinishedBlockNumber: result[0],
            currentTermId: result[1]
        };
    }
    throw new Error(`Expected [number, number], but got ${result}`);
}

export async function getPossibleAuthors(
    rpc: RPC,
    blockNumber?: number
): Promise<Address[] | null> {
    const result = await rpc.chain.getPossibleAuthors({ blockNumber });
    if (result === null) {
        return null;
    }
    if (Array.isArray(result)) {
        return result.map(Address.ensure);
    }
    throw new Error(`Expected address[], but got ${result}`);
}
