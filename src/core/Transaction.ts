import { H256, H256Value, U64, U64Value } from "codechain-primitives";

import { blake256, signEcdsa } from "../utils";
import { SignedTransaction } from "./SignedTransaction";
import { ChangeAssetSchemeActionJSON } from "./transaction/ChangeAssetScheme";
import { ComposeAssetActionJSON } from "./transaction/ComposeAsset";
import { CreateShardActionJSON } from "./transaction/CreateShard";
import { CustomActionJSON } from "./transaction/Custom";
import { DecomposeAssetActionJSON } from "./transaction/DecomposeAsset";
import { IncreaseAssetSupplyActionJSON } from "./transaction/IncreaseAssetSupply";
import { MintAssetActionJSON } from "./transaction/MintAsset";
import { PayActionJSON } from "./transaction/Pay";
import { RemoveActionJSON } from "./transaction/Remove";
import { SetRegularKeyActionJSON } from "./transaction/SetRegularKey";
import { SetShardOwnersActionJSON } from "./transaction/SetShardOwners";
import { SetShardUsersActionJSON } from "./transaction/SetShardUsers";
import { StoreActionJSON } from "./transaction/Store";
import { TransferAssetActionJSON } from "./transaction/TransferAsset";
import { UnwrapCCCActionJSON } from "./transaction/UnwrapCCC";
import { WrapCCCActionJSON } from "./transaction/WrapCCC";
import { NetworkId } from "./types";

const RLP = require("rlp");

export interface AssetTransaction {
    tracker(): H256;
}
type ActionJSON =
    | PayActionJSON
    | SetRegularKeyActionJSON
    | SetShardOwnersActionJSON
    | SetShardUsersActionJSON
    | IncreaseAssetSupplyActionJSON
    | CreateShardActionJSON
    | MintAssetActionJSON
    | TransferAssetActionJSON
    | ComposeAssetActionJSON
    | DecomposeAssetActionJSON
    | ChangeAssetSchemeActionJSON
    | StoreActionJSON
    | RemoveActionJSON
    | CustomActionJSON
    | WrapCCCActionJSON
    | UnwrapCCCActionJSON;

export interface TransactionJSON {
    action: ActionJSON & { type: string };
    networkId: string;
    seq: number | null;
    fee: string | null;
}
/**
 * A unit that collects transaction and requests processing to the network. A parsel signer pays for CCC processing fees.
 *
 * - The fee must be at least 10. The higher the fee, the higher the priority for the tx to be processed.
 * - It contains the network ID. This must be identical to the network ID to which the tx is being sent to.
 * - Its seq must be identical to the seq of the account that will sign the tx.
 * - It contains the transaction to process. After signing the Transaction's size must not exceed 1 MB.
 * - After signing with the sign() function, it can be sent to the network.
 */
export abstract class Transaction {
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

    public setFee(fee: U64Value) {
        this._fee = U64.ensure(fee);
    }

    public networkId(): NetworkId {
        return this._networkId;
    }

    public toEncodeObject(): any[] {
        const [seq, fee, networkId] = [this._seq, this._fee, this._networkId];
        if (seq == null || !fee) {
            throw Error("Seq and fee in the tx must be present");
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

    public unsignedHash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    public sign(params: {
        secret: H256Value;
        seq: number;
        fee: U64Value;
    }): SignedTransaction {
        const { secret, seq, fee } = params;
        if (this._seq != null) {
            throw Error("The tx seq is already set");
        }
        this._seq = seq;
        if (this._fee != null) {
            throw Error("The tx fee is already set");
        }
        this._fee = U64.ensure(fee);
        return new SignedTransaction(
            this,
            signEcdsa(this.unsignedHash().value, H256.ensure(secret).value)
        );
    }

    public toJSON(): TransactionJSON {
        const seq = this._seq;
        const fee = this._fee;
        const networkId = this._networkId;
        const action = this.actionToJSON();
        const result: TransactionJSON = {
            networkId,
            action: {
                ...action,
                type: this.type()
            },
            seq: seq != null ? seq : null,
            fee: fee != null ? fee.toJSON() : null
        };
        return result;
    }

    public abstract type(): string;

    protected abstract actionToJSON(): ActionJSON;
    protected abstract actionToEncodeObject(): any[];
}
