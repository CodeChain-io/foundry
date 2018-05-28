import { H256 } from "../index";
import { blake256WithKey, blake256 } from "../../utils";

const RLP = require("rlp");

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
    private type = "assetTransfer";

    constructor(data: AssetTransferOutputData) {
        this.data = data;
    }

    toEncodeObject() {
        const { lockScriptHash, parameters, assetType, amount } = this.data;
        return [lockScriptHash.toEncodeObject(), parameters, assetType.toEncodeObject(), amount];
    }
}

export type AssetTransferTransactionData = {
    inputs: AssetTransferInputData[];
    outputs: AssetTransferOutputData[];
};
export class AssetTransferTransaction {
    private inputs: AssetTransferInput[];
    private outputs: AssetTransferOutput[];
    private networkId: number;
    private nonce: number;
    constructor(networkId: number, { inputs, outputs }: AssetTransferTransactionData, nonce = 0) {
        this.inputs = inputs.map(input => new AssetTransferInput(input));
        this.outputs = outputs.map(output => new AssetTransferOutput(output));
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

    fromJSON() {
        // FIXME:
        throw new Error("Not implemented");
    }

    toJSON() {
        // FIXME:
        throw new Error("Not implemented");
    }
}
