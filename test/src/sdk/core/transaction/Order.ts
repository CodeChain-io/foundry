import { AssetAddress, H160, H256, U64, U64Value } from "codechain-primitives";

import { P2PKH } from "../../key/P2PKH";
import { P2PKHBurn } from "../../key/P2PKHBurn";

import { blake256 } from "../../utils";

import { AssetOutPoint, AssetOutPointJSON } from "./AssetOutPoint";

const RLP = require("rlp");

export interface OrderJSON {
    assetTypeFrom: string;
    assetTypeTo: string;
    assetTypeFee: string;
    shardIdFrom: number;
    shardIdTo: number;
    shardIdFee: number;
    assetQuantityFrom: string;
    assetQuantityTo: string;
    assetQuantityFee: string;
    originOutputs: AssetOutPointJSON[];
    expiration: string;
    lockScriptHashFrom: string;
    parametersFrom: string[];
    lockScriptHashFee: string;
    parametersFee: string[];
}

export interface OrderDataBasic {
    assetTypeFrom: H160;
    assetTypeTo: H160;
    assetTypeFee?: H160;
    shardIdFrom: number;
    shardIdTo: number;
    shardIdFee?: number;
    assetQuantityFrom: U64;
    assetQuantityTo: U64;
    assetQuantityFee?: U64;
    originOutputs: AssetOutPoint[];
    expiration: U64;
}

export interface OrderAddressData {
    assetTypeFrom: H160;
    assetTypeTo: H160;
    assetTypeFee?: H160;
    shardIdFrom: number;
    shardIdTo: number;
    shardIdFee?: number;
    assetQuantityFrom: U64;
    assetQuantityTo: U64;
    assetQuantityFee?: U64;
    originOutputs: AssetOutPoint[];
    expiration: U64;
    recipientFrom: AssetAddress;
    recipientFee: AssetAddress;
}

export class Order {
    /**
     * Create an Order from an OrderJSON object.
     * @param data An OrderJSON object.
     * @returns An Order.
     */
    public static fromJSON(data: OrderJSON) {
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee,
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom,
            assetQuantityTo,
            assetQuantityFee,
            originOutputs,
            expiration,
            lockScriptHashFrom,
            parametersFrom,
            lockScriptHashFee,
            parametersFee
        } = data;
        return new this({
            assetTypeFrom: new H160(assetTypeFrom),
            assetTypeTo: new H160(assetTypeTo),
            assetTypeFee: new H160(assetTypeFee),
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom: U64.ensure(assetQuantityFrom),
            assetQuantityTo: U64.ensure(assetQuantityTo),
            assetQuantityFee: U64.ensure(assetQuantityFee),
            originOutputs: originOutputs.map((point: AssetOutPointJSON) =>
                AssetOutPoint.fromJSON(point)
            ),
            expiration: U64.ensure(expiration),
            lockScriptHashFrom: new H160(lockScriptHashFrom),
            parametersFrom: parametersFrom.map((p: string) =>
                Buffer.from(p, "hex")
            ),
            lockScriptHashFee: new H160(lockScriptHashFee),
            parametersFee: parametersFee.map((p: string) =>
                Buffer.from(p, "hex")
            )
        });
    }

    public readonly assetTypeFrom: H160;
    public readonly assetTypeTo: H160;
    public readonly assetTypeFee: H160;
    public readonly shardIdFrom: number;
    public readonly shardIdTo: number;
    public readonly shardIdFee: number;
    public readonly assetQuantityFrom: U64;
    public readonly assetQuantityTo: U64;
    public readonly assetQuantityFee: U64;
    public readonly originOutputs: AssetOutPoint[];
    public readonly expiration: U64;
    public readonly lockScriptHashFrom: H160;
    public readonly parametersFrom: Buffer[];
    public readonly lockScriptHashFee: H160;
    public readonly parametersFee: Buffer[];

    /**
     * @param data.assetTypeFrom The asset type of the asset to give.
     * @param data.assetTypeTo The asset type of the asset to get.
     * @param data.assetTypeFee The asset type of the asset for fee.
     * @param data.assetQuantityFrom The quantity of the asset to give.
     * @param data.assetQuantityTo The quantity of the asset to get.
     * @param data.assetQuantityFee The quantity of the asset for fee.
     * @param data.originOutputs The previous outputs to be consumed by the order.
     * @param data.expiration The expiration time of the order, by seconds.
     * @param data.lockScriptHash The lock script hash of the asset.
     * @param data.parameters The parameters of the asset.
     */
    constructor(
        data: OrderDataBasic &
            (
                | {
                      lockScriptHashFrom: H160;
                      parametersFrom: Buffer[];
                  }
                | {
                      recipientFrom: AssetAddress;
                  }
            ) &
            (
                | {
                      lockScriptHashFee: H160;
                      parametersFee: Buffer[];
                  }
                | {
                      recipientFee: AssetAddress;
                  }
            )
    ) {
        if ("recipientFrom" in data) {
            const { lockScriptHash, parameters } = decomposeRecipient(
                data.recipientFrom
            );
            this.lockScriptHashFrom = lockScriptHash;
            this.parametersFrom = parameters;
        } else {
            const { lockScriptHashFrom, parametersFrom } = data;
            this.lockScriptHashFrom = lockScriptHashFrom;
            this.parametersFrom = parametersFrom;
        }
        if ("recipientFee" in data) {
            const { lockScriptHash, parameters } = decomposeRecipient(
                data.recipientFee
            );
            this.lockScriptHashFee = lockScriptHash;
            this.parametersFee = parameters;
        } else {
            const { lockScriptHashFee, parametersFee } = data;
            this.lockScriptHashFee = lockScriptHashFee;
            this.parametersFee = parametersFee;
        }
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee = H160.zero(),
            shardIdFrom,
            shardIdTo,
            shardIdFee = 0,
            assetQuantityFrom,
            assetQuantityTo,
            assetQuantityFee = U64.ensure(0),
            originOutputs,
            expiration
        } = data;
        // Called too many times, so moving to variables
        const assetQuantityFromIsZero = assetQuantityFrom.value.isZero();
        const assetQuantityToIsZero = assetQuantityTo.value.isZero();
        const assetQuantityFeeIsZero = assetQuantityFee.value.isZero();

        if (assetTypeFrom.isEqualTo(assetTypeTo) && shardIdFrom === shardIdTo) {
            throw Error(
                `assetTypeFrom and assetTypeTo is same: ${assetTypeFrom}(shard ${shardIdFrom})`
            );
        } else if (!assetQuantityFeeIsZero) {
            if (
                assetTypeFrom.isEqualTo(assetTypeFee) &&
                shardIdFrom === shardIdFee
            ) {
                throw Error(
                    `assetTypeFrom and assetTypeFee is same: ${assetTypeFrom}(shard ${shardIdFrom})`
                );
            }
            if (
                assetTypeTo.isEqualTo(assetTypeFee) &&
                shardIdTo === shardIdFee
            ) {
                throw Error(
                    `assetTypeTo and assetTypeFee is same: ${assetTypeTo}(shard ${shardIdTo})`
                );
            }
        }

        if (
            (assetQuantityFromIsZero && !assetQuantityToIsZero) ||
            (!assetQuantityFromIsZero && assetQuantityToIsZero) ||
            (assetQuantityFromIsZero && assetQuantityFeeIsZero) ||
            (!assetQuantityFromIsZero &&
                !assetQuantityFee.value.mod(assetQuantityFrom.value).isZero())
        ) {
            throw Error(
                `The given quantity ratio is invalid: ${assetQuantityFrom}:${assetQuantityTo}:${assetQuantityFee}`
            );
        }
        if (originOutputs.length === 0) {
            throw Error(`originOutputs is empty`);
        }
        this.assetTypeFrom = assetTypeFrom;
        this.assetTypeTo = assetTypeTo;
        this.assetTypeFee = assetTypeFee;
        this.shardIdFrom = shardIdFrom;
        this.shardIdTo = shardIdTo;
        this.shardIdFee = shardIdFee;
        this.assetQuantityFrom = assetQuantityFrom;
        this.assetQuantityTo = assetQuantityTo;
        this.assetQuantityFee = assetQuantityFee;
        this.originOutputs = originOutputs;
        this.expiration = expiration;
    }

    /**
     * Convert to an object for RLP encoding.
     */
    public toEncodeObject() {
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee,
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom,
            assetQuantityTo,
            assetQuantityFee,
            originOutputs,
            expiration,
            lockScriptHashFrom,
            parametersFrom,
            lockScriptHashFee,
            parametersFee
        } = this;
        return [
            assetTypeFrom.toEncodeObject(),
            assetTypeTo.toEncodeObject(),
            assetTypeFee.toEncodeObject(),
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom.toEncodeObject(),
            assetQuantityTo.toEncodeObject(),
            assetQuantityFee.toEncodeObject(),
            originOutputs.map(output => output.toEncodeObject()),
            expiration.toEncodeObject(),
            lockScriptHashFrom.toEncodeObject(),
            parametersFrom.map(p => Buffer.from(p)),
            lockScriptHashFee.toEncodeObject(),
            parametersFee.map(p => Buffer.from(p))
        ];
    }

    /**
     * Convert to RLP bytes.
     */
    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    /**
     * Convert to an OrderJSON object.
     * @returns An OrderJSON object.
     */
    public toJSON(): OrderJSON {
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee,
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom,
            assetQuantityTo,
            assetQuantityFee,
            originOutputs,
            expiration,
            lockScriptHashFrom,
            parametersFrom,
            lockScriptHashFee,
            parametersFee
        } = this;
        return {
            assetTypeFrom: assetTypeFrom.toJSON(),
            assetTypeTo: assetTypeTo.toJSON(),
            assetTypeFee: assetTypeFee.toJSON(),
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom: assetQuantityFrom.toJSON(),
            assetQuantityTo: assetQuantityTo.toJSON(),
            assetQuantityFee: assetQuantityFee.toJSON(),
            originOutputs: originOutputs.map(output => output.toJSON()),
            expiration: expiration.toString(),
            lockScriptHashFrom: lockScriptHashFrom.toJSON(),
            parametersFrom: parametersFrom.map((p: Buffer) =>
                p.toString("hex")
            ),
            lockScriptHashFee: lockScriptHashFee.toJSON(),
            parametersFee: parametersFee.map((p: Buffer) => p.toString("hex"))
        };
    }

    /**
     * Get the hash of an order.
     * @returns An order hash.
     */
    public hash(): H256 {
        return new H256(blake256(this.rlpBytes()));
    }

    /**
     * Return the consumed order
     * @param params.quantity the consumed quantity of the asset to give
     */
    public consume(quantity: U64Value): Order {
        const {
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee,
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom,
            assetQuantityTo,
            assetQuantityFee,
            originOutputs,
            expiration,
            lockScriptHashFrom,
            parametersFrom,
            lockScriptHashFee,
            parametersFee
        } = this;
        const quantityFrom = U64.ensure(quantity);
        if (quantityFrom.gt(assetQuantityFrom)) {
            throw Error(
                `The given quantity is too big: ${quantityFrom} > ${assetQuantityFrom}`
            );
        }
        const remainQuantityFrom = this.assetQuantityFrom.value.minus(
            quantityFrom.value
        );
        if (
            !remainQuantityFrom
                .times(assetQuantityTo.value)
                .mod(assetQuantityFrom.value)
                .isZero()
        ) {
            throw Error(
                `The given quantity does not fit to the ratio: ${assetQuantityFrom}:${assetQuantityTo}`
            );
        }
        const remainQuantityTo = remainQuantityFrom
            .times(assetQuantityTo.value)
            .idiv(assetQuantityFrom.value);
        const remainQuantityFee = remainQuantityFrom
            .times(assetQuantityFee.value)
            .idiv(assetQuantityFrom.value);
        return new Order({
            assetTypeFrom,
            assetTypeTo,
            assetTypeFee,
            shardIdFrom,
            shardIdTo,
            shardIdFee,
            assetQuantityFrom: U64.ensure(remainQuantityFrom),
            assetQuantityTo: U64.ensure(remainQuantityTo),
            assetQuantityFee: U64.ensure(remainQuantityFee),
            originOutputs,
            expiration,
            lockScriptHashFrom,
            parametersFrom,
            lockScriptHashFee,
            parametersFee
        });
    }
}

function decomposeRecipient(
    recipient: AssetAddress
): {
    lockScriptHash: H160;
    parameters: Buffer[];
} {
    // FIXME: Clean up by abstracting the standard scripts
    const { type, payload } = recipient;
    if ("pubkeys" in payload) {
        throw Error("Multisig payload is not supported yet");
    }
    switch (type) {
        case 0x00: // LOCK_SCRIPT_HASH ONLY
            return {
                lockScriptHash: payload,
                parameters: []
            };
        case 0x01: // P2PKH
            return {
                lockScriptHash: P2PKH.getLockScriptHash(),
                parameters: [Buffer.from(payload.value, "hex")]
            };
        case 0x02: // P2PKHBurn
            return {
                lockScriptHash: P2PKHBurn.getLockScriptHash(),
                parameters: [Buffer.from(payload.value, "hex")]
            };
        default:
            throw Error(
                `Unexpected type of AssetAddress: ${type}, ${recipient}`
            );
    }
}
