# Install package (not available yet)

```sh
npm install codechain-sdk
```

or

```sh
yarn add codechain-sdk
```

# Usage example
Make sure that your CodeChain RPC server is listening. In the examples, we assume it is localhost:8080

## Send PaymentTransaction

```javascript
import { SDK, Parcel, U256, H256, H160, PaymentTransaction } from "codechain-sdk";

const sdk = new SDK("http://localhost:8080");

// Creating PaymentTransaction
const senderNonce = new U256(0);
const tx = new PaymentTransaction({
    nonce: senderNonce,
    sender: new H160("0x1111111111111111111111111111111111111111"),
    receiver: new H160("0x2222222222222222222222222222222222222222"),
    value: new U256(10000)
});

// Creating Parcel
const parcelSignerNonce = new U256(0);
const fee = new U256(10);
const networkId = 17;
const parcel = new Parcel(parcelSignerNonce, fee, networkId, tx);

// Signing Parcel
const parcelSignerSecret = new H256("0x3434343434343434343434343434343434343434343434343434343434343434");
const signedParcel = parcel.sign(parcelSignerSecret);
sdk.sendSignedParcel(signedParcel).then((hash) => {
    console.log(`Parcel sent:`, hash);
}).catch((err) => {
    console.error(`Error while sendSignedParcel():`, err);
});

```

# [SDK](classes/sdk.html) methods
 * getAsset
 * getAssetScheme
 * getBalance
 * getBlock
 * getBlockHash
 * getBlockNumber
 * getNonce
 * getParcel
 * getParcelInvoices
 * getPendingParcels
 * getRegularKey
 * getTransactionInvoice
 * ping
 * sendSignedParcel

# Transactions
 * [PaymentTransaction](classes/paymenttransaction.html)
 * [SetRegularKeyTransaction](classes/setregularkeytransaction.html)
 * [AssetMintTransaction](classes/assetminttransaction.html)
 * [AssetTransferTransaction](classes/assettransfertransaction.html)