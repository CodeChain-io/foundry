
import { Parcel, H256, U256, H160 } from "../src/primitives";
import { privateKeyToAddress } from "../src/utils";
import { SDK } from "../src";
import { NoopTransaction, PaymentTransaction } from "../src/primitives/transaction/";

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));
const networkId = Number.parseInt(process.env.NETWORK_ID) || 17;

const sdk = new SDK("http://localhost:8080");


sdk.getNonce(address).then(nonce => {
    console.log(nonce);
  //const t = new NoopTransaction();
    const t = new PaymentTransaction({
        nonce: nonce.increase(),
        address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
        value: new U256("10")
    });
    const p = new Parcel(nonce, new U256(10), networkId, t);
    return sdk.sendSignedParcel(p.sign(secret));
}).then(hash => {
    console.log(hash);
    return sdk.getParcel(hash);
}).then(parcel => {
    console.log(parcel);
}).catch(err => {
    console.error(err);
});