# Install package

`npm install codechain-sdk` or `yarn add codechain-sdk`

# Usage example
Make sure that your CodeChain RPC server is listening. In the examples, we assume it is localhost:8080

## Send 10000 CCC using PaymentTransaction

```javascript
import { SDK, Parcel, U256, H256, H160, PaymentTransaction } from "codechain-sdk";

// Create SDK object with CodeChain RPC server URL
const sdk = new SDK("http://localhost:8080");

// Create PaymentTransaction that sends 10000 CCC from 0x5bcd7c... to 0x744142..
// Transaction is only valid if the nonce matches the nonce of the sender.
// The nonce of the sender is increased by 1 when this transaction is confirmed.
const tx = new PaymentTransaction({
    nonce: new U256(0),
    sender: new H160("5bcd7c840f108172d94a4d084af711d879630fe6"),
    receiver: new H160("744142069fe2d03d48e61734cbe564fcc94e6e31"),
    value: new U256(10000)
});

// Parcel is only valid if the nonce matches the nonce of the parcel signer.
// The nonce of the signer is increased by 1 when this parcel is confirmed.
const parcelSignerNonce = new U256(0);
// Parcel signer pays 10 CCC as fee.
const fee = new U256(10);
// Network ID prevents replay attacks or confusion among different CodeChain networks.
const networkId = 17;
// Create Parcel
const parcel = new Parcel(parcelSignerNonce, fee, networkId, tx);

// Sign the parcel with the secret key of the address 0x31bd8354de8f7dbab6764a11851086061fee3f25.
const parcelSignerSecret = new H256("b15139f97aad25ae0330432aeb091ef962eee643e41dc07a1e04457c5c2c6088");
const signedParcel = parcel.sign(parcelSignerSecret);

// Send the signed parcel to the CodeChain node. The node will propagate this
// parcel and attempt to confirm it.
sdk.sendSignedParcel(signedParcel).then((hash) => {
    // sendSignedParcel returns a promise that resolves with a parcel hash if parcel has
    // been verified and queued successfully. It doesn't mean parcel was confirmed.
    console.log(`Parcel sent:`, hash);
    return sdk.getParcel(hash);
}).then((parcel) => {
    // getParcel returns a promise that resolves with a parcel.
    // blockNumber/blockHash/parcelIndex fields in Parcel is present only for the
    // confirmed parcel
    console.log(`Parcel`, parcel);
}).catch((err) => {
    console.error(`Error:`, err);
});

```

## Mint 10000 Gold and send 3000 Gold using AssetMintTransaction, AssetTransferTransaction

```javascript
import { SDK, AssetMintTransaction, H256, blake256, signEcdsa, privateKeyToPublic,
    privateKeyToAddress, H160, Parcel, U256, AssetTransferTransaction,
    AssetTransferInput, AssetOutPoint, AssetTransferOutput, Transaction } from "codechain-sdk";

const sdk = new SDK("http://localhost:8080");

// CodeChain opcodes for P2PK(Pay to Public Key)
const OP_PUSHB = 0x32;
const OP_CHECKSIG = 0x81;

// Alice's key pair
const alicePrivate = "37a948d2e9ae622f3b9e224657249259312ffd3f2d105eabda6f222074608df3";
const alicePublic = privateKeyToPublic(alicePrivate);
// Alice's P2PK script
const aliceLockScript = Buffer.from([OP_PUSHB, 64, ...Buffer.from(alicePublic, "hex"), OP_CHECKSIG]);

// Bob's key pair
const bobPrivate = "f9387b3247c21e88c656490914f4598a3b52b807517753b4a9d7a51d54a6260c";
const bobPublic = privateKeyToPublic(bobPrivate);
// Bob's P2PK script
const bobLockScript = Buffer.from([OP_PUSHB, 64, ...Buffer.from(bobPublic, "hex"), OP_CHECKSIG]);

// sendTransaction() is a function to make transaction to be processed.
async function sendTransaction(t: Transaction) {
    const parcelSignerSecret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
    const parcelSignerAddress = new H160(privateKeyToAddress("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd"));
    const parcelSignerNonce = await sdk.getNonce(parcelSignerAddress);
    const parcel = new Parcel(parcelSignerNonce!, new U256(10), 17, t);
    const signedParcel = parcel.sign(parcelSignerSecret);
    return await sdk.sendSignedParcel(signedParcel);
}

(async () => {
    // Create AssetMintTransaction that creates 10000 amount of asset named Gold for Alice.
    const mintGoldTx = new AssetMintTransaction({
        // Put name, description and imageUrl to metadata.
        metadata: JSON.stringify({
            name: "Gold",
            imageUrl: "https://gold.image/"
        }),
        // hash value of locking script of the asset
        lockScriptHash: new H256(blake256(aliceLockScript)),
        parameters: [],
        // Mints 10000 golds
        amount: 10000,
        // No registrar for Gold. It means AssetTransfer of Gold can be done with any
        // parcel. If registrar is present, the parcel must be signed with the
        // registrar.
        registrar: null,
        nonce: 0
    });

    // Process the AssetMintTransaction
    await sendTransaction(mintGoldTx);

    // AssetMintTransaction creates Asset and AssetScheme object
    console.log(await sdk.getAsset(mintGoldTx.hash(), 0));
    console.log(await sdk.getAssetScheme(mintGoldTx.hash()));

    // The address of asset
    const goldAssetType = mintGoldTx.getAssetSchemeAddress();

    // Create an input that spends 10000 golds
    const inputs = [new AssetTransferInput({
        prevOut: new AssetOutPoint({
            transactionHash: mintGoldTx.hash(),
            index: 0,
            assetType: goldAssetType,
            amount: 10000
        }),
        // Provide the preimage of the lockScriptHash.
        lockScript: aliceLockScript,
        // unlockScript can't be calculated at this moment.
        unlockScript: Buffer.from([])
    })];

    // Create outputs. The sum of amount must equals to 10000. In this case, Alice
    // pays 3000 golds to Bob. Alice is paid the remains back.
    const outputs = [new AssetTransferOutput({
        lockScriptHash: new H256(blake256(bobLockScript)),
        parameters: [],
        assetType: goldAssetType,
        amount: 3000
    }), new AssetTransferOutput({
        lockScriptHash: new H256(blake256(aliceLockScript)),
        parameters: [],
        assetType: goldAssetType,
        amount: 7000
    })];

    // Create AssetTransferTransaction with the input and the outputs
    const transferTx = new AssetTransferTransaction(17, {
        burns: [],
        inputs,
        outputs,
    });

    // Calculate Alice's signature for the transaction.
    const { r, s, v } = signEcdsa(transferTx.hashWithoutScript().value, alicePrivate);
    const aliceSigBuffer = new Buffer(65);
    aliceSigBuffer.write(r.padStart(64, "0"), 0, 32, "hex");
    aliceSigBuffer.write(s.padStart(64, "0"), 32, 32, "hex");
    aliceSigBuffer.write(v.toString(16).padStart(2, "0"), 64, 1, "hex");

    // Create unlock script for the input of the transaction
    const aliceUnlockScript = Buffer.from([OP_PUSHB, 65, ...aliceSigBuffer]);
    // Put unlock script to the transaction
    transferTx.setUnlockScript(0, aliceUnlockScript);

    // Process the AssetTransferTransaction
    await sendTransaction(transferTx);

    // Spent asset will be null
    console.log(await sdk.getAsset(mintGoldTx.hash(), 0));

    // Unspent Bob's 3000 golds
    console.log(await sdk.getAsset(transferTx.hash(), 0));
    // Unspent Alice's 7000 golds
    console.log(await sdk.getAsset(transferTx.hash(), 1));
})();
```

# [SDK](classes/sdk.html) methods
 * [getAsset](classes/sdk.html#getasset)
 * [getAssetScheme](classes/sdk.html#getassetscheme)
 * [getBalance](classes/sdk.html#getbalance)
 * [getBlock](classes/sdk.html#getblock)
 * [getBlockHash](classes/sdk.html#getblockhash)
 * [getBlockNumber](classes/sdk.html#getblocknumber)
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
