import { SDK, H256, AssetMintTransaction, AssetScheme } from "../";
import { mintAsset } from "./helper";

const SERVER_URL = "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getAssetScheme", async () => {
    const mintTransaction = await mintAsset({
        metadata: "",
        lockScriptHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        parameters: [],
        amount: 111,
        registrar: null
    });
    const assetScheme = await sdk.getAssetScheme(mintTransaction.hash());
    expect(assetScheme).toEqual(new AssetScheme({
        metadata: "",
        amount: 111,
        registrar: null
    }));
});