# Install package

`npm install codechain-sdk` or `yarn add codechain-sdk`

# Usage example
Make sure that your CodeChain RPC server is listening. In the examples, we assume it is localhost:8080

## Send 10000 CCC From One Party To Another
This example involves sending CCC from one party to another.
First, make sure to import the correct sdk and use the proper server port.
```javascript
var SDK = require("codechain-sdk");
var sdk = new SDK({ server: "http://localhost:8080" });

var signerAddress = "0xa6594b7196808d161b6fb137e781abbc251385d9";
var recipientAddress = "0x744142069fe2d03d48e61734cbe564fcc94e6e31";
```
The parcel signer must pay the transaction fees. Parcels are basically a group of transactions used in CodeChain. They are the smallest unit that can be processed on the blockchain.

In order for the parcel to be valid, the nonce must match the nonce of the parcel signer. Once the parcel is confirmed, the nonce of the signer is increased by 1. When specifying the receiver, make sure the correct address is used for the recipient. In addition, the parcel must be signed with the secret key of the address. After signing the parcel, send the parcel off to the CodeChain node. The node is responsible for propagating the parcels properly.
```javascript
sdk.rpc.chain.getNonce(signerAddress).then(function (nonce) {
    var parcel = sdk.core.createPaymentParcel({
        recipient: recipientAddress,
        amount: 10000,
        nonce,
        // Parcel signer pays 10 CCC as fee.
        fee: 10
    });
    var signedParcel = parcel.sign(signerSecret);
    return sdk.rpc.chain.sendSignedParcel(signedParcel);
})
```
sendSignedParcel returns a promise that resolves with a parcel hash if the parcel has been verified and queued successfully. It doesn't mean that the parcel was confirmed, however. getParcel returns a promise that resolves with a parcel. Only confirmed parcels contain blockNumber/blockHash/parcelIndex fields.
```javascript
.then(function (parcelHash) {
    // sendSignedParcel returns a promise that resolves with a parcel hash if parcel has
    // been verified and queued successfully. It doesn't mean parcel was confirmed.
    console.log(`Parcel sent:`, parcelHash);
    return sdk.rpc.chain.getParcel(parcelHash);
}).then(function (parcel) {
    // getParcel returns a promise that resolves with a parcel.
    // blockNumber/blockHash/parcelIndex fields in Parcel is present only for the
    // confirmed parcel
    console.log(`Parcel`, parcel);
}).catch((err) => {
    console.error(`Error:`, err);
});
```
To view entire example, click [here](https://github.com/CodeChain-io/codechain-sdk-js/blob/master/examples/payment.js).

## Mint 10000 Gold and send 3000 Gold

This example involves creating new assets and sending them amongst users. It largely involves three steps. First, create key pairs for each users. Then create the asset(in this case, Gold). Finally, execute the transaction.

```javascript
const SDK = require("codechain-sdk");

const sdk = new SDK({ server: "http://localhost:8080" });
```
We create new instances of an assetAgent. AssetAgent creates addresses for assets and manages their locking/unlocking data. 
```javascript
const assetAgent = sdk.core.getAssetAgent();
```
In this example, it is assumed that there is something that created a parcel out of the transactions. sendTransaction has been declared for later use.

```javascript
// sendTransaction() is a function to make transaction to be processed.
async function sendTransaction(tx) {
    const parcelSignerSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
    const parcelSignerAddress = SDK.util.getAccountIdFromPrivate(parcelSignerSecret);
    const parcel = sdk.core.createChangeShardStateParcel({
        transactions: [tx],
        nonce: await sdk.rpc.chain.getNonce(parcelSignerAddress),
        fee: 10,
    }).sign(parcelSignerSecret)
    return await sdk.rpc.chain.sendSignedParcel(parcel);
}
```
Each users need an address for them to receive/send assets to. Addresses are created by the assetAgent.
```javascript
// Start of wrapping async function, we use async/await here because a lot of
// Promises are there.
(async () => {
    const aliceAddress = await assetAgent.createAddress();
    const bobAddress = await assetAgent.createAddress();
```
In this example, we want to create an asset called "Gold". Thus, we define a new asset scheme for the asset that will be named Gold. In schemes, the amount to be minted, and the registrar, if any, should be defined. If there is no registrar, it means that AssetTransfer of Gold can be done through any parcel. If the registrar is present, the parcel must be signed by the registrar. In this example, the registrar is set to null.

```javascript
    const goldAssetScheme = sdk.core.createAssetScheme({
        metadata: JSON.stringify({
            name: "Gold",
            imageUrl: "https://gold.image/",
        }),
        amount: 10000,
        registrar: null,
    })
```
After Gold has been defined in the scheme, the amount that is minted but belong to someone initially. In this example, we create 10000 gold for Alice.
```javascript
    const mintTx = goldAssetScheme.mint(aliceAddress);
```
Then, the AssetMintTransaction is processed with the following code:
```javascript
    await sendTransaction(mintTx);
    // Wait up to 5 minutes for transaction processing
    const mintTxInvoice = await sdk.rpc.chain.getTransactionInvoice(mintTx.hash(), 5 * 60 * 1000);
    if (!mintTxInvoice.success) {
        throw "AssetMintTransaction failed";
    }
    const firstGold = await sdk.rpc.chain.getAsset(mintTx.hash(), 0);
```
Alice then sends 3000 gold to Bob. In CodeChain, users must follow the [UTXO](https://codechain.readthedocs.io/en/latest/what-is-codechain.html#what-is-utxo) standard, and make a transaction that spends an entire UTXO balance, and receive the change back through another transaction.
```javascript
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
    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(transferTx.hash(), 5 * 60 * 1000);
    if (!transferTxInvoice.success) {
        throw "AssetTransferTransaction failed";
    }

    // Spent asset will be null
    console.log(await sdk.getAsset(mintTx.hash(), 0));

    // Unspent Bob's 3000 golds
    console.log(await sdk.getAsset(transferTx.hash(), 0));
    // Unspent Alice's 7000 golds
    console.log(await sdk.getAsset(transferTx.hash(), 1));
// End of wrapping async function
})();
```
The entire example can be viewed [here](https://github.com/CodeChain-io/codechain-sdk-js/blob/master/examples/mint-and-transfer.js).

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
