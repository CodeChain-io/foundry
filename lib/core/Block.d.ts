/// <reference types="node" />
import { PlatformAddress } from "codechain-primitives";
import { H256 } from "./H256";
import { SignedParcel } from "./SignedParcel";
import { U256 } from "./U256";
export interface BlockData {
    parentHash: H256;
    timestamp: number;
    number: number;
    author: PlatformAddress;
    extraData: Buffer;
    parcelsRoot: H256;
    stateRoot: H256;
    invoicesRoot: H256;
    score: U256;
    seal: Buffer[];
    hash: H256;
    parcels: SignedParcel[];
}
/**
 * Block is the unit of processes being handled by CodeChain. Contains information related to SignedParcel's list and block creation.
 */
export declare class Block {
    static fromJSON(data: any): Block;
    parentHash: H256;
    timestamp: number;
    number: number;
    author: PlatformAddress;
    extraData: Buffer;
    parcelsRoot: H256;
    stateRoot: H256;
    invoicesRoot: H256;
    score: U256;
    seal: Buffer[];
    hash: H256;
    parcels: SignedParcel[];
    constructor(data: BlockData);
    toJSON(): {
        parentHash: string;
        timestamp: number;
        number: number;
        author: string;
        extraData: Buffer;
        parcelsRoot: string;
        stateRoot: string;
        invoicesRoot: string;
        score: string;
        seal: Buffer[];
        hash: string;
        parcels: {
            blockNumber: number | null;
            blockHash: string | null;
            parcelIndex: number | null;
            seq: string;
            fee: string;
            networkId: string;
            action: {
                action: string;
            };
            sig: string;
            hash: string;
        }[];
    };
}
