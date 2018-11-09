"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const buffer_1 = require("buffer");
const codechain_primitives_1 = require("codechain-primitives");
const H256_1 = require("./H256");
const AssetOutPoint_1 = require("./transaction/AssetOutPoint");
const AssetTransferInput_1 = require("./transaction/AssetTransferInput");
const AssetTransferOutput_1 = require("./transaction/AssetTransferOutput");
const AssetTransferTransaction_1 = require("./transaction/AssetTransferTransaction");
const U256_1 = require("./U256");
/**
 * Object created as an AssetMintTransaction or AssetTransferTransaction.
 */
class Asset {
    static fromJSON(data) {
        const { assetType, lockScriptHash, parameters, amount, transactionHash, transactionOutputIndex } = data;
        return new Asset({
            assetType: new H256_1.H256(assetType),
            lockScriptHash: new codechain_primitives_1.H160(lockScriptHash),
            parameters: parameters.map((p) => buffer_1.Buffer.from(p)),
            amount: U256_1.U256.ensure(amount),
            transactionHash: new H256_1.H256(transactionHash),
            transactionOutputIndex
        });
    }
    constructor(data) {
        const { transactionHash, transactionOutputIndex, assetType, amount, lockScriptHash, parameters } = data;
        this.assetType = data.assetType;
        this.lockScriptHash = data.lockScriptHash;
        this.parameters = data.parameters;
        this.amount = data.amount;
        this.outPoint = new AssetOutPoint_1.AssetOutPoint({
            transactionHash,
            index: transactionOutputIndex,
            assetType,
            amount,
            lockScriptHash,
            parameters
        });
    }
    toJSON() {
        const { assetType, lockScriptHash, parameters, amount, outPoint } = this;
        const { transactionHash, index } = outPoint;
        return {
            assetType: assetType.value,
            lockScriptHash: lockScriptHash.value,
            parameters,
            amount: amount.toEncodeObject(),
            transactionHash: transactionHash.value,
            transactionOutputIndex: index
        };
    }
    createTransferInput(options) {
        const { timelock = null } = options || {};
        return new AssetTransferInput_1.AssetTransferInput({
            prevOut: this.outPoint,
            timelock
        });
    }
    createTransferTransaction(params) {
        const { outPoint, assetType } = this;
        const { recipients = [], timelock = null, networkId } = params;
        return new AssetTransferTransaction_1.AssetTransferTransaction({
            burns: [],
            inputs: [
                new AssetTransferInput_1.AssetTransferInput({
                    prevOut: outPoint,
                    timelock,
                    lockScript: buffer_1.Buffer.from([]),
                    unlockScript: buffer_1.Buffer.from([])
                })
            ],
            outputs: recipients.map(recipient => new AssetTransferOutput_1.AssetTransferOutput({
                recipient: codechain_primitives_1.AssetTransferAddress.ensure(recipient.address),
                assetType,
                amount: recipient.amount
            })),
            networkId
        });
    }
}
exports.Asset = Asset;
