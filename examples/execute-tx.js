const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";

(async () => {
    const shardId = 0;
    const aliceAddress = await sdk.key.createAssetTransferAddress();
    const seq = await sdk.rpc.chain.getSeq(ACCOUNT_ADDRESS);

    const goldAssetScheme = sdk.core.createAssetScheme({
        shardId,
        metadata: {
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/"
        },
        supply: 10000,
        administrator: ACCOUNT_ADDRESS
    });
    const mintTx = sdk.core.createMintAssetTransaction({
        scheme: goldAssetScheme,
        recipient: aliceAddress
    });

    await mintTx.setFee(10);
    await mintTx.setSeq(seq);

    const result = await sdk.rpc.chain.executeTransaction(
        mintTx,
        ACCOUNT_ADDRESS
    );
    console.log(result); // true
})().catch(err => {
    console.error(`Error:`, err);
});
