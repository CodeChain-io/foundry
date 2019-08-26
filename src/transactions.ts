// FIXME: The SDK doesn't export PlatformAddressValue and U64Value.
// In the import statement below uses "codechain-primitives" which is installed by the SDK.
// We should use the SDK's PlatformAddressValue when the SDK is updated.
import { PlatformAddressValue, U64Value } from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
import { H512, PlatformAddress, U64 } from "codechain-sdk/lib/core/classes";
import { Custom } from "codechain-sdk/lib/core/transaction/Custom";
import * as RLP from "rlp";

import { HANDLER_ID } from "./index";
import { ConsensusMessage, isStep } from "./message";
import {
    decodeH512,
    decodePlatformAddress,
    decodeU64,
    decodeUInt
} from "./util";

export const TRANSFER_CCS_ACTION_ID = 1;
export const DELEGATE_CCS_ACTION_ID = 2;
export const REVOKE_ACTION_ID = 3;
export const SELF_NOMINATE_ACTION_ID = 4;
export const REPORT_DOUBLE_VOTE_ACTION_ID = 5;
export const REDELEGATE_ACTION_ID = 6;
export const CHANGE_PARAMS_ACTION_ID = 0xff;

function messageToEncodeObject(message: ConsensusMessage) {
    return [
        [
            [
                message.on.step.height.toEncodeObject(),
                message.on.step.view.toEncodeObject(),
                message.on.step.step
            ],
            message.on.blockHash == null
                ? []
                : [message.on.blockHash.toEncodeObject()]
        ],
        message.signature.toEncodeObject(),
        message.signerIndex.toEncodeObject()
    ];
}

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
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            REPORT_DOUBLE_VOTE_ACTION_ID,
            messageToEncodeObject(message1),
            messageToEncodeObject(message2)
        ])
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

interface ReportDoubleVote {
    type: "reportDoubleVote";
    message1: ConsensusMessage;
    message2: ConsensusMessage;
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

function decodeMessage(list: any[]): ConsensusMessage {
    if (list.length !== 3) {
        throw new Error(
            "The raw value of ConsensusMessage should be a list of length 3"
        );
    }
    if (!Array.isArray(list[0]) || list[0].length !== 2) {
        throw new Error("The raw value of VoteOn should be a list of length 3");
    }
    if (!Array.isArray(list[0][0]) || list[0][0].length !== 3) {
        throw new Error(
            "The raw value of VoteStep should be a list of length 3"
        );
    }
    const step: number = decodeUInt(list[0][0][2]);
    if (!isStep(step)) {
        throw new Error("The consensus step should be in valid range");
    }

    const voteStep: ConsensusMessage["on"]["step"] = {
        height: decodeU64(list[0][0][0]),
        view: decodeU64(list[0][0][1]),
        step
    };

    if (!Array.isArray(list[0][1])) {
        throw new Error("The raw value of blockHash should be a list");
    }
    let blockHash: H512 | null;
    if (list[0][1].length === 0) {
        blockHash = null;
    } else if (list[0][1].length === 1) {
        blockHash = decodeH512(list[0][1][0]);
    } else {
        throw new Error(
            "The raw value of blockHash should be a list of length 0 or 1"
        );
    }

    const signature = decodeH512(list[1]);
    const signerIndex = decodeU64(list[2]);

    return {
        on: {
            step: voteStep,
            blockHash
        },
        signature,
        signerIndex
    };
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
            if (decoded.length !== 3) {
                throw new Error(
                    "A length of a RLP list of a reportDoubleVote action must be 3"
                );
            }
            return {
                type: "reportDoubleVote",
                message1: decodeMessage(decoded[1]),
                message2: decodeMessage(decoded[2])
            };
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
