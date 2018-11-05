import { Asset } from "../Asset";
import { H160 } from "../H160";
import { H256 } from "../H256";
import { U256 } from "../U256";

test("toJSON", () => {
    const asset = new Asset({
        assetType: new H256(
            "0000000000000000000000000000000000000000000000000000000000000000"
        ),
        lockScriptHash: new H160("1111111111111111111111111111111111111111"),
        parameters: [],
        amount: new U256(222),
        transactionHash: new H256(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ),
        transactionOutputIndex: 0
    });
    expect(Asset.fromJSON(asset.toJSON())).toEqual(asset);
});
