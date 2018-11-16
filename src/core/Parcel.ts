import { PlatformAddress } from "codechain-primitives";

import { blake256, signEcdsa } from "../utils";
import { Action, getActionFromJSON } from "./action/Action";
import { AssetTransaction } from "./action/AssetTransaction";
import { Payment } from "./action/Payment";
import { WrapCCC } from "./action/WrapCCC";
import { Asset } from "./Asset";
import { H256 } from "./H256";
import { SignedParcel } from "./SignedParcel";
import { Transaction } from "./transaction/Transaction";
import { NetworkId } from "./types";
import { U64 } from "./U64";

const RLP = require("rlp");

/**
 * A unit that collects transaction and requests processing to the network. A parsel signer pays for CCC processing fees.
 *
 * - The fee must be at least 10. The higher the fee, the higher the priority for the parcel to be processed.
 * - It contains the network ID. This must be identical to the network ID to which the parcel is being sent to.
 * - Its seq must be identical to the seq of the account that will sign the parcel.
 * - It contains the transaction to process. After signing the Parcel's size must not exceed 1 MB.
 * - After signing with the sign() function, it can be sent to the network.
 */
export class Parcel {
    /**
     * @deprecated
     */
    public static transaction(
        networkId: NetworkId,
        transaction: Transaction
    ): Parcel {
        const action = new AssetTransaction({ transaction });
        return new Parcel(networkId, action);
    }

    /**
     * @deprecated
     */
    public static payment(
        networkId: NetworkId,
        receiver: PlatformAddress,
        value: U64
    ): Parcel {
        const action = new Payment(receiver, value);
        return new Parcel(networkId, action);
    }

    public static fromJSON(result: any) {
        const { seq, fee, networkId, action } = result;
        const parcel = new Parcel(networkId, getActionFromJSON(action));
        parcel.setSeq(seq);
        parcel.setFee(fee);
        return parcel;
    }
    public seq: U64 | null;
    public fee: U64 | null;
    public readonly networkId: NetworkId;
    public readonly action: Action;

    constructor(networkId: NetworkId, action: Action) {
        this.seq = null;
        this.fee = null;
        this.networkId = networkId;
        this.action = action;
    }

    public setSeq(seq: U64 | string | number) {
        this.seq = U64.ensure(seq);
    }

    public setFee(fee: U64 | string | number) {
        this.fee = U64.ensure(fee);
    }

    public toEncodeObject(): any[] {
        const { seq, fee, action, networkId } = this;
        if (!seq || !fee) {
            throw Error("Seq and fee in the parcel must be present");
        }
        return [
            seq.toEncodeObject(),
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

    public getAsset(): Asset {
        const { action } = this;
        if (!(action instanceof WrapCCC)) {
            throw Error("Getting asset is only available with WrapCCC action");
        }
        return action.getAsset(this.hash());
    }

    public sign(params: {
        secret: H256 | string;
        seq: U64 | string | number;
        fee: U64 | string | number;
    }): SignedParcel {
        const { secret, seq, fee } = params;
        if (this.seq !== null) {
            throw Error("The parcel seq is already set");
        }
        this.seq = U64.ensure(seq);
        if (this.fee !== null) {
            throw Error("The parcel fee is already set");
        }
        this.fee = U64.ensure(fee);
        const { r, s, v } = signEcdsa(
            this.hash().value,
            H256.ensure(secret).value
        );
        const sig = SignedParcel.convertRsvToSignatureString({ r, s, v });
        return new SignedParcel(this, sig);
    }

    public toJSON() {
        const { seq, fee, networkId, action } = this;
        if (!fee) {
            throw Error("Fee in the parcel must be present");
        }
        const result: any = {
            fee: fee.toEncodeObject(),
            networkId,
            action: action.toJSON()
        };
        if (seq) {
            result.seq = seq.toEncodeObject();
        }
        return result;
    }
}
