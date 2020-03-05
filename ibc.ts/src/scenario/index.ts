import Debug from "debug";
import { getConfig } from "../common/config";
import { Chain } from "../common/chain";
import { PlatformAddress } from "codechain-primitives/lib";
import { CreateClientDatagram } from "../common/datagram/createClient";
import { strict as assert } from "assert";

require("dotenv").config();

const debug = Debug("scenario:main");

async function main() {
    const config = getConfig();
    const chainA = new Chain({
        server: config.chainA.rpcURL,
        networkId: config.chainA.networkId,
        faucetAddress: PlatformAddress.fromString(
            config.chainA.scenarioAddress
        ),
        counterpartyIdentifiers: {
            client: config.chainA.counterpartyClientId,
            connection: config.chainA.counterpartyConnectionId,
            channel: config.chainA.counterpartyChannelId
        },
        keystorePath: config.chainA.keystorePath
    });
    const chainB = new Chain({
        server: config.chainB.rpcURL,
        networkId: config.chainB.networkId,
        faucetAddress: PlatformAddress.fromString(
            config.chainB.scenarioAddress
        ),
        counterpartyIdentifiers: {
            client: config.chainB.counterpartyClientId,
            connection: config.chainB.counterpartyConnectionId,
            channel: config.chainB.counterpartyChannelId
        },
        keystorePath: config.chainB.keystorePath
    });

    console.log("Create a light client in chain A");
    await createLightClient({ chain: chainA, counterpartyChain: chainB });
    console.log("Create a light client in chain B");
    await createLightClient({ chain: chainB, counterpartyChain: chainA });
}

main().catch(console.error);

async function createLightClient({
    chain,
    counterpartyChain
}: {
    chain: Chain;
    counterpartyChain: Chain;
}) {
    debug("Create light client");
    const counterpartyBlockNumber = await counterpartyChain.latestHeight();
    const blockNumber = await chain.latestHeight();
    debug(`height is ${counterpartyBlockNumber}`);
    const counterpartyRawHeader = await counterpartyChain.queryChainHeader(
        counterpartyBlockNumber
    );
    debug(`rawHeader is ${counterpartyRawHeader}`);

    assert(counterpartyRawHeader, "header should not be empty");
    assert.notStrictEqual(
        counterpartyRawHeader!.substr(0, 2),
        "0x",
        "should not start with 0x"
    );

    debug(`Get queryClient`);
    const clientStateBefore = await chain.queryClient(blockNumber);
    assert.notEqual(clientStateBefore, null, "querying on the best block");
    assert.equal(clientStateBefore!.data, null, "client is not initialized");

    const createClient = new CreateClientDatagram({
        id: chain.counterpartyIdentifiers.client,
        kind: 0,
        consensusState: Buffer.alloc(0),
        data: Buffer.from(counterpartyRawHeader!, "hex")
    });

    debug(`Submit datagram`);
    await chain.submitDatagram(createClient);

    const clientStateAfter = await chain.queryClient();
    assert.notEqual(clientStateAfter, null, "querying on the best block");
    assert.notEqual(clientStateAfter!.data, null, "client is initialized");
    debug(`Create client is ${JSON.stringify(clientStateAfter)}`);
}
