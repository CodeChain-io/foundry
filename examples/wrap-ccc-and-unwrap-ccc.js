const SDK = require("codechain-sdk");

const sdk = new SDK({
    server: process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080",
    networkId: process.env.CODECHAIN_NETWORK_ID || "tc"
});

const ACCOUNT_ADDRESS =
    process.env.ACCOUNT_ADDRESS ||
    "tccq9h7vnl68frvqapzv3tujrxtxtwqdnxw6yamrrgd";
const ACCOUNT_PASSPHRASE = process.env.ACCOUNT_PASSPHRASE || "satoshi";

(async () => {
    const address = await sdk.key.createAssetTransferAddress({
        type: "P2PKHBurn"
    });
    const quantity = 100;

    const balanceStart = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);

    // Wrap 100 CCC into the wrapped CCC asset type.
    const wrapCCC = sdk.core.createWrapCCCTransaction({
        shardId: 0,
        recipient: address,
        quantity,
        payer: ACCOUNT_ADDRESS
    });
    const wrapCCCSignedHash = await sdk.rpc.chain.sendTransaction(wrapCCC, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const wrapCCCResult = await sdk.rpc.chain.containTransaction(
        wrapCCCSignedHash
    );
    if (!wrapCCCResult) {
        throw Error("WrapCCC failed");
    }
    const balanceAfterWrapCCC = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);
    console.log("Wrap finish");

    // Unwrap the wrapped CCC asset created before.
    const unwrapCCCTx = sdk.core.createUnwrapCCCTransaction({
        burn: wrapCCC.getAsset(), // After sendTransaction, the fee and seq field of wrapCCC is filled.
        receiver: ACCOUNT_ADDRESS
    });
    await sdk.key.signTransactionBurn(unwrapCCCTx, 0);
    const hash = await sdk.rpc.chain.sendTransaction(unwrapCCCTx, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const unwrapCCCTxResult = await sdk.rpc.chain.containTransaction(hash);
    if (!unwrapCCCTxResult) {
        throw Error("AssetUnwrapCCCTransaction failed");
    }
    const balanceAfterUnwrapCCC = await sdk.rpc.chain.getBalance(
        ACCOUNT_ADDRESS
    );

    console.log(balanceStart.toString());
    console.log(balanceAfterWrapCCC.toString());
    console.log(balanceAfterUnwrapCCC.toString());
})().catch(console.error);
