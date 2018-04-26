import U256 from "./U256";
import Action from "./Action";

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

    rlpBytes(): Buffer {
        const { nonce, fee, action, networkId } = this;
        return RLP.encode([
            nonce.rlpBytes(), 
            fee.rlpBytes(), 
            action.rlpBytes(), 
            networkId.rlpBytes()
        ]);
    }
}

export default Transaction;
