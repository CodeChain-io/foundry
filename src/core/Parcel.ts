import { BigNumber } from "bignumber.js";

import { U256 } from "./U256";
import { H160 } from "./H160";
import { H256 } from "./H256";
import { H512 } from "./H512";
import { SignedParcel } from "./SignedParcel";
import { blake256, signEcdsa } from "../utils";
import { Transaction } from "./transaction/Transaction";
import { Action, getActionFromJSON } from "./action/Action";
import { ChangeShardState } from "./action/ChangeShardState";
import { Payment } from "./action/Payment";
import { SetRegularKey } from "./action/SetReulgarKey";
import { CreateShard } from "./action/CreateShard";

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
    // FIXME: network id is 64-bit unsigned originally, so it must be changed when
    // it's serialized with leading zeros.
    networkId: U256;
    action: Action;

    static transactions(nonce: U256, fee: U256, networkId: number, ...transactions: Transaction[]): Parcel {
        const action = new ChangeShardState({ transactions });
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

    static createShard(nonce: U256, fee: U256, networkId: number): Parcel {
        const action = new CreateShard();
        return new Parcel(nonce, fee, networkId, action);
    }

    constructor(nonce: U256, fee: U256, networkId: number, action: Action) {
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

    sign(secret: H256 | string): SignedParcel {
        const { r, s, v } = signEcdsa(this.hash().value, H256.ensure(secret).value);
        return new SignedParcel(this, v, new U256(new BigNumber(r, 16)), new U256(new BigNumber(s, 16)));
    }

    static fromJSON(result: any) {
        const { nonce, fee, networkId, action } = result;
        return new Parcel(new U256(nonce), new U256(fee), networkId, getActionFromJSON(action));
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
