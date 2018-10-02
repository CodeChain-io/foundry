import { H160 } from "../../H160";
import { AssetMintTransaction } from "../AssetMintTransaction";

test("AssetMintTransaction toJSON", () => {
    const t = new AssetMintTransaction({
        networkId: "cc",
        shardId: 0,
        worldId: 0,
        metadata: "",
        output: {
            lockScriptHash: new H160(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            ),
            parameters: [],
            amount: 0
        },
        registrar: null,
        nonce: 0
    });
    expect(AssetMintTransaction.fromJSON(t.toJSON())).toEqual(t);
});
