import { PlatformAddress } from "../key/PlatformAddress";
import { U256 } from "./U256";
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

type NetworkId = string;

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
    nonce: U256 | null;
    fee: U256 | null;
    readonly networkId: NetworkId;
    readonly action: Action;

    static transactions(networkId: NetworkId, ...transactions: Transaction[]): Parcel {
        const action = new ChangeShardState({ transactions });
        return new Parcel(networkId, action);
    }

    static payment(networkId: NetworkId, receiver: PlatformAddress, value: U256): Parcel {
        const action = new Payment(receiver, value);
        return new Parcel(networkId, action);
    }

    static setRegularKey(networkId: NetworkId, key: H512): Parcel {
        const action = new SetRegularKey(key);
        return new Parcel(networkId, action);
    }

    static createShard(networkId: NetworkId): Parcel {
        const action = new CreateShard();
        return new Parcel(networkId, action);
    }

    constructor(networkId: NetworkId, action: Action) {
        this.nonce = null;
        this.fee = null;
        this.networkId = networkId;
        this.action = action;
    }

    setNonce(nonce: U256 | string | number) {
        this.nonce = U256.ensure(nonce);
    }

    setFee(fee: U256 | string | number) {
        this.fee = U256.ensure(fee);
    }

    toEncodeObject(): Array<any> {
        const { nonce, fee, action, networkId } = this;
        if (!nonce || !fee) {
            throw "Nonce and fee in the parcel must be present";
        }
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            networkId,
            action.toEncodeObject()
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    sign(params: {
        secret: H256 | string,
        nonce: U256 | string | number,
        fee: U256 | string | number
    }): SignedParcel {
        const { secret, nonce, fee } = params;
        if (this.nonce !== null) {
            throw "The parcel nonce is already set";
        }
        this.nonce = U256.ensure(nonce);
        if (this.fee !== null) {
            throw "The parcel fee is already set";
        }
        this.fee = U256.ensure(fee);
        const { r, s, v } = signEcdsa(this.hash().value, H256.ensure(secret).value);
        const sig = SignedParcel.convertRsvToSignatureString({ r, s, v });
        return new SignedParcel(this, sig);
    }

    static fromJSON(result: any) {
        const { nonce, fee, networkId, action } = result;
        const parcel = new Parcel(networkId, getActionFromJSON(action));
        parcel.setNonce(nonce);
        parcel.setFee(fee);
        return parcel;
    }

    toJSON() {
        const { nonce, fee, networkId, action } = this;
        if (!nonce || !fee) {
            throw "Nonce and fee in the parcel must be present";
        }
        return {
            nonce: nonce.value.toString(),
            fee: fee.value.toString(),
            networkId,
            action: action.toJSON()
        };
    }
}
