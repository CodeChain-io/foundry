import { Transaction } from "../transaction/Transaction";
export declare class AssetTransaction {
    transaction: Transaction;
    constructor(input: {
        transaction: Transaction;
    });
    toEncodeObject(): any[];
    toJSON(): {
        action: string;
        transaction: {
            type: string;
            data: {
                networkId: string;
                burns: {
                    prevOut: {
                        transactionHash: string;
                        index: number;
                        assetType: string;
                        amount: string | number;
                    };
                    timelock: import("../transaction/AssetTransferInput").Timelock | null;
                    lockScript: number[];
                    unlockScript: number[];
                }[];
                inputs: {
                    prevOut: {
                        transactionHash: string;
                        index: number;
                        assetType: string;
                        amount: string | number;
                    };
                    timelock: import("../transaction/AssetTransferInput").Timelock | null;
                    lockScript: number[];
                    unlockScript: number[];
                }[];
                outputs: {
                    lockScriptHash: string;
                    parameters: number[][];
                    assetType: string;
                    amount: string | number;
                }[];
            };
        } | {
            type: string;
            data: {
                networkId: string;
                shardId: number;
                metadata: string;
                output: {
                    lockScriptHash: string;
                    parameters: number[][];
                    amount: string | number | undefined;
                };
                registrar: string | null;
            };
        } | {
            type: string;
            data: {
                networkId: string;
                shardId: number;
                metadata: string;
                registrar: import("codechain-primitives/lib/address/PlatformAddress").PlatformAddress | null;
                output: {
                    lockScriptHash: string;
                    parameters: number[][];
                    amount: string | number | undefined;
                };
                inputs: {
                    prevOut: {
                        transactionHash: string;
                        index: number;
                        assetType: string;
                        amount: string | number;
                    };
                    timelock: import("../transaction/AssetTransferInput").Timelock | null;
                    lockScript: number[];
                    unlockScript: number[];
                }[];
            };
        } | {
            type: string;
            data: {
                input: {
                    prevOut: {
                        transactionHash: string;
                        index: number;
                        assetType: string;
                        amount: string | number;
                    };
                    timelock: import("../transaction/AssetTransferInput").Timelock | null;
                    lockScript: number[];
                    unlockScript: number[];
                };
                outputs: {
                    lockScriptHash: string;
                    parameters: number[][];
                    assetType: string;
                    amount: string | number;
                }[];
                networkId: string;
            };
        };
    };
}
