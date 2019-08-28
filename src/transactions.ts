// FIXME: The SDK doesn't export PlatformAddressValue and U64Value.
// In the import statement below uses "codechain-primitives" which is installed by the SDK.
// We should use the SDK's PlatformAddressValue when the SDK is updated.
import { PlatformAddressValue, U64Value } from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
import { PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import { Custom } from "codechain-sdk/lib/core/transaction/Custom";
import * as RLP from "rlp";

import ReportDoubleVote from "./actions/reportDoubleVote";
import { HANDLER_ID } from "./index";
import { ConsensusMessage } from "./message";
import { decodePlatformAddress, decodeU64, decodeUInt } from "./util";

export const TRANSFER_CCS_ACTION_ID = 1;
export const DELEGATE_CCS_ACTION_ID = 2;
export const REVOKE_ACTION_ID = 3;
export const SELF_NOMINATE_ACTION_ID = 4;
export const REPORT_DOUBLE_VOTE_ACTION_ID = ReportDoubleVote.ACTION_ID;
export const REDELEGATE_ACTION_ID = 6;
export const CHANGE_PARAMS_ACTION_ID = 0xff;

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

export function createReportDoubleVoteTransaction(
    sdk: SDK,
    message1: ConsensusMessage,
    message2: ConsensusMessage
): Custom {
    const action = new ReportDoubleVote(message1, message2);
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: action.toBytes()
    });
}

export function createRedelegateTransaction(
    sdk: SDK,
    prevDelegatee: PlatformAddressValue,
    nextDelegatee: PlatformAddressValue,
    quantity: U64Value
): Custom {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            REDELEGATE_ACTION_ID,
            PlatformAddress.ensure(prevDelegatee).accountId.toEncodeObject(),
            PlatformAddress.ensure(nextDelegatee).accountId.toEncodeObject(),
            U64.ensure(quantity).toEncodeObject()
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

interface Redelegate {
    type: "redelegate";
    prevDelegatee: PlatformAddress;
    nextDelegatee: PlatformAddress;
    quantity: U64;
}

interface ChangeParams {
    type: "changeParams";
    metadataSeq: U64;
    // TODO: Use concrete type when it is needed.
    params: any;
    signatures: any[];
}

type Action =
    | TransferCCS
    | DelegateCCS
    | Revoke
    | SelfNominate
    | ReportDoubleVote
    | Redelegate
    | ChangeParams;

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
        case REPORT_DOUBLE_VOTE_ACTION_ID:
            return ReportDoubleVote.fromEncodeObject(decoded);
        case REDELEGATE_ACTION_ID:
            if (decoded.length !== 4) {
                throw new Error(
                    "A length of a RLP list of a redelegate action must be 4"
                );
            }
            return {
                type: "redelegate",
                prevDelegatee: decodePlatformAddress(sdk, decoded[1]),
                nextDelegatee: decodePlatformAddress(sdk, decoded[2]),
                quantity: decodeU64(decoded[3])
            };
        case CHANGE_PARAMS_ACTION_ID:
            if (decoded.length <= 3) {
                throw new Error(
                    "A length of a RLP list of a changeParams action should be more than 3"
                );
            }
            const signatures: any = decoded.slice(3);
            return {
                type: "changeParams",
                metadataSeq: decodeU64(decoded[1]),
                params: decoded[2],
                signatures
            };
        default:
            throw new Error("Invalid tag for a stake action");
    }
}
