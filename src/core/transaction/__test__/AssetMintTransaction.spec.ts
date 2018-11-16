import { H160 } from "../../H160";
import { U64 } from "../../U64";
import { AssetMintOutput } from "../AssetMintOutput";
import { AssetMintTransaction } from "../AssetMintTransaction";

test("AssetMintTransaction toJSON", () => {
    const t = new AssetMintTransaction({
        networkId: "cc",
        shardId: 0,
        metadata: "",
        output: new AssetMintOutput({
            lockScriptHash: new H160(
                "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
            ),
            parameters: [],
            amount: new U64(0)
        }),
        registrar: null
    });
    expect(AssetMintTransaction.fromJSON(t.toJSON())).toEqual(t);
});
