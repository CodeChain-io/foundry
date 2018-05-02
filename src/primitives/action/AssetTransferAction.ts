import { H256 } from "../index";

type AssetOutPointData = {
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

type AssetTransferInputData = {
    prevOut: AssetOutPoint;
    lockScript: Buffer;
    unlockScript: Buffer;
};
class AssetTransferInput {
    private data: AssetTransferInputData;

    constructor(data: AssetTransferInputData) {
        this.data = data;
    }

    toEncodeObject() {
        const { prevOut, lockScript, unlockScript } = this.data;
        return [prevOut.toEncodeObject(), lockScript, unlockScript];
    }
}

type AssetTransferOutputData = {
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

type AssetTransferActionData = {
    inputs: AssetTransferInput[];
    outputs: AssetTransferOutput[];
}
export class AssetTransferAction {
    private inputs: AssetTransferInput[];
    private outputs: AssetTransferOutput[];
    constructor({ inputs, outputs }: AssetTransferActionData) {
        this.inputs = inputs;
        this.outputs = outputs;
    }

    toEncodeObject() {
        return [
            4,
            this.inputs.map(input => input.toEncodeObject()),
            this.outputs.map(output => output.toEncodeObject())
        ]
    }
}