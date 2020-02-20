import RPC from "foundry-rpc";
import { SDK } from "../../sdk/src";
import { PlatformAddress } from "../../sdk/src/core/classes";

export interface TermMetadata {
    lastTermFinishedBlockNumber: number;
    currentTermId: number;
}

export async function getTermMetadata(
    rpc: RPC,
    blockNumber?: number
): Promise<TermMetadata | null> {
    const result = await rpc.chain.getTermMetadata({blockNumber})
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
): Promise<PlatformAddress[] | null> {
    const result = await rpc.chain.getPossibleAuthors({blockNumber});
    if (result === null) {
        return null;
    }
    if (Array.isArray(result)) {
        return result.map(PlatformAddress.ensure);
    }
    throw new Error(`Expected PlatformAddress[], but got ${result}`);
}
