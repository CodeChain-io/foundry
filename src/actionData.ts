import {
    H512,
    PlatformAddress,
    PlatformAddressValue,
    U64
} from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
const RLP = require("rlp");
import { HANDLER_ID } from "./index";
import {
    decodeH512,
    decodePlatformAddress,
    decodeU64,
    isArrayOf
} from "./util";

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

export interface IntermediateRewards {
    previous: IntermediateReward[];
    current: IntermediateReward[];
}

export interface IntermediateReward {
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

export interface Validator {
    weight: U64;
    delegation: U64;
    deposit: U64;
    pubkey: H512;
}

export async function getValidators(
    sdk: SDK,
    blockNumber?: number
): Promise<Validator[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["Validators"],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const decoded = RLP.decode(Buffer.from(data, "hex"));
    function isValidatorShape(entry: any): entry is Buffer[] {
        return entry != null && Array.isArray(entry) && entry.length === 4;
    }
    if (!isArrayOf<Buffer[]>(decoded, isValidatorShape)) {
        throw new Error(
            "Expected a rlp of Buffer[4][], but an invalid shaped value"
        );
    }
    return decoded.map(([weight, delegation, deposit, pubkey]) => ({
        weight: decodeU64(weight),
        delegation: decodeU64(delegation),
        deposit: decodeU64(deposit),
        pubkey: decodeH512(pubkey)
    }));
}
