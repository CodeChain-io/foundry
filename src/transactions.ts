import {
    PlatformAddress,
    PlatformAddressValue,
    U64,
    U64Value
} from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
import { Custom } from "codechain-sdk/lib/core/transaction/Custom";
const RLP = require("rlp");
import { HANDLER_ID } from "./index";
import { decodePlatformAddress, decodeU64, decodeUInt } from "./util";

export const TRANSFER_CCS_ACTION_ID = 1;
export const DELEGATE_CCS_ACTION_ID = 2;
export const REVOKE_ACTION_ID = 3;
export const SELF_NOMINATE_ACTION_ID = 4;

export function createTransferCCSTransaction(
    sdk: SDK,
    recipient: PlatformAddressValue,
    quantity: U64Value
): Custom {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            TRANSFER_CCS_ACTION_ID,
            PlatformAddress.ensure(recipient).accountId.toEncodeObject(),
            U64.ensure(quantity).toEncodeObject()
        ])
    });
}

export function createDelegateCCSTransaction(
    sdk: SDK,
    delegatee: PlatformAddressValue,
    quantity: U64Value
): Custom {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            DELEGATE_CCS_ACTION_ID,
            PlatformAddress.ensure(delegatee).accountId.toEncodeObject(),
            U64.ensure(quantity).toEncodeObject()
        ])
    });
}

export function createRevokeTransaction(
    sdk: SDK,
    delegatee: PlatformAddressValue,
    quantity: U64Value
): Custom {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            REVOKE_ACTION_ID,
            PlatformAddress.ensure(delegatee).accountId.toEncodeObject(),
            U64.ensure(quantity).toEncodeObject()
        ])
    });
}

export function createSelfNominateTransaction(
    sdk: SDK,
    deposit: U64Value,
    metadata: Buffer | string
): Custom {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            SELF_NOMINATE_ACTION_ID,
            U64.ensure(deposit).toEncodeObject(),
            metadata
        ])
    });
}

interface TransferCCS {
    type: "transferCCS";
    recipient: PlatformAddress;
    quantity: U64;
}

interface DelegateCCS {
    type: "delegateCCS";
    delegatee: PlatformAddress;
    quantity: U64;
}

interface Revoke {
    type: "revoke";
    delegatee: PlatformAddress;
    quantity: U64;
}

interface SelfNominate {
    type: "selfNominate";
    deposit: U64;
    metadata: Buffer;
}

type Action = TransferCCS | DelegateCCS | Revoke | SelfNominate;

export function actionFromCustom(sdk: SDK, custom: Custom): Action | null {
    const { handlerId, bytes } = custom as any;
    if (!U64.ensure(handlerId).eq(HANDLER_ID)) {
        return null;
    }
    if (!Buffer.isBuffer(bytes)) {
        throw new Error("bytes should be a number");
    }
    return actionFromRLP(sdk, bytes);
}

export function actionFromRLP(sdk: SDK, rlp: Buffer): Action {
    const decoded = RLP.decode(rlp);
    if (
        !Array.isArray(decoded) ||
        decoded.length < 1 ||
        !Buffer.isBuffer(decoded[0])
    ) {
        throw new Error(
            "RLP of a stake action must be an array and it should have at least a tag as a first item"
        );
    }

    switch (decodeUInt(decoded[0])) {
        case TRANSFER_CCS_ACTION_ID:
            if (decoded.length !== 3) {
                throw new Error(
                    "A length of a RLP list of a transferCCS action must be 3"
                );
            }
            return {
                type: "transferCCS",
                recipient: decodePlatformAddress(sdk, decoded[1]),
                quantity: decodeU64(decoded[2])
            };
        case DELEGATE_CCS_ACTION_ID:
            if (decoded.length !== 3) {
                throw new Error(
                    "A length of a RLP list of a delegateCCS action must be 3"
                );
            }
            return {
                type: "delegateCCS",
                delegatee: decodePlatformAddress(sdk, decoded[1]),
                quantity: decodeU64(decoded[2])
            };
        case REVOKE_ACTION_ID:
            if (decoded.length !== 3) {
                throw new Error(
                    "A length of a RLP list of a revoke action must be 3"
                );
            }
            return {
                type: "revoke",
                delegatee: decodePlatformAddress(sdk, decoded[1]),
                quantity: decodeU64(decoded[2])
            };
        case SELF_NOMINATE_ACTION_ID:
            if (decoded.length !== 3) {
                throw new Error(
                    "A length of a RLP list of a selfNominate action must be 3"
                );
            }
            if (!Buffer.isBuffer(decoded[2])) {
                throw new Error(
                    "The metadata field of a RLP encoded selfNominate action must be a string"
                );
            }
            return {
                type: "selfNominate",
                deposit: decodeU64(decoded[1]),
                metadata: decoded[2]
            };
        default:
            throw new Error("Invalid tag for a stake action");
    }
}
