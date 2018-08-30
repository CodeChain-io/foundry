import { AssetMintTransaction } from "../AssetMintTransaction";
import { H256 } from "../../H256";

test("AssetMintTransaction toJSON", () => {
    const t = new AssetMintTransaction({
        metadata: "",
        output: {
            lockScriptHash: new H256(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            ),
            parameters: [],
            amount: 0
        },
        registrar: null,
        nonce: 0
    });
    expect(AssetMintTransaction.fromJSON(t.toJSON())).toEqual(t);
});
