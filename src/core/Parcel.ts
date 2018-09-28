import { PlatformAddress } from "codechain-primitives";

import { blake256, signEcdsa } from "../utils";
import { Action, getActionFromJSON } from "./action/Action";
import { AssetTransactionGroup } from "./action/AssetTransactionGroup";
import { CreateShard } from "./action/CreateShard";
import { Payment } from "./action/Payment";
import { SetRegularKey } from "./action/SetReulgarKey";
import { H256 } from "./H256";
import { H512 } from "./H512";
import { SignedParcel } from "./SignedParcel";
import { Transaction } from "./transaction/Transaction";
import { NetworkId } from "./types";
import { U256 } from "./U256";

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
    /**
     * @deprecated
     */
    public static transactions(
        networkId: NetworkId,
        ...transactions: Transaction[]
    ): Parcel {
        const action = new AssetTransactionGroup({ transactions });
        return new Parcel(networkId, action);
    }

    /**
     * @deprecated
     */
    public static payment(
        networkId: NetworkId,
        receiver: PlatformAddress,
        value: U256
    ): Parcel {
        const action = new Payment(receiver, value);
        return new Parcel(networkId, action);
    }

    /**
     * @deprecated
     */
    public static setRegularKey(networkId: NetworkId, key: H512): Parcel {
        const action = new SetRegularKey(key);
        return new Parcel(networkId, action);
    }

    /**
     * @deprecated
     */
    public static createShard(networkId: NetworkId): Parcel {
        const action = new CreateShard();
        return new Parcel(networkId, action);
    }

    public static fromJSON(result: any) {
        const { nonce, fee, networkId, action } = result;
        const parcel = new Parcel(networkId, getActionFromJSON(action));
        parcel.setNonce(nonce);
        parcel.setFee(fee);
        return parcel;
    }
    public nonce: U256 | null;
    public fee: U256 | null;
    public readonly networkId: NetworkId;
    public readonly action: Action;

    constructor(networkId: NetworkId, action: Action) {
        this.nonce = null;
        this.fee = null;
        this.networkId = networkId;
        this.action = action;
    }

    public setNonce(nonce: U256 | string | number) {
        this.nonce = U256.ensure(nonce);
    }

    public setFee(fee: U256 | string | number) {
        this.fee = U256.ensure(fee);
    }

    public toEncodeObject(): any[] {
        const { nonce, fee, action, networkId } = this;
        if (!nonce || !fee) {
            throw Error("Nonce and fee in the parcel must be present");
        }
        return [
            nonce.toEncodeObject(),
            fee.toEncodeObject(),
            networkId,
            action.toEncodeObject()
        ];
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    public sign(params: {
        secret: H256 | string;
        nonce: U256 | string | number;
        fee: U256 | string | number;
    }): SignedParcel {
        const { secret, nonce, fee } = params;
        if (this.nonce !== null) {
            throw Error("The parcel nonce is already set");
        }
        this.nonce = U256.ensure(nonce);
        if (this.fee !== null) {
            throw Error("The parcel fee is already set");
        }
        this.fee = U256.ensure(fee);
        const { r, s, v } = signEcdsa(
            this.hash().value,
            H256.ensure(secret).value
        );
        const sig = SignedParcel.convertRsvToSignatureString({ r, s, v });
        return new SignedParcel(this, sig);
    }

    public toJSON() {
        const { nonce, fee, networkId, action } = this;
        if (!fee) {
            throw Error("Fee in the parcel must be present");
        }
        const result: any = {
            fee: fee.toEncodeObject(),
            networkId,
            action: action.toJSON()
        };
        if (nonce) {
            result.nonce = nonce.toEncodeObject();
        }
        return result;
    }
}
