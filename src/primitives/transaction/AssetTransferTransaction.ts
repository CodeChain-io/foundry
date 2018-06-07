import { H256 } from "../index";
import { blake256WithKey, blake256 } from "../../utils";

const RLP = require("rlp");

export type AssetOutPointData = {
    transactionHash: H256;
    index: number;
    assetType: H256;
    amount: number;
};
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
}

export type AssetTransferOutputData = {
    lockScriptHash: H256;
    parameters: Buffer[];
    assetType: H256;
    amount: number;
};
export class AssetTransferOutput {
    private data: AssetTransferOutputData;
    private type = "assetTransfer";

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
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
};
export class AssetTransferTransaction {
    private inputs: AssetTransferInput[];
    private outputs: AssetTransferOutput[];
    private networkId: number;
    private nonce: number;
    constructor(networkId: number, { inputs, outputs }: AssetTransferTransactionData, nonce = 0) {
        this.inputs = inputs;
        this.outputs = outputs;
        this.networkId = networkId;
        this.nonce = nonce;
    }

    toEncodeObject() {
        return [
            4,
            this.networkId,
            this.inputs.map(input => input.toEncodeObject()),
            this.outputs.map(output => output.toEncodeObject()),
            this.nonce
        ];
    }

    rlpBytes() {
        return RLP.encode(this.toEncodeObject());
    }

    hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
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
        const { networkId, inputs, outputs, nonce } = data;
        return new this(networkId, {
            inputs: inputs.map((input: any) => AssetTransferInput.fromJSON(input)),
            outputs: outputs.map((output: any) => AssetTransferOutput.fromJSON(output))
        }, nonce);
    }

    toJSON() {
        const { networkId, inputs, outputs, nonce } = this;
        return {
            networkId,
            inputs: inputs.map(input => input.toJSON()),
            outputs: outputs.map(output => output.toJSON()),
            nonce,
        };
    }
}
