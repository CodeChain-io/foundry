import {
    blake128,
    H160,
    H256,
    PlatformAddress
} from "codechain-primitives/lib";
import * as _ from "lodash";

import {
    blake256,
    blake256WithKey,
    encodeSignatureTag,
    SignatureTag
} from "../../utils";

import { Asset } from "../Asset";
import { AssetScheme } from "../AssetScheme";
import { NetworkId } from "../types";
import { AssetMintOutput } from "./AssetMintOutput";
import { AssetTransferInput } from "./AssetTransferInput";

const RLP = require("rlp");

/**
 * Compose assets.
 */
export class AssetComposeTransaction {
    /**
     * Create an AssetComposeTransaction from an AssetComposeTransaction JSON object.
     * @param obj An AssetComposeTransaction JSON object.
     * @returns An AssetComposeTransaction.
     */
    public static fromJSON(obj: any) {
        const {
            data: {
                networkId,
                shardId,
                worldId,
                metadata,
                inputs,
                output,
                registrar,
                nonce
            }
        } = obj;
        return new this({
            networkId,
            shardId,
            worldId,
            metadata,
            registrar:
                registrar === null ? null : PlatformAddress.ensure(registrar),
            inputs: inputs.map((input: any) =>
                AssetTransferInput.fromJSON(input)
            ),
            output: AssetMintOutput.fromJSON(output),
            nonce
        });
    }

    public readonly networkId: NetworkId;
    public readonly shardId: number;
    public readonly worldId: number;
    public readonly metadata: string;
    public readonly registrar: PlatformAddress | null;
    public readonly inputs: AssetTransferInput[];
    public readonly output: AssetMintOutput;
    public readonly nonce: number;
    public readonly type = "assetCompose";

    /**
     * @param params.networkId A network ID of the transaction.
     * @param params.shardId A shard ID of the transaction.
     * @param params.worldId A world ID of the transaction.
     * @param params.metadata A metadata of the asset.
     * @param params.registrar A registrar of the asset.
     * @param params.inputs A list of inputs of the transaction.
     * @param params.output An output of the transaction.
     * @param params.nonce A nonce of the transaction.
     */
    constructor(params: {
        networkId: NetworkId;
        shardId: number;
        worldId: number;
        metadata: string;
        registrar: PlatformAddress | null;
        inputs: AssetTransferInput[];
        output: AssetMintOutput;
        nonce: number;
    }) {
        const {
            networkId,
            shardId,
            worldId,
            metadata,
            registrar,
            inputs,
            output,
            nonce
        } = params;
        this.networkId = networkId;
        this.shardId = shardId;
        this.worldId = worldId;
        this.metadata = metadata;
        this.registrar =
            registrar === null ? null : PlatformAddress.ensure(registrar);
        this.inputs = inputs;
        this.output = new AssetMintOutput(output);
        this.nonce = nonce;
    }

    /**
     * Convert to an AssetComposeTransaction JSON object.
     * @returns An AssetComposeTransaction JSON object.
     */
    public toJSON() {
        return {
            type: this.type,
            data: {
                networkId: this.networkId,
                shardId: this.shardId,
                worldId: this.worldId,
                metadata: this.metadata,
                registrar: this.registrar,
                output: this.output.toJSON(),
                inputs: this.inputs.map(input => input.toJSON()),
                nonce: this.nonce
            }
        };
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        return [
            6,
            this.networkId,
            this.shardId,
            this.worldId,
            this.metadata,
            this.registrar ? [this.registrar.toString()] : [],
            this.inputs.map(input => input.toEncodeObject()),
            this.output.lockScriptHash.toEncodeObject(),
            this.output.parameters.map(parameter => Buffer.from(parameter)),
            this.output.amount !== null ? [this.output.amount] : [],
            this.nonce
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Get the hash of an AssetComposeTransaction.
     * @returns A transaction hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Get a hash of the transaction that doesn't contain the scripts. The hash
     * is used as a message to create a signature for a transaction.
     * @returns A hash.
     */
    public hashWithoutScript(params?: {
        tag: SignatureTag;
        index: number;
    }): H256 {
        const {
            tag = { input: "all", output: "all" } as SignatureTag,
            index = null
        } = params || {};

        let inputs: AssetTransferInput[];
        if (tag.input === "all") {
            inputs = this.inputs.map(input => input.withoutScript());
        } else if (tag.input === "single") {
            if (typeof index !== "number") {
                throw Error(`Unexpected value of the index: ${index}`);
            }
            inputs = [this.inputs[index].withoutScript()];
        } else {
            throw Error(`Unexpected value of the tag input: ${tag.input}`);
        }
        let output: AssetMintOutput;
        if (tag.output === "all") {
            output = this.output;
        } else if (Array.isArray(tag.output) && tag.output.length === 0) {
            // NOTE: An empty array is allowed only
            output = new AssetMintOutput({
                lockScriptHash: new H160(
                    "0000000000000000000000000000000000000000"
                ),
                parameters: [],
                amount: null
            });
        } else {
            throw Error(`Unexpected value of the tag output: ${tag.output}`);
        }
        const {
            networkId,
            shardId,
            worldId,
            metadata,
            registrar,
            nonce
        } = this;
        return new H256(
            blake256WithKey(
                new AssetComposeTransaction({
                    networkId,
                    shardId,
                    worldId,
                    metadata,
                    registrar,
                    inputs,
                    output,
                    nonce
                }).rlpBytes(),
                Buffer.from(blake128(encodeSignatureTag(tag)), "hex")
            )
        );
    }

    /**
     * Add an AssetTransferInput to spend.
     * @param inputs An array of either an AssetTransferInput or an Asset.
     * @returns The modified AssetComposeTransaction.
     */
    public addInputs(
        inputs: AssetTransferInput | Asset | Array<AssetTransferInput | Asset>,
        ...rest: Array<AssetTransferInput | Asset>
    ): AssetComposeTransaction {
        if (!Array.isArray(inputs)) {
            inputs = [inputs, ...rest];
        }
        inputs.forEach((input, index) => {
            if (input instanceof AssetTransferInput) {
                this.inputs.push(input);
            } else if (input instanceof Asset) {
                this.inputs.push(input.createTransferInput());
            } else {
                throw Error(
                    `Expected an array of either AssetTransferInput or Asset but found ${input} at ${index}`
                );
            }
        });
        return this;
    }

    /**
     * Get the output of this transaction.
     * @returns An Asset.
     */
    public getComposedAsset(): Asset {
        const { lockScriptHash, parameters, amount } = this.output;
        if (amount === null) {
            throw Error("not implemented");
        }
        return new Asset({
            assetType: this.getAssetSchemeAddress(),
            lockScriptHash,
            parameters,
            amount,
            transactionHash: this.hash(),
            transactionOutputIndex: 0
        });
    }

    /**
     * Get the asset scheme of this transaction.
     * @return An AssetScheme.
     */
    public getAssetScheme(): AssetScheme {
        const {
            networkId,
            shardId,
            worldId,
            metadata,
            inputs,
            output: { amount },
            registrar
        } = this;
        // FIXME: need U64 to be implemented or use U256
        if (amount === null) {
            throw Error("not implemented");
        }
        return new AssetScheme({
            networkId,
            shardId,
            worldId,
            metadata,
            amount,
            registrar,
            pool: _.toPairs(
                // NOTE: Get the sum of each asset type
                inputs.reduce((acc: { [assetType: string]: number }, input) => {
                    const { assetType, amount: assetAmount } = input.prevOut;
                    // FIXME: Check integer overflow
                    acc[assetType.value] += assetAmount;
                    return acc;
                }, {})
            ).map(([assetType, assetAmount]) => ({
                assetType: H256.ensure(assetType),
                amount: assetAmount as number
            }))
        });
    }

    /**
     * Get the address of the asset scheme. An asset scheme address equals to an
     * asset type value.
     * @returns An asset scheme address which is H256.
     */
    public getAssetSchemeAddress(): H256 {
        const { shardId, worldId } = this;
        const blake = blake256WithKey(
            this.hash().value,
            new Uint8Array([
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff,
                0xff
            ])
        );
        const shardPrefix = convertU16toHex(shardId);
        const worldPrefix = convertU16toHex(worldId);
        const prefix = `5300${shardPrefix}${worldPrefix}`;
        return new H256(
            blake.replace(new RegExp(`^.{${prefix.length}}`), prefix)
        );
    }

    /**
     * Get the asset address of the output.
     * @returns An asset address which is H256.
     */
    public getAssetAddress(): H256 {
        const { shardId, worldId } = this;
        const blake = blake256WithKey(
            this.hash().value,
            new Uint8Array([
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00
            ])
        );
        const shardPrefix = convertU16toHex(shardId);
        const worldPrefix = convertU16toHex(worldId);
        const prefix = `4100${shardPrefix}${worldPrefix}`;
        return new H256(
            blake.replace(new RegExp(`^.{${prefix.length}}`), prefix)
        );
    }
}

function convertU16toHex(id: number) {
    const hi: string = ("0" + ((id >> 8) & 0xff).toString(16)).slice(-2);
    const lo: string = ("0" + (id & 0xff).toString(16)).slice(-2);
    return hi + lo;
}
