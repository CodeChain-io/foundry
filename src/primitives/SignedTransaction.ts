import U256 from "./U256";
import H256 from "./H256";
import Transaction from "./Transaction";

const blake = require("blakejs");
const RLP = require("rlp");

class SignedTransaction {
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
        const context = blake.blake2bInit(32, null);
        blake.blake2bUpdate(context, this.rlpBytes());
        let hash: Buffer = blake.blake2bFinal(context);
        let hashStr = Array.from(hash).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
        return new H256(hashStr);
    }
}

export default SignedTransaction;
