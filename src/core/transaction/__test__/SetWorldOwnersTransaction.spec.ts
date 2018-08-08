import { SetWorldOwnersTransaction } from "../SetWorldOwnersTransaction";
import { H160 } from "../../H160";
import { getTransactionFromJSON } from "../Transaction";

describe("SetWorldOwnersTransaction", () => {
    test("toJSON", () => {
        const t = new SetWorldOwnersTransaction({
            networkId: 1,
            shardId: 42,
            worldId: 0x42,
            owners: [H160.ensure("0x0123456789012345678901234567890123456789")],
            nonce: 0,
        });
        expect(SetWorldOwnersTransaction.fromJSON(t.toJSON())).toEqual(t);
    });

    test("getTransactionFromJSON", () => {
        const t = new SetWorldOwnersTransaction({
            networkId: 1,
            shardId: 42,
            worldId: 0x42,
            owners: [H160.ensure("0x0123456789012345678901234567890123456789")],
            nonce: 0,
        });
        expect(getTransactionFromJSON(t.toJSON())).toEqual(t);
    });
})
