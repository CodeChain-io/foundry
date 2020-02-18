const SDK = require("..");

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({
    server: SERVER_URL
});

const tx = sdk.core.createPayTransaction({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    quantity: 10000
});

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    const account = await sdk.key.createPlatformAddress({
        keyStore
    });
    const seq = await sdk.rpc.chain.getSeq(account);

    const signed = await sdk.key.signTransaction(tx, {
        account,
        keyStore,
        fee: 10,
        seq
    });
    console.log(signed);
    // FIXME: needs fee
    // const hash = await sdk.rpc.chain.sendSignedTransaction(tx);
})().catch(console.error);
