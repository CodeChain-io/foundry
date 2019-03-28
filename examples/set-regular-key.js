const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_SECRET =
    process.env.ACCOUNT_SECRET ||
    "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
const ACCOUNT_ADDRESS = sdk.core.classes.PlatformAddress.fromAccountId(
    sdk.util.getAccountIdFromPrivate(ACCOUNT_SECRET),
    { networkId: "tc" }
);

const masterSecret = sdk.util.generatePrivateKey();
const masterAccountId = SDK.util.getAccountIdFromPrivate(masterSecret);
const masterAddress = sdk.core.classes.PlatformAddress.fromAccountId(
    masterAccountId,
    { networkId: "tc" }
);

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

(async () => {
    const pay = sdk.core
        .createPayTransaction({
            recipient: masterAddress,
            quantity: 1000
        })
        .sign({
            secret: ACCOUNT_SECRET,
            seq: await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS),
            fee: 10
        });
    await sdk.rpc.chain.sendSignedTransaction(pay);

    const seq = await sdk.rpc.chain.getSeq(masterAddress);
    // Set `regularSecret` as the master account's regular key.
    // It means that you can sign a tx with the "regularSecret" instead of the "masterSecert".
    const setRegularKey = sdk.core.createSetRegularKeyTransaction({
        key: regularPublic
    });
    const hash = await sdk.rpc.chain.sendSignedTransaction(
        setRegularKey.sign({
            secret: masterSecret,
            seq,
            fee: 10
        })
    );

    while (!(await sdk.rpc.chain.containTransaction(hash))) {
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    console.log("The tx contains 'setRegularkey' has been mined");

    const beforeBalance = await sdk.rpc.chain.getBalance(masterAddress);
    console.log(`Current master account's balance is ${beforeBalance}`);

    const seq2 = await sdk.rpc.chain.getSeq(masterAddress);
    const p2 = sdk.core.createPayTransaction({
        recipient: masterAddress,
        quantity: 10
    });
    // We can sign a tx with our `regularSecret`.
    // The tx's fee is charged from the master account.
    const hash2 = await sdk.rpc.chain.sendSignedTransaction(
        p2.sign({
            secret: regularSecret,
            seq: seq2,
            fee: 10
        })
    );
    while (!(await sdk.rpc.chain.containTransaction(hash2))) {
        await new Promise(resolve => setTimeout(resolve, 100));
    }
    console.log("The tx signed with 'regularSecret' has been mined");

    const afterBalance = await sdk.rpc.chain.getBalance(masterAddress);
    console.log(
        `After the transaction which signed with regularSecret, master account's balance changed to ${afterBalance}`
    );
})().catch(err => {
    console.error(`Error:`, err);
});
