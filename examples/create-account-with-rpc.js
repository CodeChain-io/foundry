const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

(async () => {
    const address = await sdk.rpc.account.create("my-secret");
    console.log(address); // string that starts with "ccc"(mainnet) or "tcc"(testnet). For example: cccqzn9jjm3j6qg69smd7cn0eup4w7z2yu9myd6c4d7
})().catch(console.error);
