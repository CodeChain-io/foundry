import { Transaction, H256, U256, Action, H160 } from "../src/primitives";
import { privateKeyToAddress } from "../src/Utils";
import { SDK } from "../src";

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd")
const address = new H160(privateKeyToAddress(secret.value));
const networkId = Number.parseInt(process.env.NETWORK_ID) || 17;

const sdk = new SDK("http://localhost:8080");

sdk.getNonce(address).then(nonce => {
    console.log(nonce);

    const fee = new U256(10);
    const action = new Action("noop");
    const t = new Transaction(nonce, fee, action, networkId);
    return sdk.sendSignedTransaction(t.sign(secret));
}).then(hash => {
    console.log(hash);

    return sdk.getTransactionInvoice(hash, 0);
}).then(invoice => {
    if (invoice === null) {
        return console.log("Invoice not found");
    }
    console.log(invoice);
}).catch(err => {
    console.error(err);
});
