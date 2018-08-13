import { ChangeShardOwners } from "../ChangeShardOwners";
import { PlatformAddress } from "../../../key/PlatformAddress";
import { getActionFromJSON } from "../Action";

describe("ChangeShardOwners", () => {
    test("getActionFromJSON", () => {
        const t = new ChangeShardOwners({
            shardId: 42,
            owners: [PlatformAddress.fromAccountId("0x0123456789012345678901234567890123456789")]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
})
