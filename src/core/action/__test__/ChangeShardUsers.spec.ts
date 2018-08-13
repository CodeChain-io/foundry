import { ChangeShardUsers } from "../ChangeShardUsers";
import { PlatformAddress } from "../../../key/PlatformAddress";
import { getActionFromJSON } from "../Action";

describe("ChangeShardUsers", () => {
    test("getActionFromJSON", () => {
        const t = new ChangeShardUsers({
            shardId: 42,
            users: [PlatformAddress.fromAccountId("0x0123456789012345678901234567890123456789")]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
})
