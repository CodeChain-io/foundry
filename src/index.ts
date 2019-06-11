import {
    H256,
    H512,
    PlatformAddress,
    PlatformAddressValue,
    U64,
    U64Value
} from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
import { Custom } from "codechain-sdk/lib/core/transaction/Custom";

const RLP = require("rlp");

export const HANDLER_ID = 2;
const TRANSFER_CCS_ACTION_ID = 1;
const DELEGATE_CCS_ACTION_ID = 2;
const REVOKE_ACTION_ID = 3;
const SELF_NOMINATE_ACTION_ID = 4;

export async function getUndelegatedCCS(
    sdk: SDK,
    address: PlatformAddressValue,
    blockNumber?: number
): Promise<U64> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        [
            "Account",
            PlatformAddress.ensure(address)
                .getAccountId()
                .toEncodeObject()
        ],
        blockNumber
    );
    if (data == null) {
        return new U64(0);
    }
    return decodeU64(RLP.decode(Buffer.from(data, "hex")));
}

export async function getCCSHolders(
    sdk: SDK,
    blockNumber?: number
): Promise<PlatformAddress[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["StakeholderAddresses"],
        blockNumber
    );
    if (data == null) {
        throw Error("Expected non-null value, but got a null");
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    if (!isArrayOf<Buffer>(decoded, Buffer.isBuffer)) {
        throw Error(
            "Expected a rlp of Array<Buffer>, but got an invalid shaped value"
        );
    }
    return decoded.map(buf => decodePlatformAddress(sdk, buf));
}

export interface Delegation {
    delegatee: PlatformAddress;
    quantity: U64;
}

export async function getDelegations(
    sdk: SDK,
    delegator: PlatformAddress,
    blockNumber?: number
): Promise<Delegation[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["Delegation", delegator.accountId.toEncodeObject()],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    function isDelegationShape(entry: any): entry is Buffer[] {
        return entry != null && Array.isArray(entry) && entry.length === 2;
    }
    if (!isArrayOf<Buffer[]>(decoded, isDelegationShape)) {
        throw new Error(
            "Expected a rlp of Array<Buffer[4]>, but got an invalid shaped value"
        );
    }
    return decoded.map(([delegatee, quantity]) => {
        return {
            delegatee: decodePlatformAddress(sdk, delegatee),
            quantity: decodeU64(quantity)
        };
    });
}

export interface Candidate {
    pubkey: H512;
    deposit: U64;
    nominationEndsAt: U64;
    metadata: Buffer;
}

export async function getCandidates(
    sdk: SDK,
    blockNumber?: number
): Promise<Candidate[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["Candidates"],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    function isCandidateShape(entry: any): entry is Buffer[] {
        return entry != null && Array.isArray(entry) && entry.length === 4;
    }
    if (!isArrayOf<Buffer[]>(decoded, isCandidateShape)) {
        throw new Error(
            "Expected a rlp of Array<Buffer[4]>, but got an invalid shaped value"
        );
    }
    return decoded.map(([pubkey, deposit, nominationEndsAt, metadata]) => ({
        pubkey: decodeH512(pubkey),
        deposit: decodeU64(deposit),
        nominationEndsAt: decodeU64(nominationEndsAt),
        metadata
    }));
}

export interface Prisoner {
    address: PlatformAddress;
    deposit: U64;
    custodyUntil: U64;
    releasedAt: U64;
}

export async function getJailed(
    sdk: SDK,
    blockNumber?: number
): Promise<Prisoner[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["Jail"],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    const isCandidateShape = (entry: any): entry is Buffer[] =>
        entry != null && Array.isArray(entry) && entry.length === 4;
    if (!isArrayOf<Buffer[]>(decoded, isCandidateShape)) {
        throw new Error(
            "Expected a rlp of Array<Buffer[4]>, but got an invalid shaped value"
        );
    }
    return decoded.map(([address, deposit, custodyUntil, releasedAt]) => ({
        address: decodePlatformAddress(sdk, address),
        deposit: decodeU64(deposit),
        custodyUntil: decodeU64(custodyUntil),
        releasedAt: decodeU64(releasedAt)
    }));
}

export async function getBanned(
    sdk: SDK,
    blockNumber?: number
): Promise<PlatformAddress[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["Banned"],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    if (!isArrayOf<Buffer>(decoded, Buffer.isBuffer)) {
        throw new Error(
            "Expected a rlp of Array<Buffer>, but an invalid shaped value"
        );
    }
    return decoded.map(address => decodePlatformAddress(sdk, address));
}

interface IntermediateRewards {
    previous: IntermediateReward[];
    current: IntermediateReward[];
}
interface IntermediateReward {
    address: PlatformAddress;
    quantity: U64;
}

export async function getIntermediateRewards(
    sdk: SDK,
    blockNumber?: number
): Promise<IntermediateRewards> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["IntermediateRewards"],
        blockNumber
    );
    if (data == null) {
        return {
            previous: [],
            current: []
        };
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    function isIntermediateRewardShape(entry: any): entry is Buffer[] {
        return entry != null && Array.isArray(entry) && entry.length === 2;
    }
    function isIntermediateRewardsFieldShape(entry: any): entry is Buffer[][] {
        return isArrayOf<Buffer[]>(entry, isIntermediateRewardShape);
    }
    if (
        !isArrayOf<Buffer[][]>(decoded, isIntermediateRewardsFieldShape) ||
        decoded.length !== 2
    ) {
        throw new Error(
            "Expected a rlp of Buffer[2][][2], but an invalid shaped value"
        );
    }
    function convert(entries: Buffer[][]): IntermediateReward[] {
        return entries.map(([address, quantity]) => ({
            address: decodePlatformAddress(sdk, address),
            quantity: decodeU64(quantity)
        }));
    }
    return {
        previous: convert(decoded[0]),
        current: convert(decoded[1])
    };
}

function isArrayOf<T>(
    list: any,
    predicate: (entry: any) => entry is T
): list is Array<T> {
    if (list == null) {
        return false;
    }
    if (!Array.isArray(list)) {
        return false;
    }
    return list.every(predicate);
}

function decodeUInt(buffer: Buffer): number {
    return buffer.readUIntBE(0, buffer.length);
}

function decodeU64(buffer: Buffer): U64 {
    return U64.ensure("0x" + buffer.toString("hex"));
}

function decodeH256(buffer: Buffer): H256 {
    return H256.ensure("0x" + buffer.toString("hex"));
}

function decodeH512(buffer: Buffer): H512 {
    return H512.ensure("0x" + buffer.toString("hex"));
}

function decodePlatformAddress(sdk: SDK, buffer: Buffer): PlatformAddress {
    const accountId = buffer.toString("hex");
    return PlatformAddress.fromAccountId(accountId, {
        networkId: sdk.networkId
    });
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
