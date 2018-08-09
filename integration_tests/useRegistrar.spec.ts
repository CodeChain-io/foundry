import { SDK } from "../";

const SERVER_URL = process.env.CODECHAIN_RPC_HTTP || "http://localhost:8080";
const sdk = new SDK({ server: SERVER_URL });
const masterSecret = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd";
const masterAddress = SDK.util.getAccountIdFromPrivate(masterSecret);

const otherSecret = "0000000000000000000000000000000000000000000000000000000000000001";
const otherAddress = SDK.util.getAccountIdFromPrivate(otherSecret);

const regularSecret = SDK.util.generatePrivateKey();
const regularPublic = SDK.util.getPublicFromPrivate(regularSecret);

test("checkRegistrarValidation", async () => {
    await setRegularKey();
    await sendCCCToOther();

    const keyStore = await sdk.key.createMemoryKeyStore();
    const p2pkh = await sdk.key.createP2PKH({ keyStore });
    const aliceAddress = await p2pkh.createAddress();
    const bobAddress = "ccaqqqap7lazh5g84jsfxccp686jakdy0z9v4chrq4vz8pj4nl9lzvf7rs2rnmc0";

    const mintTx = await mintAssetUsingMaster(p2pkh, aliceAddress, bobAddress);
    await transferAssetUsingOther(mintTx, p2pkh, aliceAddress, bobAddress);
    await transferAssetUsingRegular(mintTx, p2pkh, aliceAddress, bobAddress);
});

async function setRegularKey() {
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    const p = sdk.core.createSetRegularKeyParcel({
        key: regularPublic,
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret: masterSecret,
        nonce,
        fee: 10
    }));

    await sdk.rpc.chain.getParcelInvoice(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function sendCCCToOther() {
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    const p = sdk.core.createPaymentParcel({
        recipient: otherAddress,
        amount: 100,
    });
    const hash = await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret: regularSecret,
        nonce,
        fee: 10
    }));

    await sdk.rpc.chain.getParcelInvoice(hash, {
        timeout: 5 * 60 * 1000
    });
}

async function mintAssetUsingMaster(p2pkh, aliceAddress, bobAddress) {
    const assetScheme = sdk.core.createAssetScheme({
        shardId: 0,
        metadata: JSON.stringify({
            name: "Gold",
            description: "An asset example",
            icon_url: "https://gold.image/",
        }),
        amount: 10000,
        registrar: masterAddress,
    });

    const mintTx = sdk.core.createAssetMintTransaction({
        scheme: assetScheme,
        recipient: aliceAddress
    });

    const p = sdk.core.createChangeShardStateParcel({
        transactions: [mintTx]
    });
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret: masterSecret,
        nonce,
        fee: 10
    }));

    const mintTxInvoice = await sdk.rpc.chain.getTransactionInvoice(mintTx.hash(), {
        timeout: 5 * 60 * 1000
    });

    expect(mintTxInvoice.success).toBe(true);
    return mintTx;
}

async function transferAssetUsingRegular(mintTx, p2pkh, aliceAddress, bobAddress) {
    const asset = mintTx.getMintedAsset();
    const transferTx = sdk.core.createAssetTransferTransaction()
        .addInputs(asset)
        .addOutputs(
            {
                recipient: bobAddress,
                amount: 3000,
                assetType: asset.assetType
            }, {
                recipient: aliceAddress,
                amount: 7000,
                assetType: asset.assetType
            });
    await transferTx.sign(0, { signer: p2pkh });
    transferTx.getTransferredAssets();

    const p = sdk.core.createChangeShardStateParcel({
        transactions: [transferTx]
    });
    const nonce = await sdk.rpc.chain.getNonce(masterAddress);
    await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret: regularSecret,
        nonce,
        fee: 10
    }));

    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(transferTx.hash(), {
        timeout: 5 * 60 * 1000
    });
    expect(transferTxInvoice.success).toBe(true);
}
async function transferAssetUsingOther(mintTx, p2pkh, aliceAddress, bobAddress) {
    const asset = mintTx.getMintedAsset();

    const transferTx = sdk.core.createAssetTransferTransaction(
        {
            burns: [],
            inputs: [],
            outputs: [],
            nonce: 1, // use nonce here because "transferAssetUsingRegular" will send the same transaction
        })
        .addInputs(asset)
        .addOutputs(
            {
                recipient: bobAddress,
                amount: 3000,
                assetType: asset.assetType
            }, {
                recipient: aliceAddress,
                amount: 7000,
                assetType: asset.assetType
            });

    await transferTx.sign(0, { signer: p2pkh });
    transferTx.getTransferredAssets();

    const p = sdk.core.createChangeShardStateParcel({
        transactions: [transferTx]
    });
    const nonce = await sdk.rpc.chain.getNonce(otherAddress);
    await sdk.rpc.chain.sendSignedParcel(p.sign({
        secret: otherSecret,
        nonce,
        fee: 10
    }));

    const transferTxInvoice = await sdk.rpc.chain.getTransactionInvoice(transferTx.hash(), {
        timeout: 5 * 60 * 1000
    });
    expect(transferTxInvoice.success).toBe(false);
}