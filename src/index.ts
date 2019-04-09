import {
    PlatformAddress,
    PlatformAddressValue,
    U64,
    U64Value
} from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";
import { Custom } from "codechain-sdk/lib/core/transaction/Custom";

const RLP = require("rlp");

const HANDLER_ID = 2;
const TRANSFER_CCS_ACTION_ID = 1;
const DELEGATE_CCS_ACTION_ID = 2;
const REQUEST_REVOKE_ACTION_ID = 3;

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
        throw Error("never");
    }
    const list: Buffer[] = RLP.decode(Buffer.from(data, "hex"));
    return list.map(buf => decodePlatformAddress(sdk, buf));
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
        2,
        ["Delegation", delegator.accountId.toEncodeObject()],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const list: Buffer[][] = RLP.decode(Buffer.from(data, "hex"));
    return list.map(([delegatee, quantity]) => {
        return {
            delegatee: decodePlatformAddress(sdk, delegatee),
            quantity: decodeU64(quantity)
        };
    });
}

export interface Revocation {
    delegator: PlatformAddress;
    delegatee: PlatformAddress;
    endTime: number;
    quantity: U64;
}

export async function getPendingRevocations(
    sdk: SDK,
    blockNumber?: number
): Promise<Revocation[]> {
    const data = await sdk.rpc.engine.getCustomActionData(
        2,
        ["Revocations"],
        blockNumber
    );
    if (data == null) {
        return [];
    }
    const list: Buffer[][] = RLP.decode(Buffer.from(data, "hex"));
    return list.map(([delegator, delegatee, endTime, quantity]) => {
        return {
            delegator: decodePlatformAddress(sdk, delegator),
            delegatee: decodePlatformAddress(sdk, delegatee),
            endTime: decodeNumber(endTime),
            quantity: decodeU64(quantity)
        };
    });
}

function decodeNumber(buffer: Buffer): number {
    const parsed = parseInt(buffer.toString("hex"), 16);
    if (isNaN(parsed)) {
        throw new Error("buffer is not a number");
    }
    return parsed;
}

function decodeU64(buffer: Buffer): U64 {
    return U64.ensure("0x" + buffer.toString("hex"));
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

export function createRequestRevokeTransaction(
    sdk: SDK,
    delegatee: PlatformAddressValue,
    quantity: U64Value
): Custom {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: RLP.encode([
            REQUEST_REVOKE_ACTION_ID,
            PlatformAddress.ensure(delegatee).accountId.toEncodeObject(),
            U64.ensure(quantity).toEncodeObject()
        ])
    });
}
