/// <reference types="node" />
import { PlatformAddress } from "codechain-primitives";
import { H160 } from "./H160";
import { H256 } from "./H256";
import { H512 } from "./H512";
import { Parcel } from "./Parcel";
import { NetworkId } from "./types";
import { U256 } from "./U256";
/**
 * A [Parcel](parcel.html) signed by a private key. It is possible to request
 * the CodeChain network to process this parcel with the
 * [sendSignedParcel](chainrpc.html#sendsignedparcel) function.
 *
 * Parcels signed with a regular key has the same effect as those signed with
 * the original key. The original key is the key of the account that registered
 * the regular key.
 *
 * If any of the following is true, the Parcel will not be processed:
 * - The Parcel's processing fee is less than 10.
 * - A network ID is not identical.
 * - A seq is not identical to the signer's seq.
 */
export declare class SignedParcel {
    /**
     * Create a SignedParcel from a SignedParcel JSON object.
     * @param data A SignedParcel JSON object.
     * @returns A SignedParcel.
     */
    static fromJSON(data: any): SignedParcel;
    /**
     * Convert r, s, v values of an ECDSA signature to a string.
     * @param params.r The r value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.s The s value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.v The recovery parameter of an ECDSA signature.
     * @returns A 65 byte hexadecimal string.
     */
    static convertRsvToSignatureString(params: {
        r: string;
        s: string;
        v: number;
    }): string;
    private static convertSignatureStringToRsv;
    unsigned: Parcel;
    v: number;
    r: U256;
    s: U256;
    blockNumber: number | null;
    blockHash: H256 | null;
    parcelIndex: number | null;
    /**
     * @param unsigned A Parcel.
     * @param sig An ECDSA signature which is a 65 byte hexadecimal string.
     * @param blockNumber The block number of the block that contains the parcel.
     * @param blockHash The hash of the block that contains the parcel.
     * @param parcelIndex The index(location) of the parcel within the block.
     */
    constructor(unsigned: Parcel, sig: string, blockNumber?: number, blockHash?: H256, parcelIndex?: number);
    /**
     * Get the signature of a parcel.
     */
    signature(): {
        v: number;
        r: U256;
        s: U256;
    };
    /**
     * Convert to an object for RLP encoding.
     */
    toEncodeObject(): any[];
    /**
     * Convert to RLP bytes.
     */
    rlpBytes(): Buffer;
    /**
     * Get the hash of a parcel.
     * @returns A parcel hash.
     */
    hash(): H256;
    /**
     * Get the account ID of a parcel's signer.
     * @returns An account ID.
     * @deprecated
     */
    getSignerAccountId(): H160;
    /**
     * Get the platform address of a parcel's signer.
     * @returns A PlatformAddress.
     * @deprecated
     */
    getSignerAddress(params: {
        networkId: NetworkId;
    }): PlatformAddress;
    /**
     * Get the public key of a parcel's signer.
     * @returns A public key.
     */
    getSignerPublic(): H512;
    /**
     * Convert to a SignedParcel JSON object.
     * @returns A SignedParcel JSON object.
     */
    toJSON(): {
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
    };
}
