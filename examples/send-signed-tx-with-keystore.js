const SDK = require("..");

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({
    server: SERVER_URL
});

const tx = sdk.core.createPayTransaction({
    recipient: "tccqxv9y4cw0jwphhu65tn4605wadyd2sxu5yezqghw",
    amount: 10000
});

(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    const account = await sdk.key.createPlatformAddress({
        keyStore
    });
    const seq = await sdk.rpc.chain.getSeq(account);

    const tx = await sdk.key.signTransaction(tx, {
        account,
        keyStore,
        fee: 10,
        seq
    });
    console.log(tx);
    // FIXME: needs fee
    // const hash = await sdk.rpc.chain.sendSignedTransaction(tx);
})();
