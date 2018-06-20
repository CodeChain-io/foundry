import { U256 } from "./U256";
import { H256 } from "./H256";
import { Parcel } from "./Parcel";
import { blake256 } from "../utils";

const RLP = require("rlp");

/**
 * A [Parcel](parcel.html) signed by a private key. It is possible to request
 * processing on the CodeChain network with the
 * [sendSignedParcel](sdk.html#sendsignedparcel) function.
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

    constructor(unsigned: Parcel, v: number, r: U256, s: U256, blockNumber?: number, blockHash?: H256, parcelIndex?: number) {
        this.unsigned = unsigned;
        this.v = v;
        this.r = r;
        this.s = s;
        this.blockNumber = blockNumber || null;
        this.blockHash = blockHash || null;
        this.parcelIndex = parcelIndex || null;
    }

    signature() {
        const { v, r, s } = this;
        return { v, r, s };
    }

    toEncodeObject(): Array<any> {
        const { unsigned: { nonce, fee, action, networkId }, v, r, s } = this;
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            networkId.toEncodeObject(),
            action.toEncodeObject(),
            v,
            r.toEncodeObject(),
            s.toEncodeObject()
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    static fromJSON(data: any) {
        const { v, r, s, blockNumber, blockHash, parcelIndex } = data;
        if (blockNumber) {
            return new SignedParcel(Parcel.fromJSON(data), v, new U256(r), new U256(s), blockNumber, new H256(blockHash), parcelIndex);
        } else {
            return new SignedParcel(Parcel.fromJSON(data), v, new U256(r), new U256(s));
        }
    }

    toJSON() {
        const { blockNumber, blockHash, parcelIndex,
            unsigned: { nonce, fee, networkId, action }, v, r, s } = this;
        return {
            blockNumber,
            blockHash: blockHash === null ? null : blockHash.value,
            parcelIndex,
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            networkId: networkId.value.toNumber(),
            action: action.toJSON(),
            v,
            r: r.value.toString(),
            s: s.value.toString(),
        };
    }
}
