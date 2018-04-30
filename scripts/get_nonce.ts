import { Transaction, H256, U256, Action, H160 } from "../src/primitives";
import { SDK } from "../src";
import { privateKeyToAddress } from "../src/Utils";

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd")
const sdk = new SDK("http://localhost:8080");

sdk.getNonce(new H160(privateKeyToAddress(secret.value))).then(nonce => {
    console.log(nonce);
}).catch(err => {
    console.error(err);
});
