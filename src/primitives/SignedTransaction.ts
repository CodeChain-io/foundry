import { U256, H256, Transaction } from "./index"
import { blake256 } from "../utils";

const RLP = require("rlp");

export class SignedTransaction {
    private unsigned: Transaction;
    private v: number;
    private r: U256;
    private s: U256;

    constructor(unsigned: Transaction, v: number, r: U256, s: U256) {
        this.unsigned = unsigned;
        this.v = v + 27;
        this.r = r;
        this.s = s;
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
            action.toEncodeObject(),
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
}
