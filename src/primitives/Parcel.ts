import { BigNumber } from "bignumber.js";

import { U256 } from "./U256";
import { H256 } from "./H256";
import { SignedParcel } from "./SignedParcel";
import { blake256, signEcdsa } from "../utils";
import { Transaction, getTransactionFromJSON } from "./transaction";

const RLP = require("rlp");
/**
 * A unit that collects transactions and requests processing to the network. A parsel signer pays for CCC processing fees.
 *
 * - The fee must be at least 10. The higher the fee, the higher the priority for the parcel to be processed.
 * - It contains the network ID. This must be identical to the network ID to which the parcel is being sent to.
 * - Its nonce must be identical to the nonce of the account that will sign the parcel.
 * - It contains the list of transactions to process. After signing the Parcel's size must not exceed 1 MB.
 * - After signing with the sign() function, it can be sent to the network.
 */
export class Parcel {
    nonce: U256;
    fee: U256;
    transactions: Transaction[];
    // FIXME: network id is 64-bit unsigned originally, so it must be changed when
    // it's serialized with leading zeros.
    networkId: U256;

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
        const { r, s, v } = signEcdsa(this.hash().value, secret.value);
        return new SignedParcel(this, v, new U256(new BigNumber(r, 16)), new U256(new BigNumber(s, 16)));
    }

    static fromJSON(result: any) {
        const { nonce, fee, transactions, networkId } = result;
        return new Parcel(new U256(nonce), new U256(fee), networkId, ...transactions.map(getTransactionFromJSON));
    }

    toJSON() {
        const { nonce, fee, networkId, transactions } = this;
        return {
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            networkId: networkId.value.toNumber(),
            transactions: transactions.map(t => t.toJSON()),
        };
    }
}
