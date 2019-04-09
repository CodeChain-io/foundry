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

export async function getUndelegatedCCS(
    sdk: SDK,
    address: PlatformAddressValue,
    blockNumber?: number
): Promise<U64> {
    return sdk.rpc.engine
        .getCustomActionData(
            HANDLER_ID,
            [
                PlatformAddress.ensure(address)
                    .getAccountId()
                    .toEncodeObject()
            ],
            blockNumber
        )
        .then(data => {
            if (data == null) {
                return new U64(0);
            }
            const balance = RLP.decode(Buffer.from(data, "hex"));
            return U64.ensure("0x" + balance.toString("hex"));
        });
}

export async function getCCSHolders(
    sdk: SDK,
    blockNumber?: number
): Promise<PlatformAddress[]> {
    return sdk.rpc.engine
        .getCustomActionData(HANDLER_ID, ["StakeholderAddresses"], blockNumber)
        .then(data => {
            if (data == null) {
                throw Error("never");
            }
            const accountIds: string[] = RLP.decode(
                Buffer.from(data, "hex")
            ).map((buf: Buffer) => buf.toString("hex"));
            return accountIds.map(accountId =>
                PlatformAddress.fromAccountId(accountId, {
                    networkId: sdk.networkId
                })
            );
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
