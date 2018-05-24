import { U256, H256, SignedParcel } from "./index";
import { blake256, signEcdsa } from "../utils";
import { Transaction, getTransactionFromJSON } from "./transaction/index";

const RLP = require("rlp");

export class Parcel {
    private nonce: U256;
    private fee: U256;
    private transactions: Transaction[];
    // FIXME: network id is 64-bit unsigned originally, so it must be changed when
    // it's serialized with leading zeros.
    private networkId: U256;

    constructor(nonce: U256, fee: U256, networkId: number, ...transactions: Transaction[]) {
        this.nonce = nonce;
        this.fee = fee;
        this.transactions = transactions;
        this.networkId = new U256(networkId);
    }

    toEncodeObject(): Array<any> {
        const { nonce, fee, transactions, networkId } = this;
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            transactions.map(transaction => transaction.toEncodeObject()),
            networkId.toEncodeObject()
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    sign(secret: H256): SignedParcel {
        const { r, s, recoveryParam: v } = signEcdsa(this.hash().value, secret.value);
        return new SignedParcel(this, v, new U256(r.toString()), new U256(s.toString()));
    }

    static fromJSON(result: any) {
        const { nonce, fee, transactions, networkId } = result;
        return new Parcel(new U256(nonce), new U256(fee), networkId, ...transactions.map(getTransactionFromJSON));
    }
}
