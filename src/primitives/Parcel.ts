import { BigNumber } from "bignumber.js";

import { U256 } from "./U256";
import { H160 } from "./H160";
import { H256 } from "./H256";
import { H512 } from "./H512";
import { SignedParcel } from "./SignedParcel";
import { blake256, signEcdsa } from "../utils";
import { Transaction, getTransactionFromJSON } from "./transaction";

const RLP = require("rlp");

interface Action {
    toEncodeObject(): Array<any>;
    toJSON(): Object;
}

class ChangeShardState implements Action {
    transactions: Transaction[];

    constructor(transactions: Transaction[]) {
        this.transactions = transactions;
    }

    toEncodeObject(): Array<any> {
        return [1, this.transactions.map(transaction => transaction.toEncodeObject())];
    }

    toJSON(): Object {
        return {
            action: "changeShardState",
            transactions: this.transactions.map(t => t.toJSON())
        };
    }
}

class Payment implements Action {
    receiver: H160;
    value: U256;

    constructor(receiver: H160, value: U256) {
        this.receiver = receiver;
        this.value = value;
    }

    toEncodeObject(): Array<any> {
        return [2, this.receiver.toEncodeObject(), this.value.toEncodeObject()];
    }

    toJSON(): Object {
        return {
            action: "payment",
            receiver: this.receiver.value,
            value: this.value.value.toString()
        };
    }
}

class SetRegularKey implements Action {
    key: H512;

    constructor(key: H512) {
        this.key = key;
    }

    toEncodeObject(): Array<any> {
        return [3, this.key.toEncodeObject()];
    }

    toJSON(): Object {
        return {
            action: "setRegularKey",
            key: this.key.value
        };
    }
}

function getActionFromJson(json: any): Action {
    const { action } = json;
    switch (action) {
        case "changeShardState":
            const { transactions } = json;
            return new ChangeShardState(transactions.map(getTransactionFromJSON));
        case "payment":
            const { receiver, value } = json;
            return new Payment(new H160(receiver), new U256(value));
        case "setRegularKey":
            const { key } = json;
            return new SetRegularKey(new H512(key));
        default:
            throw new Error(`Unexpected parcel action: ${action}`);
    }
}

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
    // FIXME: network id is 64-bit unsigned originally, so it must be changed when
    // it's serialized with leading zeros.
    networkId: U256;
    private action: Action;

    static transactions(nonce: U256, fee: U256, networkId: number, ...transactions: Transaction[]): Parcel {
        const action = new ChangeShardState(transactions);
        return new Parcel(nonce, fee, networkId, action);
    }

    static payment(nonce: U256, fee: U256, networkId: number, receiver: H160, value: U256): Parcel {
        const action = new Payment(receiver, value);
        return new Parcel(nonce, fee, networkId, action);
    }

    static setRegularKey(nonce: U256, fee: U256, networkId: number, key: H512): Parcel {
        const action = new SetRegularKey(key);
        return new Parcel(nonce, fee, networkId, action);
    }

    private constructor(nonce: U256, fee: U256, networkId: number, action: Action) {
        this.nonce = nonce;
        this.fee = fee;
        this.networkId = new U256(networkId);
        this.action = action;
    }

    toEncodeObject(): Array<any> {
        const { nonce, fee, action, networkId } = this;
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            networkId.toEncodeObject(),
            action.toEncodeObject()
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
        const { nonce, fee, networkId, action } = result;
        return new Parcel(new U256(nonce), new U256(fee), networkId, getActionFromJson(action));
    }

    toJSON() {
        const { nonce, fee, networkId, action } = this;
        return {
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            networkId: networkId.value.toNumber(),
            action: action.toJSON()
        };
    }
}
