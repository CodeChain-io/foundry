import { H256 } from "../index";

export type AssetOutPointData = {
    transactionHash: H256;
    index: number;
    assetType: H256;
    amount: number;
};
class AssetOutPoint {
    private data: AssetOutPointData;

    constructor(data: AssetOutPointData) {
        this.data = data;
    }

    toEncodeObject() {
        const { transactionHash, index, assetType, amount } = this.data;
        return [transactionHash.toEncodeObject(), index, assetType.toEncodeObject(), amount];
    }
}

export type AssetTransferInputData = {
    prevOut: AssetOutPointData;
    lockScript: Buffer;
    unlockScript: Buffer;
};
class AssetTransferInput {
    private prevOut: AssetOutPoint;
    private lockScript: Buffer;
    private unlockScript: Buffer;

    constructor(data: AssetTransferInputData) {
        const { prevOut, lockScript, unlockScript } = data;
        this.prevOut = new AssetOutPoint(prevOut);
        this.lockScript = lockScript;
        this.unlockScript = unlockScript;
    }

    toEncodeObject() {
        const { prevOut, lockScript, unlockScript } = this;
        return [prevOut.toEncodeObject(), lockScript, unlockScript];
    }
}

export type AssetTransferOutputData = {
    lockScriptHash: H256;
    parameters: Buffer[];
    assetType: H256;
    amount: number;
};
class AssetTransferOutput {
    private data: AssetTransferOutputData;

    constructor(data: AssetTransferOutputData) {
        this.data = data;
    }

    toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this.data;
        return [lockScriptHash.toEncodeObject(), parameters, assetType.toEncodeObject(), amount];
    }
}

export type AssetTransferActionData = {
    inputs: AssetTransferInputData[];
    outputs: AssetTransferOutputData[];
};
export class AssetTransferAction {
    private inputs: AssetTransferInput[];
    private outputs: AssetTransferOutput[];
    constructor({ inputs, outputs }: AssetTransferActionData) {
        this.inputs = inputs.map(input => new AssetTransferInput(input));
        this.outputs = outputs.map(output => new AssetTransferOutput(output));
    }

    toEncodeObject() {
        return [
            4,
            this.inputs.map(input => input.toEncodeObject()),
            this.outputs.map(output => output.toEncodeObject())
        ];
    }
}