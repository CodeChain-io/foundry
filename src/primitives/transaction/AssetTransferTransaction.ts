import { H256 } from "../H256";
import { blake256WithKey, blake256 } from "../../utils";

const RLP = require("rlp");

export type AssetOutPointData = {
    transactionHash: H256;
    index: number;
    assetType: H256;
    amount: number;
};
/**
 * AssetOutPoint consists of transactionHash and index, asset type, and amount.
 *
 * - The transaction that it points to must be either AssetMint or AssetTransfer.
 * - Index is what decides which Asset to point to amongst the Asset list that transaction creates.
 * - The asset type and amount must be identical to the Asset that it points to.
 */
export class AssetOutPoint {
    private data: AssetOutPointData;

    constructor(data: AssetOutPointData) {
        this.data = data;
    }

    toEncodeObject() {
        const { transactionHash, index, assetType, amount } = this.data;
        return [transactionHash.toEncodeObject(), index, assetType.toEncodeObject(), amount];
    }

    static fromJSON(data: any) {
        const { transactionHash, index, assetType, amount } = data;
        return new this({
            transactionHash: new H256(transactionHash),
            index,
            assetType: new H256(assetType),
            amount,
        });
    }

    toJSON() {
        const { transactionHash, index, assetType, amount } = this.data;
        return {
            transactionHash: transactionHash.value,
            index,
            assetType: assetType.value,
            amount,
        };
    }
}

export type AssetTransferInputData = {
    prevOut: AssetOutPoint;
    lockScript: Buffer;
    unlockScript: Buffer;
};
/**
 * AssetTransferInput consists of the following:
 *
 * - AssetOutPoint, which points to the asset to be spent.
 * - lockScript and unlockScript, that prove ownership of the asset
 * - The hashed value(blake256) of lockScript must be identical to that of the pointed asset's lockScriptHash.
 * - The results of running the script must return successful in order for the Asset's Input to be valid.
 */
export class AssetTransferInput {
    private prevOut: AssetOutPoint;
    private lockScript: Buffer;
    private unlockScript: Buffer;

    constructor(data: AssetTransferInputData) {
        const { prevOut, lockScript, unlockScript } = data;
        this.prevOut = prevOut;
        this.lockScript = lockScript;
        this.unlockScript = unlockScript;
    }

    toEncodeObject() {
        const { prevOut, lockScript, unlockScript } = this;
        return [prevOut.toEncodeObject(), lockScript, unlockScript];
    }

    static fromJSON(data: any) {
        const { prevOut, lockScript, unlockScript } = data;
        return new this({
            prevOut: AssetOutPoint.fromJSON(prevOut),
            lockScript,
            unlockScript,
        });
    }

    toJSON() {
        const { prevOut, lockScript, unlockScript } = this;
        return {
            prevOut: prevOut.toJSON(),
            lockScript,
            unlockScript,
        };
    }

    withoutScript() {
        const { prevOut } = this;
        return new AssetTransferInput({
            prevOut,
            lockScript: Buffer.from([]),
            unlockScript: Buffer.from([]),
        });
    }

    setUnlockScript(unlockScript: Buffer) {
        this.unlockScript = unlockScript;
    }
}

export type AssetTransferOutputData = {
    lockScriptHash: H256;
    parameters: Buffer[];
    assetType: H256;
    amount: number;
};
/**
 * AssetTransferOutput consists of lockScriptHash and parameters, which mark ownership of the asset, and asset type and amount, which indicate the asset's type and quantity.
 */
export class AssetTransferOutput {
    private data: AssetTransferOutputData;

    constructor(data: AssetTransferOutputData) {
        this.data = data;
    }

    toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this.data;
        return [lockScriptHash.toEncodeObject(), parameters, assetType.toEncodeObject(), amount];
    }

    static fromJSON(data: any) {
        const { lockScriptHash, parameters, assetType, amount } = data;
        return new this({
            lockScriptHash: new H256(lockScriptHash),
            parameters,
            assetType: new H256(assetType),
            amount,
        });
    }

    toJSON() {
        const { lockScriptHash, parameters, assetType, amount } = this.data;
        return {
            lockScriptHash: lockScriptHash.value,
            parameters,
            assetType: assetType.value,
            amount,
        };
    }
}

export type AssetTransferTransactionData = {
    burns: AssetTransferInput[];
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
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
    private burns: AssetTransferInput[];
    private inputs: AssetTransferInput[];
    private outputs: AssetTransferOutput[];
    private networkId: number;
    private nonce: number;
    private type = "assetTransfer";

    constructor(networkId: number, { burns, inputs, outputs }: AssetTransferTransactionData, nonce = 0) {
        this.burns = burns;
        this.inputs = inputs;
        this.outputs = outputs;
        this.networkId = networkId;
        this.nonce = nonce;
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

    hashWithoutScript(): H256 {
        const { networkId, burns, inputs, outputs, nonce } = this;
        return new H256(blake256(new AssetTransferTransaction(networkId, {
            burns: burns.map(input => input.withoutScript()),
            inputs: inputs.map(input => input.withoutScript()),
            outputs,
        }, nonce).rlpBytes()));
    }

    setUnlockScript(index: number, unlockScript: Buffer) {
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
        const blake = blake256WithKey(this.hash().value, iv);
        const prefix = "4100000000000000";
        return new H256(blake.replace(new RegExp(`^.{${prefix.length}}`), prefix));
    }

    static fromJSON(data: any) {
        const { networkId, burns, inputs, outputs, nonce } = data["assetTransfer"];
        return new this(networkId, {
            burns: burns.map((input: any) => AssetTransferInput.fromJSON(input)),
            inputs: inputs.map((input: any) => AssetTransferInput.fromJSON(input)),
            outputs: outputs.map((output: any) => AssetTransferOutput.fromJSON(output))
        }, nonce);
    }

    toJSON() {
        const { networkId, burns, inputs, outputs, nonce } = this;
        return {
            [this.type]: {
                networkId,
                burns: burns.map(input => input.toJSON()),
                inputs: inputs.map(input => input.toJSON()),
                outputs: outputs.map(output => output.toJSON()),
                nonce,
            }
        };
    }
}
