import { AssetMintTransaction, Invoice, SDK, H160, H256, U256, Parcel, PaymentTransaction, privateKeyToAddress } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK(SERVER_URL);

test("getTransactionInvoice", async () => {
    const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
    const address = new H160(privateKeyToAddress(secret.value));
    const nonce = await sdk.getNonce(address);

    const assetMintTransaction = new AssetMintTransaction({
        nonce: 1,
        metadata: "",
        lockScriptHash: new H256("0000000000000000000000000000000000000000000000000000000000000000"),
        parameters: [],
        amount: 111,
        registrar: null
    });
    const fee = new U256(10);
    const networkId = 17;
    const p = Parcel.transactions(nonce, fee, networkId, assetMintTransaction).sign(secret);
    await sdk.sendSignedParcel(p);
    expect(await sdk.getTransactionInvoice(assetMintTransaction.hash())).toEqual(new Invoice(true));
});
