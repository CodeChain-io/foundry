import { H256 } from "../../core/H256";

import { AssetTransferAddress } from "../AssetTransferAddress";

test("AssetTransferAddress.fromString - mainnet", () => {
    const address = AssetTransferAddress.fromString(
        "ccaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvcev083x"
    );
    expect(address.payload).toEqual(
        new H256(
            "50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"
        )
    );
});

test("AssetTransferAddress.fromString - testnet", () => {
    const address = AssetTransferAddress.fromString(
        "tcaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvctt5eze"
    );
    expect(address.payload).toEqual(
        new H256(
            "50a2c0d145539c1fb32f60e0d8425b1c03f6120c40171971b8de9c0017a4bfb3"
        )
    );
});

test("AssetTransferAddress.fromString - invalid checksum", () => {
    expect(() => {
        AssetTransferAddress.fromString(
            "ccaqqq9pgkq69z488qlkvhkpcxcgfd3cqlkzgxyq9cewxuda8qqz7jtlvcqqqqqq"
        );
    }).toThrow();
});
