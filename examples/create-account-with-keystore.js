const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

// createRemoteKeyStore("http://localhost:7007") is also available.
// If you want to know how to set up the external key store, go to
// https://codechain.readthedocs.io/en/latest/asset-management.html#use-remotekeystore-to-save-asset-address-private-key
(async () => {
    const keyStore = await sdk.key.createLocalKeyStore();
    const address = await sdk.key.createPlatformAddress({
        keyStore
    });
    console.log(address.toString());
}).catch(console.error);
