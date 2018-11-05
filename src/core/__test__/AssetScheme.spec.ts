import { AssetScheme } from "../AssetScheme";
import { U256 } from "../U256";

test("toJSON", () => {
    const assetScheme = new AssetScheme({
        metadata: "abcd",
        amount: new U256(111),
        registrar: null,
        pool: []
    });
    expect(AssetScheme.fromJSON(assetScheme.toJSON())).toEqual(assetScheme);
});
