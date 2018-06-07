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
    sender: new H160("0x5bcd7c840f108172d94a4d084af711d879630fe6"),
    receiver: new H160("0x744142069fe2d03d48e61734cbe564fcc94e6e31"),
    value: new U256(10000)
});

// Creating Parcel
const parcelSignerNonce = new U256(0);
const fee = new U256(10);
const networkId = 17;
const parcel = new Parcel(parcelSignerNonce, fee, networkId, tx);

// Signing Parcel
const parcelSignerSecret = new H256("b15139f97aad25ae0330432aeb091ef962eee643e41dc07a1e04457c5c2c6088");
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