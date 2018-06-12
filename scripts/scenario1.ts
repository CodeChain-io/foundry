import * as assert from "assert";
import SDK from "../src";

const { Parcel, H256, U256, H160, AssetTransferTransaction,
    AssetMintTransaction, PaymentTransaction, AssetOutPoint,
    AssetTransferInput, AssetTransferOutput, privateKeyToAddress } = SDK;

const secret = new H256("ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd");
const address = new H160(privateKeyToAddress(secret.value));
const networkId = 17;

const sdk = new SDK("http://localhost:8080");


const emptyLockScriptHash = new H256("50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3");

const mint1 = new AssetMintTransaction({
    metadata: "metadata of permissioned infinite asset",
    lockScriptHash: emptyLockScriptHash,
    parameters: [],
    amount: null,
    registrar: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    nonce: 0,
});

const mint2 = new AssetMintTransaction({
    metadata: "metadata of non-permissioned asset",
    lockScriptHash: emptyLockScriptHash,
    parameters: [],
    amount: 100,
    registrar: null,
    nonce: 0,
});


const transfer1 = (() => {
    const inputs = [ new AssetTransferInput({
        prevOut: new AssetOutPoint({
            transactionHash: mint2.hash(),
            index: 0,
            assetType: mint2.getAssetSchemeAddress(),
            amount: 100
        }),
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    })];
    const outputs = [new AssetTransferOutput({
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 93
    }), new AssetTransferOutput({
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 4
    }), new AssetTransferOutput({
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 2
    }), new AssetTransferOutput({
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 1
    })];
    return new AssetTransferTransaction(networkId, { inputs, outputs });
})();

const transfer2 = (() => {
    const inputs = [new AssetTransferInput({
        prevOut: new AssetOutPoint({
            transactionHash: transfer1.hash(),
            index: 1,
            assetType: mint2.getAssetSchemeAddress(),
            amount: 4
        }),
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }), new AssetTransferInput({
        prevOut: new AssetOutPoint({
            transactionHash: transfer1.hash(),
            index: 3,
            assetType: mint2.getAssetSchemeAddress(),
            amount: 1
        }),
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    })];
    const outputs = [new AssetTransferOutput({
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 5
    })];
    return new AssetTransferTransaction(networkId, {inputs, outputs});

})();

const payment2 = new PaymentTransaction({
    nonce: new U256(2),
    sender: address,
    receiver: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});
const payment3 = new PaymentTransaction({
    nonce: new U256(3),
    sender: address,
    receiver: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});
const payment5 = new PaymentTransaction({
    nonce: new U256(5),
    sender: address,
    receiver: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});
const payment7 = new PaymentTransaction({
    nonce: new U256(7),
    sender: address,
    receiver: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("1000000000000000")
});
const payment8 = new PaymentTransaction({
    nonce: new U256(8),
    sender: address,
    receiver: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});


sdk.getNonce(address).then(async (nonce) => {
    if (!nonce) {
        throw new Error("No nonce");
    }
    console.log(`Current nonce: ${nonce.value}`);
    await printTransactionInvoices();

    const parcel0 = new Parcel(new U256(0), new U256(10), networkId);
    await sendParcel(parcel0, new H256("06bedb2443dcb609597ea74d5dcee9b3bf35fc0ff7a5271ff144c7faa9d7ef69"));
    await printTransactionInvoices();

    const parcel1 = new Parcel(new U256(1), new U256(10), networkId, payment2, payment3);
    await sendParcel(parcel1, new H256("b21d26a6ff8dc326ba949d331d66893ffca05397241a65cb59cadb7d27530311"));
    await printTransactionInvoices();

    const parcel2 = new Parcel(new U256(4), new U256(10), networkId, payment5);
    await sendParcel(parcel2, new H256("b5c44620cd36b73029b847f1a338470c2dfb0f8782b7b34d04621f77c15bb311"));
    await printTransactionInvoices();

    const parcel3 = new Parcel(new U256(6), new U256(10), networkId, payment7, payment8);
    await sendParcel(parcel3, new H256("fe39587fa0153359db5944cc761d82ceb7951ce22c7dfc38e9c7564eb9eed074"));
    await printTransactionInvoices();

    const parcel4 = new Parcel(new U256(9), new U256(10), networkId, mint1, mint2);
    await sendParcel(parcel4, new H256("2ed039d66c1c76d271556489a2046ebf464570abe431120025b497c686ec773f"));
    await printResults();

    const parcel5 = new Parcel(new U256(10), new U256(10), networkId, transfer1);
    await sendParcel(parcel5, new H256("3bafc24fb3ccfbb0da99269a7b6517e1adc007b541e970efa1caad26e8fa8ec5"));
    await printResults();

    const parcel6 = new Parcel(new U256(11), new U256(10), networkId, transfer2);
    await sendParcel(parcel6, new H256("b04f0aecde43c1860d4b30904b531d93c7b424bb87b9221b6ba6a68059ee78b0"));
    await printResults();

    console.log("Succeed");
}).catch( err => {
    console.error(err);
});

async function printAsset(hash: H256, index: number) {
    const asset = await sdk.getAsset(hash, index);
    if (asset) {
        console.log(JSON.stringify(asset.toJSON()));
    } else {
        console.log("No asset");
    }
}

async function printAssetScheme(hash: H256) {
    const scheme = await sdk.getAssetScheme(hash);
    if (scheme) {
        console.log(JSON.stringify(scheme.toJSON()));
    } else {
        console.log("No scheme");
    }
}

async function printTransactionInvoice(prefix: string, hash: H256) {
    const invoice = await sdk.getTransactionInvoice(hash);
    if (invoice) {
        console.log(prefix, invoice);
    } else {
        console.log(prefix, "no invoice")
    }
}

async function printTransactionInvoices(): Promise<any> {
    console.log("=====================");
    await printTransactionInvoice("payment2", payment2.hash());
    await printTransactionInvoice("payment3", payment3.hash());
    await printTransactionInvoice("payment5", payment5.hash());
    await printTransactionInvoice("payment7", payment7.hash());
    await printTransactionInvoice("payment8", payment8.hash());
    await printTransactionInvoice("mint1", mint1.hash());
    await printTransactionInvoice("mint2", mint2.hash());
    await printTransactionInvoice("transfer1", transfer1.hash());
    await printTransactionInvoice("transfer2", transfer2.hash());
}

async function printResults(): Promise<any> {
    try {
        await printTransactionInvoices();
        await printAssetScheme(mint1.hash());
        await printAssetScheme(mint2.hash());
        await printAsset(mint1.hash(), 0);
        await printAsset(mint2.hash(), 0);
        await printAsset(transfer1.hash(), 0);
        await printAsset(transfer1.hash(), 1);
        await printAsset(transfer1.hash(), 2);
        await printAsset(transfer1.hash(), 3);
        await printAsset(transfer2.hash(), 0);
    } catch(err) {
        console.error(err);
    };
}

function sendParcel(parcel: Parcel, parcel_hash: H256): Promise<any> {
    return sdk.sendSignedParcel(parcel.sign(secret)).then(hash => {
        assert(hash.isEqualTo(parcel_hash), `${hash.toEncodeObject()} != ${parcel_hash.toEncodeObject()}`);
        return new Promise<H256>(resolver => {
            setTimeout(() => resolver(hash), 2000);
        });
    }).catch(err => {
        console.error(err);
        return parcel_hash;
    }).then(hash => {
        return sdk.getParcelInvoices(hash);
    });
}
