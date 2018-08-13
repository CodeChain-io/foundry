import { SetWorldOwnersTransaction } from "../SetWorldOwnersTransaction";
import { getTransactionFromJSON } from "../Transaction";
import { PlatformAddress } from "../../../key/classes";

describe("SetWorldOwnersTransaction", () => {
    test("toJSON", () => {
        const t = new SetWorldOwnersTransaction({
            networkId: "x1",
            shardId: 42,
            worldId: 0x42,
            owners: [PlatformAddress.fromAccountId("0x0123456789012345678901234567890123456789")],
            nonce: 0,
        });
        expect(SetWorldOwnersTransaction.fromJSON(t.toJSON())).toEqual(t);
    });

    test("getTransactionFromJSON", () => {
        const t = new SetWorldOwnersTransaction({
            networkId: "1a",
            shardId: 42,
            worldId: 0x42,
            owners: [PlatformAddress.fromAccountId("0x0123456789012345678901234567890123456789")],
            nonce: 0,
        });
        expect(getTransactionFromJSON(t.toJSON())).toEqual(t);
    });
})
