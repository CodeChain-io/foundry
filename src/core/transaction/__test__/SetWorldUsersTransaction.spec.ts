import { SetWorldUsersTransaction } from "../SetWorldUsersTransaction";
import { getTransactionFromJSON } from "../Transaction";
import { PlatformAddress } from "../../../key/classes";

describe("SetWorldUsersTransaction", () => {
    test("toJSON", () => {
        const t = new SetWorldUsersTransaction({
            networkId: "12",
            shardId: 42,
            worldId: 0x42,
            users: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789"
                )
            ],
            nonce: 0
        });
        expect(SetWorldUsersTransaction.fromJSON(t.toJSON())).toEqual(t);
    });

    test("getTransactionFromJSON", () => {
        const t = new SetWorldUsersTransaction({
            networkId: "12",
            shardId: 42,
            worldId: 0x42,
            users: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789"
                )
            ],
            nonce: 0
        });
        expect(getTransactionFromJSON(t.toJSON())).toEqual(t);
    });
});
