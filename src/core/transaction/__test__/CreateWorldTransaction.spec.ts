import { CreateWorldTransaction } from "../CreateWorldTransaction";
import { getTransactionFromJSON } from "../Transaction";
import { PlatformAddress } from "../../../key/classes";

describe("CreateWorldTransaction", () => {
    test("toJSON", () => {
        const t = new CreateWorldTransaction({
            networkId: "a1",
            shardId: 2,
            owners: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789"
                )
            ],
            nonce: 0
        });
        expect(CreateWorldTransaction.fromJSON(t.toJSON())).toEqual(t);
    });

    test("getTransactionFromJSON", () => {
        const t = new CreateWorldTransaction({
            networkId: "x1",
            shardId: 2,
            owners: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789"
                )
            ],
            nonce: 0
        });
        expect(getTransactionFromJSON(t.toJSON())).toEqual(t);
    });
});
