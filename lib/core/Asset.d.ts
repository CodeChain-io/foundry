/// <reference types="node" />
import { AssetTransferAddress, H160 } from "codechain-primitives";
import { H256 } from "./H256";
import { AssetOutPoint } from "./transaction/AssetOutPoint";
import { AssetTransferInput, Timelock } from "./transaction/AssetTransferInput";
import { AssetTransferTransaction } from "./transaction/AssetTransferTransaction";
import { NetworkId } from "./types";
import { U256 } from "./U256";
export interface AssetData {
    assetType: H256;
    lockScriptHash: H160;
    parameters: Buffer[];
    amount: U256;
    transactionHash: H256;
    transactionOutputIndex: number;
}
/**
 * Object created as an AssetMintTransaction or AssetTransferTransaction.
 */
export declare class Asset {
    static fromJSON(data: any): Asset;
    readonly assetType: H256;
    readonly lockScriptHash: H160;
    readonly parameters: Buffer[];
    readonly amount: U256;
    readonly outPoint: AssetOutPoint;
    constructor(data: AssetData);
    toJSON(): {
        assetType: string;
        lockScriptHash: string;
        parameters: Buffer[];
        amount: string | number;
        transactionHash: string;
        transactionOutputIndex: number;
    };
    createTransferInput(options?: {
        timelock: Timelock | null;
    }): AssetTransferInput;
    createTransferTransaction(params: {
        recipients?: Array<{
            address: AssetTransferAddress | string;
            amount: U256;
        }>;
        timelock?: null | Timelock;
        networkId: NetworkId;
    }): AssetTransferTransaction;
}
