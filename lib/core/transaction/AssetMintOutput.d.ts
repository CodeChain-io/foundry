/// <reference types="node" />
import { AssetTransferAddress, H160 } from "codechain-primitives/lib";
import { U256 } from "../U256";
export declare class AssetMintOutput {
    /**
     * Create an AssetMintOutput from an AssetMintOutput JSON object.
     * @param data An AssetMintOutput JSON object.
     * @returns An AssetMintOutput.
     */
    static fromJSON(data: {
        lockScriptHash: string;
        parameters: Buffer[];
        amount?: string | null;
    }): AssetMintOutput;
    readonly lockScriptHash: H160;
    readonly parameters: Buffer[];
    readonly amount?: U256 | null;
    /**
     * @param data.lockScriptHash A lock script hash of the output.
     * @param data.parameters Parameters of the output.
     * @param data.amount Asset amount of the output.
     */
    constructor(data: {
        lockScriptHash: H160;
        parameters: Buffer[];
        amount?: U256 | null;
    } | {
        recipient: AssetTransferAddress;
        amount?: U256 | null;
    });
    /**
     * Convert to an AssetMintOutput JSON object.
     * @returns An AssetMintOutput JSON object.
     */
    toJSON(): {
        lockScriptHash: string;
        parameters: number[][];
        amount: string | number | undefined;
    };
}
