const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: "http://52.78.210.78:8080",
    networkId: "sc"
});

(async () => {
    // LocalKeyStore creates `keystore.db` file in the working directory.
    const keyStore = await sdk.key.createLocalKeyStore();
    const address = await sdk.key.createAssetTransferAddress({
        keyStore
    });
    // You can mint assets using this address.
    // Address example: scaqyqu9ll6mw0nt07sjuvk7w5wq8tzvpdureps9gnepm
    console.log(address.toString());
})().catch(console.error);
