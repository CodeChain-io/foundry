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
    address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});
const payment3 = new PaymentTransaction({
    nonce: new U256(3),
    address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});
const payment5 = new PaymentTransaction({
    nonce: new U256(5),
    address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});
const payment7 = new PaymentTransaction({
    nonce: new U256(7),
    address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("1000000000000000")
});
const payment8 = new PaymentTransaction({
    nonce: new U256(8),
    address: new H160("3f4aa1fedf1f54eeb03b759deadb36676b184911"),
    value: new U256("0")
});


sdk.getNonce(address).then(nonce => {
    if (!nonce) {
        throw new Error("No nonce");
    }
    console.log(`Current nonce: ${nonce.value}`);
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(0), new U256(10), networkId);
    return sendParcel(p, new H256("208cbe076e68fda488bcba6f2884c3eedcb5917163a259d32010598307213d22"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(1), new U256(10), networkId, payment2, payment3);
    return sendParcel(p, new H256("bc6d1d1372eaa4e52c24d0eb97118a8607a8f437fe128c0a92aa0ff9602ddde2"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(4), new U256(10), networkId, payment5);
    return sendParcel(p, new H256("6f5b38da7dcfb2fa43bcc0e20b7c8eee8773bcd94f3f8ec32dba4e10a93256fa"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(6), new U256(10), networkId, payment7, payment8);
    return sendParcel(p, new H256("3358086da063e7dfee4a3022c0b3bda9f93c6f4ae4acc17fd13e19d8306702cb"));
}).then(printTransactionInvoices).then( () => {
    const p = new Parcel(new U256(9), new U256(10), networkId, mint1, mint2);
    return sendParcel(p, new H256("aa761e78ee92ac5a94ded11bdd7ed0774737452c042805989b62dad4951f7051"));
}).then(printResults).then( () => {
}).then( () => {
    const p = new Parcel(new U256(10), new U256(10), networkId, transfer1);
    return sendParcel(p, new H256("d4af0ed1be40511ba0b8c705becdef5f049ea4bac0cee56ddc487276f2511dc6"));
}).then(printResults).then( () => {
}).then( () => {
    const p = new Parcel(new U256(11), new U256(10), networkId, transfer2);
    return sendParcel(p, new H256("72390e25526a83b2f7b855c6f0f8e13045601345affce19ec33079859c2df159"));
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