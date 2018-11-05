import { PlatformAddress } from "codechain-primitives";

import { getActionFromJSON } from "../Action";
import { SetShardUsers } from "../SetShardUsers";

describe("SetShardUsers", () => {
    test("getActionFromJSON", () => {
        const t = new SetShardUsers({
            shardId: 42,
            users: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789",
                    { networkId: "tc" }
                )
            ]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
});
