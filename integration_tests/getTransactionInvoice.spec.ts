import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });

test("getTransactionInvoice", async () => {
    const { AssetMintTransaction, H256, Invoice } = SDK.Core.classes;

    const secret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
    const address = SDK.util.getAccountIdFromPrivate(secret);
    const nonce = await sdk.rpc.chain.getNonce(address);

    const assetMintTransaction = new AssetMintTransaction({
        networkId: 17,
        nonce: 1,
        metadata: "",
        lockScriptHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        parameters: [],
        amount: 111,
        registrar: null
    });
    const p = sdk.core.createChangeShardStateParcel({
        transactions: [assetMintTransaction],
        nonce,
        fee: 10
    }).sign(secret);
    await sdk.rpc.chain.sendSignedParcel(p);
    expect(await sdk.rpc.chain.getTransactionInvoice(assetMintTransaction.hash())).toEqual(new Invoice(true));
});
