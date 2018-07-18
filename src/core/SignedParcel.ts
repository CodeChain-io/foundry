import * as _ from "lodash";

import { U256 } from "./U256";
import { H160 } from "./H160";
import { H256 } from "./H256";
import { Parcel } from "./Parcel";
import { blake256, ripemd160, recoverEcdsa } from "../utils";

const RLP = require("rlp");

/**
 * A [Parcel](parcel.html) signed by a private key. It is possible to request
 * processing on the CodeChain network with the
 * [sendSignedParcel](chainrpc.html#sendsignedparcel) function.
 *
 * Parcels signed with a regular key has the same effect as those signed with
 * the original key. The original key is the key of the account that registered
 * the regular key.
 *
 * If any of the following is true, the Parcel will not be processed:
 * the Parcel's processing fee is less than 10, network ID is not identical, or
 * the nonce is not identical.
 *
 * - When including a Payment transaction, the payment's sender and the parcel's
 * signer must be identical.
 * - When including a SetRegularKey transaction, the transaction's address and
 * the parcel's signer must be identical.
 * - If the asset type that is being transferred from AssetTransferTransaction
 * has a registrar, the registrar must be identical to the parcel's signer.
 * If any of the transactions above have an invalid signer for any of the
 * conditions, then individual transactions will fail.
 */
export class SignedParcel {
    unsigned: Parcel;
    v: number;
    r: U256;
    s: U256;
    blockNumber: number | null;
    blockHash: H256 | null;
    parcelIndex: number | null;

    constructor(unsigned: Parcel, sig: string, blockNumber?: number, blockHash?: H256, parcelIndex?: number) {
        this.unsigned = unsigned;
        const { r, s, v } = SignedParcel.convertSignatureStringToRsv(sig);
        this.v = v;
        this.r = new U256(r);
        this.s = new U256(s);
        this.blockNumber = blockNumber === undefined ? null : blockNumber;
        this.blockHash = blockHash || null;
        this.parcelIndex = parcelIndex === undefined ? null : parcelIndex;
    }

    signature() {
        const { v, r, s } = this;
        return { v, r, s };
    }

    toEncodeObject(): Array<any> {
        const { unsigned: { nonce, fee, action, networkId }, v, r, s } = this;
        const sig = `0x${_.padStart(r.value.toString(16), 64, "0")}${_.padStart(s.value.toString(16), 64, "0")}${_.padStart(v.toString(16), 2, "0")}`;
        if (!nonce || !fee) {
            throw "Nonce and fee in the parcel must be present";
        }
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            networkId.toEncodeObject(),
            action.toEncodeObject(),
            sig
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    getSender(): H160 {
        const { r, s, v, unsigned } = this;
        const publicKey = recoverEcdsa(unsigned.hash().value, {
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        return new H160(ripemd160(blake256(publicKey)));
    }

    static fromJSON(data: any) {
        const { sig, blockNumber, blockHash, parcelIndex } = data;
        if (typeof sig !== "string") {
            throw "Unexpected type of sig";
        }
        if (blockNumber) {
            return new SignedParcel(Parcel.fromJSON(data), sig, blockNumber, new H256(blockHash), parcelIndex);
        } else {
            return new SignedParcel(Parcel.fromJSON(data), sig);
        }
    }

    toJSON() {
        const { blockNumber, blockHash, parcelIndex,
            unsigned: { nonce, fee, networkId, action }, v, r, s } = this;
        const sig = SignedParcel.convertRsvToSignatureString({
            r: r.value.toString(16),
            s: s.value.toString(16),
            v
        });
        if (!nonce || !fee) {
            throw "Nonce and fee in the parcel must be present";
        }
        return {
            blockNumber,
            blockHash: blockHash === null ? null : blockHash.value,
            parcelIndex,
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            networkId: networkId.value.toNumber(),
            action: action.toJSON(),
            sig,
            hash: this.hash().value,
        };
    }

    private static convertSignatureStringToRsv(signature: string): { r: string, s: string, v: number } {
        const r = `0x${signature.substr(2, 64)}`;
        const s = `0x${signature.substr(66, 64)}`;
        const v = Number.parseInt(signature.substr(130, 2), 16);
        return { r, s, v };
    }

    static convertRsvToSignatureString(params: { r: string, s: string, v: number }) {
        const { r, s, v } = params;
        return `0x${_.padStart(r, 64, "0")}${_.padStart(s, 64, "0")}${_.padStart(v.toString(16), 2, "0")}`;
    }
}
