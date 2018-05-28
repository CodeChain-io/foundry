import { Parcel, H256, U256, H160 } from "../src/primitives";
import { privateKeyToAddress } from "../src/utils";
import { SDK } from "../src";
import { PaymentTransaction, AssetMintTransaction, AssetTransferTransaction } from "../src/primitives/transaction/";

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));
const networkId = Number.parseInt(process.env.NETWORK_ID) || 17;

const sdk = new SDK("http://localhost:8080");

const paymentTransaction = new PaymentTransaction({
    nonce: 0,
    sender: address,
    receiver: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("10")
});
const assetMintTransaction = new AssetMintTransaction({
    metadata: "",
    lockScriptHash: new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"),
    parameters: [],
    amount: 100,
    registrar: null,
    nonce: 0,
});

sdk.getNonce(address).then(nonce => {
    console.log(nonce);

    const fee = new U256(10);
    const p = new Parcel(nonce, fee, networkId, assetMintTransaction);
    return sdk.sendSignedParcel(p.sign(secret));
}).then(hash => {
    console.log(hash);
    return sdk.getParcelInvoices(hash, 0);
}).then(invoice => {
    if (invoice === null) {
        return console.log("Invoice not found");
    }
    console.log(invoice);
}).catch(err => {
    console.error(err);
});
