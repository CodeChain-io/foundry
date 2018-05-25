import { Asset, H256 } from "..";

test("toJSON", () => {
    const asset = new Asset({
        assetType: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        lockScriptHash: new H256("1111111111111111111111111111111111111111111111111111111111111111"),
        parameters: [],
        amount: 222,
    });
    expect(Asset.fromJSON(asset.toJSON())).toEqual(asset);
});
