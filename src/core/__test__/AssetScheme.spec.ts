import { AssetScheme } from "../AssetScheme";

test("toJSON", () => {
    const assetScheme = new AssetScheme({
        metadata: "abcd",
        amount: 111,
        registrar: null,
        pool: []
    });
    expect(AssetScheme.fromJSON(assetScheme.toJSON())).toEqual(assetScheme);
});
