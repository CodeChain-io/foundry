import { Asset } from "../Asset";
import { H256 } from "../H256";

test("toJSON", () => {
    const asset = new Asset({
        assetType: new H256(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ),
        lockScriptHash: new H256(
            "1111111111111111111111111111111111111111111111111111111111111111"
        ),
        parameters: [],
        amount: 222,
        transactionHash: new H256(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ),
        transactionOutputIndex: 0
    });
    expect(Asset.fromJSON(asset.toJSON())).toEqual(asset);
});
