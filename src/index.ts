import { SDK } from "codechain-sdk";
import { H160, PlatformAddress } from "codechain-sdk/lib/core/classes";
import { Custom } from "codechain-sdk/lib/core/transaction/Custom";

const RLP = require("rlp");

const HANDLER_ID = 2;
const TRANSFER_CSS_ACTION_ID = 1;

export const getCCSBalance = async (
    sdk: SDK,
    address: string,
    blockNumber?: number
): Promise<any> => {
    // FIXME: Interpret the result
    return sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        [H160.ensure(address).toEncodeObject()],
        blockNumber
    );
};

export const getCCSHolders = (sdk: SDK, blockNumber?: number): Promise<any> => {
    // FIXME: Interpret the result
    return sdk.rpc.engine.getCustomActionData(
        HANDLER_ID,
        ["StakeholderAddresses"],
        blockNumber
    );
};

export const createTransferCCSTransaction = (
    sdk: SDK,
    recipient: string,
    quantity: number
): Custom => {
    return sdk.core.createCustomTransaction({
        handlerId: HANDLER_ID,
        bytes: Buffer.from([
            RLP.encode(
                TRANSFER_CSS_ACTION_ID,
                PlatformAddress.ensure(recipient).accountId.toEncodeObject(),
                quantity
            )
        ])
    });
};
