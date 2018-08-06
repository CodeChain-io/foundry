import { CreateWorldTransaction } from "../CreateWorldTransaction";
import { H160 } from "../../H160";
import { getTransactionFromJSON } from "../Transaction";

describe("CreateWorldTransaction", () => {
    test("toJSON", () => {
        const t = new CreateWorldTransaction({
            networkId: 1,
            shardId: 2,
            owners: [H160.ensure("0x0123456789012345678901234567890123456789")],
            nonce: 0,
        });
        expect(CreateWorldTransaction.fromJSON(t.toJSON())).toEqual(t);
    });

    test("getTransactionFromJSON", () => {
        const t = new CreateWorldTransaction({
            networkId: 1,
            shardId: 2,
            owners: [H160.ensure("0x0123456789012345678901234567890123456789")],
            nonce: 0,
        });
        expect(getTransactionFromJSON(t.toJSON())).toEqual(t);
    });
})
