import Debug from "debug";
import { getConfig } from "../common/config";
import { Chain } from "../common/chain";
import { PlatformAddress } from "codechain-primitives/lib";
import { CreateClientDatagram } from "../common/datagram/createClient";
import { strict as assert } from "assert";
import { ConnOpenInitDatagram } from "../common/datagram/connOpenInit";
const { Select } = require("enquirer");

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

    const lightclientPrompt = new Select({
        name: "light client",
        message: "Will you create light clients?",
        choices: ["yes", "skip", "exit"]
    });
    const lightclientAnswer = await lightclientPrompt.run();

    if (lightclientAnswer === "exit") {
        return;
    }

    if (lightclientAnswer === "yes") {
        console.log("Create a light client in chain A");
        await createLightClient({ chain: chainA, counterpartyChain: chainB });
        console.log("Create a light client in chain B");
        await createLightClient({ chain: chainB, counterpartyChain: chainA });
    }

    const connectionPrompt = new Select({
        name: "connection",
        message: "Will you create connection?",
        choices: ["yes", "skip", "exit"]
    });
    const connectionAnswer = await connectionPrompt.run();

    if (connectionAnswer === "exit") {
        return;
    }

    if (connectionAnswer === "yes") {
        console.log("Create a connection");
        await createConnection({ chainA, chainB });
    }

    while (true) {
        const connectionCheckPrompt = new Select({
            name: "connection check",
            message: "Will you check connection?",
            choices: ["yes", "skip", "exit"]
        });
        const connectionCheckAnswer = await connectionCheckPrompt.run();

        if (connectionCheckAnswer === "exit") {
            return;
        }

        if (connectionCheckAnswer === "yes") {
            console.log("Check a connection");
            await checkConnections({ chainA, chainB });
        }

        if (connectionCheckAnswer === "skip") {
            break;
        }
    }
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

async function createConnection({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}) {
    await chainA.submitDatagram(
        new ConnOpenInitDatagram({
            id: chainA.counterpartyIdentifiers.connection,
            desiredCounterpartyConnectionIdentifier:
                chainB.counterpartyIdentifiers.connection,
            counterpartyPrefix: "",
            clientIdentifier: chainA.counterpartyIdentifiers.client,
            counterpartyClientIdentifier: chainB.counterpartyIdentifiers.client
        })
    );
}

async function checkConnections({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}) {
    const connectionA = await chainA.queryConnection();
    console.log(`Connection in A ${JSON.stringify(connectionA)}`);

    const connectionB = await chainB.queryConnection();
    console.log(`Connection in B ${JSON.stringify(connectionB)}`);
}
