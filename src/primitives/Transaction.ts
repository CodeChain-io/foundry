import { U256, H256, Action, SignedTransaction } from "./index";
import { blake256 } from "../utils";

const EC = require("elliptic").ec;
const RLP = require("rlp");

export class Transaction {
    private nonce: U256;
    private fee: U256;
    private action: Action;
    // FIXME: network id is 64-bit unsigned originally, so it must be changed when
    // it's serialized with leading zeros.
    private networkId: U256;

    constructor(nonce: U256, fee: U256, action: Action, networkId: number) {
        this.nonce = nonce;
        this.fee = fee;
        this.action = action;
        this.networkId = new U256(networkId);
    }

    toEncodeObject(): Array<any> {
        const { nonce, fee, action, networkId } = this;
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            action.toEncodeObject(),
            networkId.toEncodeObject()
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    sign(secret: H256): SignedTransaction {
        const ec = new EC("secp256k1");
        const key = ec.keyFromPrivate(secret.value);
        const sig = key.sign(this.hash().value, { "canonical": true });
        return new SignedTransaction(this, sig.recoveryParam, new U256(sig.r.toString()), new U256(sig.s.toString()));
    }
}
