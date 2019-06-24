import { PlatformAddress } from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";

export interface TermMetadata {
    lastTermFinishedBlockNumber: number;
    currentTermId: number;
}

export async function getTermMetadata(
    sdk: SDK,
    blockNumber?: number
): Promise<TermMetadata | null> {
    const result = await sdk.rpc.sendRpcRequest("chain_getTermMetadata", [
        blockNumber
    ]);
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
    sdk: SDK,
    blockNumber?: number
): Promise<PlatformAddress[] | null> {
    const result = await sdk.rpc.sendRpcRequest("chain_getPossibleAuthors", [
        blockNumber
    ]);
    if (result === null) {
        return null;
    }
    if (Array.isArray(result)) {
        return result.map(PlatformAddress.ensure);
    }
    throw new Error(`Expected PlatformAddress[], but got ${result}`);
}
