import { PlatformAddress } from "codechain-primitives";

import { getActionFromJSON } from "../Action";
import { SetShardOwners } from "../SetShardOwners";

describe("SetShardOwners", () => {
    test("getActionFromJSON", () => {
        const t = new SetShardOwners({
            shardId: 42,
            owners: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789",
                    { networkId: "tc" }
                )
            ]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
});
