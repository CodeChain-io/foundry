const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";

const masterSecret = ACCOUNT_SECRET;
const masterAccountId = SDK.util.getAccountIdFromPrivate(masterSecret);
const masterAddress = sdk.core.classes.PlatformAddress.fromAccountId(
    masterAccountId
);

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

(async () => {
    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    // Set `regularSecret` as the master account's regular key.
    // It means that you can sign a parcel with the "regularSecret" instead of the "masterSecert".
    const setRegularKeyParcel = sdk.core.createSetRegularKeyParcel({
        key: regularPublic
    });
    const setRegularKeyParcelHash = await sdk.rpc.chain.sendSignedParcel(
        setRegularKeyParcel.sign({
            secret: masterSecret,
            seq,
            fee: 10
        })
    );

    await sdk.rpc.chain.getParcelInvoice(setRegularKeyParcelHash, {
        timeout: 60 * 60 * 1000
    });
    console.log("The parcel contains 'setRegularkey' has been mined");

    const beforeBalance = await sdk.rpc.chain.getBalance(masterAddress);
    console.log(`Current master account's balance is ${beforeBalance}`);

    const seq2 = await sdk.rpc.chain.getSeq(masterAddress);
    const p2 = sdk.core.createPaymentParcel({
        recipient: masterAddress,
        amount: 10
    });
    // We can sign a parcel with our `regularSecret`.
    // The parcel's fee is charged from the master account.
    const hash2 = await sdk.rpc.chain.sendSignedParcel(
        p2.sign({
            secret: regularSecret,
            seq: seq2,
            fee: 10
        })
    );
    await sdk.rpc.chain.getParcelInvoice(hash2, {
        timeout: 60 * 60 * 1000
    });
    console.log("The parcel signed with 'regularSecret' has been mined");

    const afterBalance = await sdk.rpc.chain.getBalance(masterAddress);
    console.log(
        `After the parcel which signed with regularSecret, master account's balance changed to ${afterBalance}`
    );
})().catch(err => {
    console.error(`Error:`, err);
});
