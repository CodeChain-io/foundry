import { ChangeShardUsers } from "../ChangeShardUsers";
import { H160 } from "../../H160";
import { getActionFromJSON } from "../Action";

describe("ChangeShardUsers", () => {
    test("getActionFromJSON", () => {
        const t = new ChangeShardUsers({
            shardId: 42,
            users: [H160.ensure("0x0123456789012345678901234567890123456789")]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
})
