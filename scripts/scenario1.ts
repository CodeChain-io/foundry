import * as assert from "assert";
import { AssetTransferTransaction, AssetMintTransaction, PaymentTransaction } from "../src/primitives/transaction/";
import { Parcel, H256, U256, H160 } from "../src/primitives";
import { SDK } from "../src";
import { privateKeyToAddress } from "../src/utils";

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
    const inputs = [ {
        prevOut: {
            transactionHash: mint2.hash(),
            index: 0,
            assetType: mint2.getAssetSchemeAddress(),
            amount: 100
        },
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }];
    const outputs = [{
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 93
    }, {
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 4
    }, {
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 2
    }, {
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 1
    }];
    return new AssetTransferTransaction(networkId, { inputs, outputs });
})();

const transfer2 = (() => {
    const inputs = [{
        prevOut: {
            transactionHash: transfer1.hash(),
            index: 1,
            assetType: mint2.getAssetSchemeAddress(),
            amount: 4
        },
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }, {
        prevOut: {
            transactionHash: transfer1.hash(),
            index: 3,
            assetType: mint2.getAssetSchemeAddress(),
            amount: 1
        },
        lockScript: Buffer.from([0x2, 0x1]),
        unlockScript: Buffer.from([])
    }];
    const outputs = [{
        lockScriptHash: emptyLockScriptHash,
        parameters: [],
        assetType: mint2.getAssetSchemeAddress(),
        amount: 5
    }];
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


sdk.getNonce(address).then(nonce => {
    if (!nonce) {
        throw new Error("No nonce");
    }
    console.log(`Current nonce: ${nonce.value}`);
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(0), new U256(10), networkId);
    return sendParcel(p, new H256("06bedb2443dcb609597ea74d5dcee9b3bf35fc0ff7a5271ff144c7faa9d7ef69"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(1), new U256(10), networkId, payment2, payment3);
    return sendParcel(p, new H256("b21d26a6ff8dc326ba949d331d66893ffca05397241a65cb59cadb7d27530311"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(4), new U256(10), networkId, payment5);
    return sendParcel(p, new H256("b5c44620cd36b73029b847f1a338470c2dfb0f8782b7b34d04621f77c15bb311"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(6), new U256(10), networkId, payment7, payment8);
    return sendParcel(p, new H256("fe39587fa0153359db5944cc761d82ceb7951ce22c7dfc38e9c7564eb9eed074"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(9), new U256(10), networkId, mint1, mint2);
    return sendParcel(p, new H256("2ed039d66c1c76d271556489a2046ebf464570abe431120025b497c686ec773f"));
}).then(printResults).then( () => {
}).then( () => {
    const p = new Parcel(new U256(10), new U256(10), networkId, transfer1);
    return sendParcel(p, new H256("3bafc24fb3ccfbb0da99269a7b6517e1adc007b541e970efa1caad26e8fa8ec5"));
}).then(printResults).then( () => {
}).then( () => {
    const p = new Parcel(new U256(11), new U256(10), networkId, transfer2);
    return sendParcel(p, new H256("b04f0aecde43c1860d4b30904b531d93c7b424bb87b9221b6ba6a68059ee78b0"));
}).then(printResults).then( () => {
    console.log("Succeed");
}).catch( err => {
    console.error(err);
});

function printAsset(hash: H256, index: number) {
    return sdk.getAsset(hash, index).then(asset => {
        if (asset) {
            console.log(JSON.stringify(asset.toJSON()));
        } else {
            console.log("No asset");
        }
    });
}
function printAssetScheme(hash: H256) {
    return sdk.getAssetScheme(hash).then(scheme => {
        if (scheme) {
            console.log(JSON.stringify(scheme.toJSON()));
        } else {
            console.log("No scheme");
        }
    });
}
function printTransactionInvoice(prefix: string, hash: H256) {
    return sdk.getTransactionInvoice(hash).then(invoice => {
        if (invoice) {
            console.log(prefix, invoice);
        } else {
            console.log(prefix, "no invoice")
        }
    });

}

function printTransactionInvoices(): Promise<any> {
    console.log("=====================");
    return printTransactionInvoice("payment2", payment2.hash())
        .then(() => printTransactionInvoice("payment3", payment3.hash()))
        .then(() => printTransactionInvoice("payment5", payment5.hash()))
        .then(() => printTransactionInvoice("payment7", payment7.hash()))
        .then(() => printTransactionInvoice("payment8", payment8.hash()))
        .then(() => printTransactionInvoice("mint1", mint1.hash()))
        .then(() => printTransactionInvoice("mint2", mint2.hash()))
        .then(() => printTransactionInvoice("transfer1", transfer1.hash()))
        .then(() => printTransactionInvoice("transfer2", transfer2.hash()));
}
function printResults(): Promise<any> {
    return printTransactionInvoices()
        .then(() => printAssetScheme(mint1.hash()))
        .then(() => printAssetScheme(mint2.hash()))
        .then(() => printAsset(mint1.hash(), 0))
        .then(() => printAsset(mint2.hash(), 0))
        .then(() => printAsset(transfer1.hash(), 0))
        .then(() => printAsset(transfer1.hash(), 1))
        .then(() => printAsset(transfer1.hash(), 2))
        .then(() => printAsset(transfer1.hash(), 3))
        .then(() => printAsset(transfer2.hash(), 0))
        .catch(err => {
            console.error(err);
        });
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
