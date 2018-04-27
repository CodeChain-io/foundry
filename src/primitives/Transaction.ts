import U256 from "./U256";
import H256 from "./H256";
import Action from "./Action";
import SignedTransaction from "./SignedTransaction";

const blake = require("blakejs");
const EC = require("elliptic").ec;
const RLP = require("rlp");

class Transaction {
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
        const context = blake.blake2bInit(32, null);
        blake.blake2bUpdate(context, this.rlpBytes());
        let hash: Buffer = blake.blake2bFinal(context);
        let hashStr = Array.from(hash).map(byte => byte < 0x10 ? `0${byte.toString(16)}` : byte.toString(16)).join("");
        return new H256(hashStr);
    }

    sign(secret: H256): SignedTransaction {
        const ec = new EC('secp256k1');
        const key = ec.keyFromPrivate(secret.value);
        const sig = key.sign(this.hash().value);
        return new SignedTransaction(this, sig.recoveryParam, new U256(sig.r.toString()), new U256(sig.s.toString()));
    }
}

export default Transaction;
