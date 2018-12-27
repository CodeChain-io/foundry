import { blake256, signEcdsa } from "../utils";
import { H256 } from "./H256";
import { SignedParcel } from "./SignedParcel";
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
export abstract class Parcel {
    private _seq: number | null;
    private _fee: U64 | null;
    private readonly _networkId: NetworkId;

    protected constructor(networkId: NetworkId) {
        this._seq = null;
        this._fee = null;
        this._networkId = networkId;
    }

    public seq(): number | null {
        return this._seq;
    }

    public fee(): U64 | null {
        return this._fee;
    }

    public setSeq(seq: number) {
        this._seq = seq;
    }

    public setFee(fee: U64 | string | number) {
        this._fee = U64.ensure(fee);
    }

    public networkId(): NetworkId {
        return this._networkId;
    }

    public toEncodeObject(): any[] {
        const [seq, fee, networkId] = [this._seq, this._fee, this._networkId];
        if (seq == null || !fee) {
            throw Error("Seq and fee in the parcel must be present");
        }
        return [
            seq,
            fee.toEncodeObject(),
            networkId,
            this.actionToEncodeObject()
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
        seq: number;
        fee: U64 | string | number;
    }): SignedParcel {
        const { secret, seq, fee } = params;
        if (this._seq !== null) {
            throw Error("The parcel seq is already set");
        }
        this._seq = seq;
        if (this._fee !== null) {
            throw Error("The parcel fee is already set");
        }
        this._fee = U64.ensure(fee);
        const { r, s, v } = signEcdsa(
            this.hash().value,
            H256.ensure(secret).value
        );
        const sig = SignedParcel.convertRsvToSignatureString({ r, s, v });
        return new SignedParcel(this, sig);
    }

    public toJSON() {
        const seq = this._seq;
        const fee = this._fee;
        const networkId = this._networkId;
        if (!fee) {
            throw Error("Parcel must have the fee");
        }
        const action = this.actionToJSON();
        action.action = this.action();
        const result: any = {
            fee: fee.toJSON(),
            networkId,
            action
        };
        if (seq != null) {
            result.seq = seq;
        }
        return result;
    }

    protected abstract actionToJSON(): any;
    protected abstract actionToEncodeObject(): any[];
    protected abstract action(): string;
}
