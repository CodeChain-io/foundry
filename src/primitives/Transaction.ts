import { U256, H256, SignedTransaction } from "./index";
import { blake256, signEcdsa } from "../utils";
import { Action } from "./action/index";

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
        const { r, s, recoveryParam: v } = signEcdsa(this.hash().value, secret.value);
        return new SignedTransaction(this, v, new U256(r.toString()), new U256(s.toString()));
    }
}
