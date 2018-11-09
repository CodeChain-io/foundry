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
    const amount = 100;

    const balanceStart = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);

    // Wrap 100 CCC into the wrapped CCC asset type.
    const wrapCCCParcel = sdk.core.createWrapCCCParcel({
        shardId: 0,
        recipient: address,
        amount
    });
    const wrapCCCSignedParcelHash = await sdk.rpc.chain.sendParcel(
        wrapCCCParcel,
        {
            account: ACCOUNT_ADDRESS,
            passphrase: ACCOUNT_PASSPHRASE
        }
    );
    const wrapCCCParcelInvoice = await sdk.rpc.chain.getParcelInvoice(
        wrapCCCSignedParcelHash,
        {
            // Wait up to 120 seconds to get the invoice.
            timeout: 120 * 1000
        }
    );
    if (!wrapCCCParcelInvoice.success) {
        throw Error(
            `WrapCCC failed: ${JSON.stringify(wrapCCCParcelInvoice.error)}`
        );
    }
    const balanceAfterWrapCCC = await sdk.rpc.chain.getBalance(ACCOUNT_ADDRESS);
    console.log("Wrap finish");

    // Unwrap the wrapped CCC asset created before.
    const unwrapCCCTx = sdk.core.createAssetUnwrapCCCTransaction({
        burn: wrapCCCParcel.getAsset() // After sendParcel, the fee and seq field of wrapCCCParcel is filled.
    });
    await sdk.key.signTransactionBurn(unwrapCCCTx, 0);
    const unwrapCCCParcel = sdk.core.createAssetTransactionParcel({
        transaction: unwrapCCCTx
    });
    await sdk.rpc.chain.sendParcel(unwrapCCCParcel, {
        account: ACCOUNT_ADDRESS,
        passphrase: ACCOUNT_PASSPHRASE
    });
    const unwrapCCCTxInvoices = await sdk.rpc.chain.getTransactionInvoices(
        unwrapCCCTx.hash(),
        {
            // Wait up to 120 seconds to get the invoice.
            timeout: 120 * 1000
        }
    );
    if (!unwrapCCCTxInvoices[0].success) {
        throw Error(
            `AssetUnwrapCCCTransaction failed: ${JSON.stringify(
                unwrapCCCTxInvoices[0].error
            )}`
        );
    }
    const balanceAfterUnwrapCCC = await sdk.rpc.chain.getBalance(
        ACCOUNT_ADDRESS
    );

    console.log(balanceStart.toString());
    console.log(balanceAfterWrapCCC.toString());
    console.log(balanceAfterUnwrapCCC.toString());
})().catch(console.error);
