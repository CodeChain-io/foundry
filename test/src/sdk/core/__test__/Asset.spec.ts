import { H160, H256, U64 } from "codechain-primitives";

import { Asset } from "../Asset";

test("toJSON", () => {
    const asset = new Asset({
        assetType: H160.zero(),
        shardId: 0,
        lockScriptHash: new H160("1111111111111111111111111111111111111111"),
        parameters: [],
        quantity: new U64(222),
        tracker: new H256(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ),
        transactionOutputIndex: 0
    });
    expect(Asset.fromJSON(asset.toJSON())).toEqual(asset);
});
