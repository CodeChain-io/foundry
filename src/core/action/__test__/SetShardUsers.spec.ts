import { PlatformAddress } from "codechain-primitives";

import { SetShardUsers } from "../SetShardUsers";
import { getActionFromJSON } from "../Action";

describe("SetShardUsers", () => {
    test("getActionFromJSON", () => {
        const t = new SetShardUsers({
            shardId: 42,
            users: [
                PlatformAddress.fromAccountId(
                    "0x0123456789012345678901234567890123456789"
                )
            ]
        });
        expect(getActionFromJSON(t.toJSON())).toEqual(t);
    });
});
