import { SDK } from "../";
import { mintAsset } from "./helper";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getAssetScheme", async () => {
    const { H256, AssetScheme } = SDK.Core.classes;
    const { assetMintTransaction } = await mintAsset({
        metadata: "",
        lockScriptHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        amount: 111,
        registrar: null
    });
    const assetScheme = await sdk.rpc.chain.getAssetScheme(assetMintTransaction.hash());
    expect(assetScheme).toEqual(new AssetScheme({
        metadata: "",
        amount: 111,
        registrar: null
    }));
});