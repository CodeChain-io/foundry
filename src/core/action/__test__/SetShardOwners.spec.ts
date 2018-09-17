import { PlatformAddress } from "codechain-primitives";

import { SetShardOwners } from "../SetShardOwners";
import { getActionFromJSON } from "../Action";

describe("SetShardOwners", () => {
    test("getActionFromJSON", () => {
        const t = new SetShardOwners({
            shardId: 42,
            owners: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789"
                )
            ]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
});
