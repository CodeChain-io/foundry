import { U256, H256, Parcel } from "./index";
import { blake256 } from "../utils";

const RLP = require("rlp");

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
        this.v = v + 27;
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
        const { unsigned: { nonce, fee, transactions, networkId }, v, r, s } = this;
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            transactions.map(transaction => transaction.toEncodeObject()),
            networkId.toEncodeObject(),
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
        const { nonce, fee, transactions, networkId, v, r, s,
            blockNumber, blockHash, parcelIndex } = data;
        if (blockNumber) {
            return new SignedParcel(Parcel.fromJSON(data), v, new U256(r), new U256(s), blockNumber, new H256(blockHash), parcelIndex);
        } else {
            return new SignedParcel(Parcel.fromJSON(data), v, new U256(r), new U256(s));
        }
    }

    toJSON() {
        const { blockNumber, blockHash, parcelIndex,
            unsigned: { nonce, fee, transactions, networkId }, v, r, s } = this;
        return {
            blockNumber,
            blockHash: blockHash === null ? null : blockHash.value,
            parcelIndex,
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            transactions: transactions.map(t => t.toJSON()),
            networkId: networkId.value.toNumber(),
            v: v - 27,
            r: r.value.toString(),
            s: s.value.toString(),
        };
    }
}
