import { SDK } from "../";
import { mintAsset } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getAsset", async () => {
    const { H256, Asset } = SDK.Core.classes;
    const { assetMintTransaction } = await mintAsset({
        metadata: "",
        lockScriptHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        amount: 111,
        registrar: null
    });
    const asset = await sdk.rpc.chain.getAsset(assetMintTransaction.hash(), 0);
    expect(asset).toEqual(new Asset({
        assetType: assetMintTransaction.getAssetSchemeAddress(),
        lockScriptHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        parameters: [],
        amount: 111,
        transactionHash: assetMintTransaction.hash(),
        transactionOutputIndex: 0
    }));
});
