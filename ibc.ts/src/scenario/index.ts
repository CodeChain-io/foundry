import Debug from "debug";
import { getConfig } from "../common/config";
import { Chain } from "../common/chain";
import { PlatformAddress } from "codechain-primitives/lib";
import { CreateClientDatagram } from "../common/datagram/createClient";
import { strict as assert } from "assert";
import { ConnOpenInitDatagram } from "../common/datagram/connOpenInit";
import { ChanOpenInitDatagram } from "../common/datagram/chanOpenInit";
import { ChannelOrdered, Packet } from "../common/datagram";
import { SendPacketDatagram } from "../common/datagram/sendPacket";
const { Select } = require("enquirer");

require("dotenv").config();

const debug = Debug("scenario:main");

// Fix the packet's sequence in the PoC scenario.
const SEQUENCE = 1;

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

    if ("exit" === (await runLightClientCreationPrompt({ chainA, chainB }))) {
        return;
    }

    if ("exit" === (await runConnectionCreationPrompt({ chainA, chainB }))) {
        return;
    }

    while (true) {
        const result = await runConnectionCheckPrompt({ chainA, chainB });
        if (result === "exit") {
            return;
        } else if (result === "break") {
            break;
        }
    }

    if ("exit" === (await runChannelCreationPrompt({ chainA, chainB }))) {
        return;
    }

    while (true) {
        const result = await runChannelCheckPrompt({ chainA, chainB });
        if (result === "exit") {
            return;
        } else if (result === "break") {
            break;
        }
    }

    if ("exit" === (await runSendPacketPrompt({ chainA, chainB }))) {
        return;
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

async function createChannel({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}) {
    await chainA.submitDatagram(
        new ChanOpenInitDatagram({
            order: ChannelOrdered,
            connection: chainA.counterpartyIdentifiers.connection,
            channelIdentifier: chainA.counterpartyIdentifiers.channel,
            counterpartyChannelIdentifier:
                chainB.counterpartyIdentifiers.channel,
            version: ""
        })
    );
}

async function checkChannels({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}) {
    const channelA = await chainA.queryChannel();
    console.log(`Channel in A ${JSON.stringify(channelA)}`);
    const channelB = await chainB.queryChannel();
    console.log(`Channel in B ${JSON.stringify(channelB)}`);
}

async function sendPacket({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}) {
    const chainBHeight = await chainB.latestHeight();
    await chainA.submitDatagram(
        new SendPacketDatagram({
            packet: new Packet({
                sequence: SEQUENCE,
                timeoutHeight: chainBHeight + 1000,
                sourcePort: "DEFAULT_PORT",
                sourceChannel: chainA.counterpartyIdentifiers.channel,
                destPort: "DEFAULT_PORT",
                destChannel: chainB.counterpartyIdentifiers.channel,
                data: Buffer.from("PING", "utf8").toString("hex")
            })
        })
    );
}

async function runLightClientCreationPrompt({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}): Promise<"exit" | null> {
    const lightclientPrompt = new Select({
        name: "light client",
        message: "Will you create light clients?",
        choices: ["yes", "skip", "exit"]
    });
    const lightclientAnswer = await lightclientPrompt.run();

    if (lightclientAnswer === "exit") {
        return "exit";
    }

    if (lightclientAnswer === "yes") {
        console.log("Create a light client in chain A");
        await createLightClient({ chain: chainA, counterpartyChain: chainB });
        console.log("Create a light client in chain B");
        await createLightClient({ chain: chainB, counterpartyChain: chainA });
    }

    return null;
}

async function runConnectionCreationPrompt({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}): Promise<"exit" | null> {
    const connectionPrompt = new Select({
        name: "connection",
        message: "Will you create connection?",
        choices: ["yes", "skip", "exit"]
    });
    const connectionAnswer = await connectionPrompt.run();

    if (connectionAnswer === "exit") {
        return "exit";
    }

    if (connectionAnswer === "yes") {
        console.log("Create a connection");
        await createConnection({ chainA, chainB });
    }

    return null;
}

async function runConnectionCheckPrompt({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}): Promise<"exit" | "break" | null> {
    const connectionCheckPrompt = new Select({
        name: "connection check",
        message: "Will you check connection?",
        choices: ["yes", "skip", "exit"]
    });
    const connectionCheckAnswer = await connectionCheckPrompt.run();

    if (connectionCheckAnswer === "exit") {
        return "exit";
    }

    if (connectionCheckAnswer === "yes") {
        console.log("Check a connection");
        await checkConnections({ chainA, chainB });
    }

    if (connectionCheckAnswer === "skip") {
        return "break";
    }

    return null;
}

async function runChannelCreationPrompt({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}): Promise<"exit" | null> {
    const channelPrompt = new Select({
        name: "channel",
        message: "Will you create a channel?",
        choices: ["yes", "skip", "exit"]
    });
    const channelAnswer = await channelPrompt.run();

    if (channelAnswer === "exit") {
        return "exit";
    }

    if (channelAnswer === "yes") {
        console.log("Create a channel");
        await createChannel({ chainA, chainB });
    }

    return null;
}

async function runChannelCheckPrompt({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}): Promise<"exit" | "break" | null> {
    const channelCheckPrompt = new Select({
        name: "channel check",
        message: "Will you check the channel?",
        choices: ["yes", "skip", "exit"]
    });
    const channelCheckAnsser = await channelCheckPrompt.run();

    if (channelCheckAnsser === "exit") {
        return "exit";
    }

    if (channelCheckAnsser === "yes") {
        console.log("Check a channel");
        await checkChannels({ chainA, chainB });
    }

    if (channelCheckAnsser === "skip") {
        return "break";
    }

    return null;
}

async function runSendPacketPrompt({
    chainA,
    chainB
}: {
    chainA: Chain;
    chainB: Chain;
}): Promise<"exit" | null> {
    const packetPrompt = new Select({
        name: "packet",
        message: "Will you send a packet?",
        choices: ["yes", "skip", "exit"]
    });

    const packetAnswer = await packetPrompt.run();

    if (packetAnswer === "exit") {
        return "exit";
    }

    if (packetAnswer === "yes") {
        console.log("Send a packet");
        await sendPacket({ chainA, chainB });
    }

    return null;
}
