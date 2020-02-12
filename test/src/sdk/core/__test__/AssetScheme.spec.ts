import { U64 } from "foundry-primitives";

import { AssetScheme } from "../AssetScheme";

test("toJSON", () => {
    const assetScheme = new AssetScheme({
        metadata: "abcd",
        supply: new U64(111),
        approver: null,
        registrar: null,
        allowedScriptHashes: [],
        pool: []
    });
    expect(AssetScheme.fromJSON(assetScheme.toJSON())).toEqual(assetScheme);
});
