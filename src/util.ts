import { H512, PlatformAddress, U64 } from "codechain-primitives/lib";
import { SDK } from "codechain-sdk";

export function isArrayOf<T>(
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

export function decodeUInt(buffer: Buffer): number {
    return buffer.readUIntBE(0, buffer.length);
}

export function decodeU64(buffer: Buffer): U64 {
    if (buffer.length === 0) {
        return new U64(0);
    }
    return U64.ensure("0x" + buffer.toString("hex"));
}

export function decodeH512(buffer: Buffer): H512 {
    return H512.ensure("0x" + buffer.toString("hex"));
}

export function decodePlatformAddress(
    sdk: SDK,
    buffer: Buffer
): PlatformAddress {
    const accountId = buffer.toString("hex");
    return PlatformAddress.fromAccountId(accountId, {
        networkId: sdk.networkId
    });
}
