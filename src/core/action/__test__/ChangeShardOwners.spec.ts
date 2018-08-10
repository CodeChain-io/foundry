import { ChangeShardOwners } from "../ChangeShardOwners";
import { H160 } from "../../H160";
import { getActionFromJSON } from "../Action";

describe("ChangeShardOwners", () => {
    test("getActionFromJSON", () => {
        const t = new ChangeShardOwners({
            shardId: 42,
            owners: [H160.ensure("0x0123456789012345678901234567890123456789")]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
})
