import * as _ from "lodash";

import { AssetTransferAddress } from "../../key/classes";

import { H256 } from "../H256";
import { blake256, blake256WithKey } from "../../utils";
import { AssetTransferInput } from "./AssetTransferInput";
import { AssetTransferOutput } from "./AssetTransferOutput";
import { Asset } from "../Asset";

const RLP = require("rlp");

export interface TransactionSigner {
    sign: (transaction: AssetTransferTransaction, index: number) => Promise<{
        unlockScript: Buffer,
        lockScript: Buffer
    }>;
}

export type AssetTransferTransactionData = {
    burns: AssetTransferInput[];
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
    networkId: number;
    nonce?: number;
};
/**
 * Spends the existing asset and creates a new asset. Ownership can be transferred during this process.
 *
 * - AssetTransfer consists of AssetTransferInput's list to spend and AssetTransferOutput's list to create.
 * - All inputs must be valid for the transaction to be valid.
 * - When each asset types' amount have been summed, the sum of inputs and the sum of outputs must be identical.
 * - It contains the network ID. This must be identical to the network ID to which the transaction is being sent to.
 * - If an identical transaction hash already exists, then the change fails. In this situation, a transaction can be created again by arbitrarily changing the nonce.
 */
export class AssetTransferTransaction {
    readonly burns: AssetTransferInput[];
    readonly inputs: AssetTransferInput[];
    readonly outputs: AssetTransferOutput[];
    readonly networkId: number;
    readonly nonce: number;
    readonly type = "assetTransfer";

    constructor({ burns, inputs, outputs, networkId, nonce }: AssetTransferTransactionData) {
        this.burns = burns;
        this.inputs = inputs;
        this.outputs = outputs;
        this.networkId = networkId;
        this.nonce = nonce || 0;
    }

    toEncodeObject() {
        return [
            4,
            this.networkId,
            this.burns.map(input => input.toEncodeObject()),
            this.inputs.map(input => input.toEncodeObject()),
            this.outputs.map(output => output.toEncodeObject()),
            this.nonce
        ];
    }

    rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    addBurns(...burns: (AssetTransferInput | Asset)[]): AssetTransferTransaction {
        burns.forEach(burn => {
            if (burn instanceof AssetTransferInput) {
                this.burns.push(burn);
            } else {
                this.burns.push(burn.createTransferInput());
            }
        });
        return this;
    }

    addInputs(...inputs: (AssetTransferInput | Asset)[]): AssetTransferTransaction {
        inputs.forEach(input => {
            if (input instanceof AssetTransferInput) {
                this.inputs.push(input);
            } else {
                this.inputs.push(input.createTransferInput());
            }
        });
        return this;
    }

    addOutputs(...outputs: (AssetTransferOutput | {
        amount: number,
        assetType: H256 | string
        recipient: AssetTransferAddress | string,
    })[]): AssetTransferTransaction {
        outputs.forEach(output => {
            if (output instanceof AssetTransferOutput) {
                this.outputs.push(output);
            } else {
                const { assetType, amount, recipient } = output;
                this.outputs.push(new AssetTransferOutput({
                    ...AssetTransferAddress.ensure(recipient).getLockScriptHashAndParameters(),
                    amount,
                    assetType: H256.ensure(assetType),
                }));
            }
        });
        return this;
    }

    getTransferredAsset(index: number): Asset {
        if (index >= this.outputs.length) {
            throw "invalid output index";
        }
        const output = this.outputs[index];
        const { assetType, lockScriptHash, parameters, amount } = output;
        return new Asset({
            assetType,
            lockScriptHash,
            parameters,
            amount,
            transactionHash: this.hash(),
            transactionOutputIndex: index
        });
    }

    getTransferredAssets(): Asset[] {
        return _.range(this.outputs.length).map(i => this.getTransferredAsset(i));
    }

    hashWithoutScript(): H256 {
        const { networkId, burns, inputs, outputs, nonce } = this;
        return new H256(blake256(new AssetTransferTransaction({
            burns: burns.map(input => input.withoutScript()),
            inputs: inputs.map(input => input.withoutScript()),
            outputs,
            networkId,
            nonce
        }).rlpBytes()));
    }

    async sign(index: number, params: { signer: TransactionSigner }): Promise<void> {
        const { signer } = params;
        if (index >= this.inputs.length) {
            throw "Invalid index";
        }
        const { lockScript, unlockScript } = await signer.sign(this, index);
        this.setLockScript(index, lockScript);
        this.setUnlockScript(index, unlockScript);
    }

    setLockScript(index: number, lockScript: Buffer): void {
        if (index < 0 || this.inputs.length <= index) {
            throw "Invalid index";
        }
        this.inputs[index].setLockScript(lockScript);
    }

    setUnlockScript(index: number, unlockScript: Buffer): void {
        if (index < 0 || this.inputs.length <= index) {
            throw "Invalid index";
        }
        this.inputs[index].setUnlockScript(unlockScript);
    }

    getAssetAddress(index: number): H256 {
        const iv = new Uint8Array([
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            (index >> 56) & 0xFF, (index >> 48) & 0xFF, (index >> 40) & 0xFF, (index >> 32) & 0xFF,
            (index >> 24) & 0xFF, (index >> 16) & 0xFF, (index >> 8) & 0xFF, index & 0xFF,
        ]);
        const shardId = this.outputs[index].shardId();

        const blake = blake256WithKey(this.hash().value, iv);
        const shardPrefix = convertU16toHex(shardId);
        const worldPrefix = "0000";
        const prefix = `4100${shardPrefix}${worldPrefix}`;
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }

    static fromJSON(obj: any) {
        const { data: { networkId, burns, inputs, outputs, nonce } } = obj;
        return new this({
            burns: burns.map((input: any) => AssetTransferInput.fromJSON(input)),
            inputs: inputs.map((input: any) => AssetTransferInput.fromJSON(input)),
            outputs: outputs.map((output: any) => AssetTransferOutput.fromJSON(output)),
            networkId,
            nonce
        });
    }

    toJSON() {
        const { networkId, burns, inputs, outputs, nonce } = this;
        return {
            type: this.type,
            data: {
                networkId,
                burns: burns.map(input => input.toJSON()),
                inputs: inputs.map(input => input.toJSON()),
                outputs: outputs.map(output => output.toJSON()),
                nonce,
            }
        };
    }
}

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xFF).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xFF).toString(16)).slice(-2);
    return hi + lo;
}
