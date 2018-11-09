"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const codechain_primitives_1 = require("codechain-primitives");
const AssetMintOutput_1 = require("./transaction/AssetMintOutput");
const AssetMintTransaction_1 = require("./transaction/AssetMintTransaction");
const U256_1 = require("./U256");
/**
 * Object that contains information about the Asset when performing AssetMintTransaction.
 */
class AssetScheme {
    static fromJSON(data) {
        const { metadata, amount, registrar, pool } = data;
        return new AssetScheme({
            metadata,
            amount: U256_1.U256.ensure(amount),
            registrar: registrar === null ? null : codechain_primitives_1.PlatformAddress.ensure(registrar),
            pool: pool.map(({ assetType, amount: assetAmount }) => ({
                assetType: codechain_primitives_1.H256.ensure(assetType),
                amount: U256_1.U256.ensure(assetAmount)
            }))
        });
    }
    constructor(data) {
        this.networkId = data.networkId;
        this.shardId = data.shardId;
        this.metadata = data.metadata;
        this.registrar = data.registrar;
        this.amount = data.amount;
        this.pool = data.pool;
    }
    toJSON() {
        const { metadata, amount, registrar, pool } = this;
        return {
            metadata,
            amount: amount.toEncodeObject(),
            registrar: registrar === null ? null : registrar.toString(),
            pool: pool.map(a => ({
                assetType: a.assetType.value,
                amount: a.amount.toEncodeObject()
            }))
        };
    }
    createMintTransaction(params) {
        const { recipient } = params;
        const { networkId, shardId, metadata, amount, registrar } = this;
        if (networkId === undefined) {
            throw Error(`networkId is undefined`);
        }
        if (shardId === undefined) {
            throw Error(`shardId is undefined`);
        }
        return new AssetMintTransaction_1.AssetMintTransaction({
            networkId,
            shardId,
            metadata,
            output: new AssetMintOutput_1.AssetMintOutput({
                amount,
                recipient: codechain_primitives_1.AssetTransferAddress.ensure(recipient)
            }),
            registrar
        });
    }
}
exports.AssetScheme = AssetScheme;
