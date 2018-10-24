const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: "http://52.78.210.78:8080",
    networkId: "sc"
});

(async () => {
    // LocalKeyStore creates `keystore.db` file in the working directory.
    const keyStore = await sdk.key.createLocalKeyStore();
    const address = await sdk.key.createPlatformAddress({
        keyStore
    });
    // Visit https://saluki.codechain.io/faucet and get CCC for your account.
    console.log(address.toString());
})().catch(console.error);
