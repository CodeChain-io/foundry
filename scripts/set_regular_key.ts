import { Parcel, H256, U256, H160, H512 } from "../src/primitives";
import { privateKeyToAddress } from "../src/utils";
import { SDK } from "../src";
import { NoopTransaction, PaymentTransaction, AssetMintTransaction, AssetTransferTransaction } from "../src/primitives/transaction/";
import { SetRegularKeyTransaction } from "../src/primitives/transaction/SetRegularKeyTransaction";

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));
const networkId = Number.parseInt(process.env.NETWORK_ID) || 17;

const sdk = new SDK("http://localhost:8080");

sdk.getNonce(address).then(nonce => {
    console.log(nonce.value.toString());

    const fee = new U256(10);
    const setRegularKeyTransaction = new SetRegularKeyTransaction({
        nonce: nonce.increase(),
        key: new H512("beefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeefbeef"),
        address
    });
    const p = new Parcel(nonce, fee, networkId, setRegularKeyTransaction);
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
