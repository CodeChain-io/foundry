import { AssetMintTransaction } from "..";
import { H256 } from "../..";

test("AssetMintTransaction toJSON", () => {
    const t = new AssetMintTransaction({
        metadata: "",
        lockScriptHash: new H256("eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"),
        parameters: [],
        amount: 0,
        registrar: null,
        nonce: 0,
    });
    expect(AssetMintTransaction.fromJSON(t.toJSON())).toEqual(t);
});
