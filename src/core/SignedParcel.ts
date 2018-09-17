import { PlatformAddress } from "codechain-primitives";
import * as _ from "lodash";

import { blake256, recoverEcdsa, ripemd160 } from "../utils";

import { H160 } from "./H160";
import { H256 } from "./H256";
import { H512 } from "./H512";
import { Parcel } from "./Parcel";
import { U256 } from "./U256";

const RLP = require("rlp");

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
 * - A nonce is not identical to the signer's nonce.
 */
export class SignedParcel {
    // FIXME: any
    /**
     * Create a SignedParcel from a SignedParcel JSON object.
     * @param data A SignedParcel JSON object.
     * @returns A SignedParcel.
     */
    public static fromJSON(data: any) {
        const { sig, blockNumber, blockHash, parcelIndex } = data;
        if (typeof sig !== "string") {
            throw Error("Unexpected type of sig");
        }
        if (blockNumber) {
            return new SignedParcel(
                Parcel.fromJSON(data),
                sig,
                blockNumber,
                new H256(blockHash),
                parcelIndex
            );
        } else {
            return new SignedParcel(Parcel.fromJSON(data), sig);
        }
    }

    /**
     * Convert r, s, v values of an ECDSA signature to a string.
     * @param params.r The r value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.s The s value of an ECDSA signature, which is up to 32 bytes of hexadecimal string.
     * @param params.v The recovery parameter of an ECDSA signature.
     * @returns A 65 byte hexadecimal string.
     */
    public static convertRsvToSignatureString(params: {
        r: string;
        s: string;
        v: number;
    }) {
        const { r, s, v } = params;
        return `0x${_.padStart(r, 64, "0")}${_.padStart(
            s,
            64,
            "0"
        )}${_.padStart(v.toString(16), 2, "0")}`;
    }

    private static convertSignatureStringToRsv(
        signature: string
    ): { r: string; s: string; v: number } {
        if (signature.startsWith("0x")) {
            signature = signature.substr(2);
        }
        const r = `0x${signature.substr(0, 64)}`;
        const s = `0x${signature.substr(64, 64)}`;
        const v = Number.parseInt(signature.substr(128, 2), 16);
        return { r, s, v };
    }
    public unsigned: Parcel;
    public v: number;
    public r: U256;
    public s: U256;
    public blockNumber: number | null;
    public blockHash: H256 | null;
    public parcelIndex: number | null;

    /**
     * @param unsigned A Parcel.
     * @param sig An ECDSA signature which is a 65 byte hexadecimal string.
     * @param blockNumber The block number of the block that contains the parcel.
     * @param blockHash The hash of the block that contains the parcel.
     * @param parcelIndex The index(location) of the parcel within the block.
     */
    constructor(
        unsigned: Parcel,
        sig: string,
        blockNumber?: number,
        blockHash?: H256,
        parcelIndex?: number
    ) {
        this.unsigned = unsigned;
        const { r, s, v } = SignedParcel.convertSignatureStringToRsv(sig);
        this.v = v;
        this.r = new U256(r);
        this.s = new U256(s);
        this.blockNumber = blockNumber === undefined ? null : blockNumber;
        this.blockHash = blockHash || null;
        this.parcelIndex = parcelIndex === undefined ? null : parcelIndex;
    }

    /**
     * Get the signature of a parcel.
     */
    public signature() {
        const { v, r, s } = this;
        return { v, r, s };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject(): any[] {
        const {
            unsigned: { nonce, fee, action, networkId },
            v,
            r,
            s
        } = this;
        const sig = `0x${_.padStart(r.value.toString(16), 64, "0")}${_.padStart(
            s.value.toString(16),
            64,
            "0"
        )}${_.padStart(v.toString(16), 2, "0")}`;
        if (!nonce || !fee) {
            throw Error("Nonce and fee in the parcel must be present");
        }
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            networkId,
            action.toEncodeObject(),
            sig
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of a parcel.
     * @returns A parcel hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Get the account ID of a parcel's signer.
     * @returns An account ID.
     * @deprecated
     */
    public getSignerAccountId(): H160 {
        const { r, s, v, unsigned } = this;
        const publicKey = recoverEcdsa(unsigned.hash().value, {
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        return new H160(ripemd160(blake256(publicKey)));
    }

    /**
     * Get the platform address of a parcel's signer.
     * @returns A PlatformAddress.
     * @deprecated
     */
    public getSignerAddress(): PlatformAddress {
        return PlatformAddress.fromAccountId(this.getSignerAccountId());
    }

    /**
     * Get the public key of a parcel's signer.
     * @returns A public key.
     */
    public getSignerPublic(): H512 {
        const { r, s, v, unsigned } = this;
        return new H512(
            recoverEcdsa(unsigned.hash().value, {
                r: r.value.toString(16),
                s: s.value.toString(16),
                v
            })
        );
    }

    /**
     * Convert to a SignedParcel JSON object.
     * @returns A SignedParcel JSON object.
     */
    public toJSON() {
        const {
            blockNumber,
            blockHash,
            parcelIndex,
            unsigned: { nonce, fee, networkId, action },
            v,
            r,
            s
        } = this;
        const sig = SignedParcel.convertRsvToSignatureString({
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        if (!nonce || !fee) {
            throw Error("Nonce and fee in the parcel must be present");
        }
        return {
            blockNumber,
            blockHash: blockHash === null ? null : blockHash.value,
            parcelIndex,
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            networkId,
            action: action.toJSON(),
            sig,
            hash: this.hash().value
        };
    }
}
