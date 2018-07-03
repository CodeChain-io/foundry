# Install package

`npm install codechain-sdk` or `yarn add codechain-sdk`

# Usage example
Make sure that your CodeChain RPC server is listening. In the examples, we assume it is localhost:8080

## Send 10000 CCC From One Party To Another
This example involves sending CCC from one party to another.
First, make sure to import the correct sdk and use the proper server port.
```javascript
const SDK = require("codechain-sdk");
const { Parcel, U256, H256, H160 } = SDK;

// Create SDK object with CodeChain RPC server URL
const sdk = new SDK("http://localhost:8080");
```
The parcel signer must pay the transaction fees. Parcels are basically a group of transactions used in CodeChain. They are the smallest unit that can be processed on the blockchain.

In order for the parcel to be valid, the nonce must match the nonce of the parcel signer. Once the parcel is confirmed, the nonce of the signer is increased by 1. When specifying the receiver, make sure the correct address is used for the recipient. In addition, the parcel must be signed with the secret key of the address.
```javascript
const parcelSignerNonce = new U256(0);
// Parcel signer pays 10 CCC as fee.
const fee = new U256(10);
// Network ID prevents replay attacks or confusion among different CodeChain networks.
const networkId = 17;
// Recipient of the payment
const receiver = new H160("744142069fe2d03d48e61734cbe564fcc94e6e31");
// Amount of the payment. The parcel signer's balance must be at least 10010.
const value = new U256(10000);
// Create the Parcel for the payment
const parcel = Parcel.payment(parcelSignerNonce, fee, networkId, receiver, value);

const parcelSignerSecret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const signedParcel = parcel.sign(parcelSignerSecret);
```
After signing the parcel, send the parcel off to the CodeChain node. The node is responsible for propagating the parcels properly. sendSignedParcel returns a promise that resolves with a parcel hash if the parcel has been verified and queued successfully. It doesn't mean that the parcel was confirmed, however. getParcel returns a promise that resolves with a parcel. Only confirmed parcels contain blockNumber/blockHash/parcelIndex fields. 
```javascript
sdk.sendSignedParcel(signedParcel).then((hash) => {
    console.log(`Parcel sent:`, hash);
    return sdk.getParcel(hash);
}).then((parcel) => {
    console.log(`Parcel`, parcel);
}).catch((err) => {
    console.error(`Error:`, err);
});

```
To view entire example, click [here](https://github.com/CodeChain-io/codechain-sdk-js/blob/gh-pages/examples/payment.js).

## Mint 10000 Gold and send 3000 Gold

This example involves creating new assets and sending them amongst users. It largely involves three steps. First, create key pairs for each users. Then create the asset(in this case, Gold). Finally, execute the transaction.

```javascript
const SDK = require(".");
const { H256, privateKeyToAddress, H160, Parcel, U256,
    AssetScheme, PubkeyAssetAgent, MemoryKeyStore } = SDK;

const sdk = new SDK("http://localhost:8080");
```
We create new instances of a keyStore and an assetAgent. keyStore is where all the public and private keys are managed. 
```javascript
const keyStore = new MemoryKeyStore();
const assetAgent = new PubkeyAssetAgent({ keyStore });
```
In this example, it is assumed that there is something that created a parcel out of the transactions. sendTransaction has been declared for later use.

```javascript
// sendTransaction() is a function to make transaction to be processed.
async function sendTransaction(tx) {
    const parcelSignerSecret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
    const parcelSignerAddress = new H160(privateKeyToAddress("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const parcelSignerNonce = await sdk.getNonce(parcelSignerAddress);
    const parcel = Parcel.transactions(parcelSignerNonce, new U256(10), 17, tx);
    const signedParcel = parcel.sign(parcelSignerSecret);
    return await sdk.sendSignedParcel(signedParcel);
}
```
Each users need an address for them to receive/send assets to. Addresses are created by the assetAgent.
```javascript
    const aliceAddress = await assetAgent.createAddress();
    const bobAddress = await assetAgent.createAddress();
```
In this example, we want to create an asset called "Gold". Thus, we define a new asset scheme for the asset that will be named Gold. In schemes, the amount to be minted, and the registrar, if any, should be defined. If there is no registrar, it means that AssetTransfer of Gold can be done through any parcel. If the registrar is present, the parcel must be signed by the registrar. In this example, the registrar is set to null.

```javascript
 const goldAssetScheme = new AssetScheme({
        metadata: JSON.stringify({
            name: "Gold",
            imageUrl: "https://gold.image/",
        }),
        amount: 10000,
        registrar: null,
    });
```
After Gold has been defined in the scheme, the amount that is minted but belong to someone initially. In this example, we create 10000 gold for Alice.
```javascript
    const mintTx = goldAssetScheme.mint(aliceAddress);
```
Then, the AssetMintTransaction is processed with the following code:
```javascript
    // Process the AssetMintTransaction
    await sendTransaction(mintTx);

    // AssetMintTransaction creates Asset and AssetScheme object
    console.log("minted asset scheme: ", await sdk.getAssetScheme(mintTx.hash()));
    const firstGold = await sdk.getAsset(mintTx.hash(), 0);
    console.log("alice's gold: ", firstGold);
```
Alice then sends 3000 gold to Bob. In CodeChain, users must follow the [UTXO](https://codechain.readthedocs.io/en/latest/what-is-codechain.html#what-is-utxo) standard, and make a transaction that spends an entire UTXO balance, and receive the change back through another transaction.
```javascript
    // Spend Alice's 10000 golds. In this case, Alice pays 3000 golds to Bob. Alice
    // is paid the remains back.
    // The sum of amount must equal to the amount of firstGold.
    const transferTx = await firstGold.transfer(assetAgent, [{
        address: bobAddress,
        amount: 3000
    }, {
        address: aliceAddress,
        amount: 7000
    }]);
```
By using Alice's signature, the 10000 Gold that was first minted can now be transferred to other users like Bob.
```javascript
    await sendTransaction(transferTx);

    // Spent asset will be null
    console.log(await sdk.getAsset(mintTx.hash(), 0));

    // Unspent Bob's 3000 golds
    console.log(await sdk.getAsset(transferTx.hash(), 0));
    // Unspent Alice's 7000 golds
    console.log(await sdk.getAsset(transferTx.hash(), 1));
```
The entire example can be viewed [here](https://github.com/CodeChain-io/codechain-sdk-js/blob/gh-pages/examples/mint-and-transfer.js).

# [SDK](classes/sdk.html) methods
 * [getAsset](classes/sdk.html#getasset)
 * [getAssetScheme](classes/sdk.html#getassetscheme)
 * [getBalance](classes/sdk.html#getbalance)
 * [getBlock](classes/sdk.html#getblock)
 * [getBlockHash](classes/sdk.html#getblockhash)
 * [getBestBlockNumber](classes/sdk.html#getbestblocknumber)
 * [getNonce](classes/sdk.html#getnonce)
 * [getParcel](classes/sdk.html#getparcel)
 * [getParcelInvoices](classes/sdk.html#getparcelinvoices)
 * [getPendingParcels](classes/sdk.html#getpendingparcels)
 * [getRegularKey](classes/sdk.html#getregularkey)
 * [getTransactionInvoice](classes/sdk.html#gettransactioninvoice)
 * [ping](classes/sdk.html#ping)
 * [sendSignedParcel](classes/sdk.html#sendsignedparcel)

# Transactions
 * [PaymentTransaction](classes/paymenttransaction.html)
 * [SetRegularKeyTransaction](classes/setregularkeytransaction.html)
 * [AssetMintTransaction](classes/assetminttransaction.html)
 * [AssetTransferTransaction](classes/assettransfertransaction.html)
