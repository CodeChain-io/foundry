// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
import { createDecipheriv, createHash } from "crypto";
import { Socket } from "net";
import {
    exchange,
    generatePrivateKey,
    H256,
    toArray,
    U128,
    U256,
    x25519GetPublicFromPrivate,
    X25519Private
} from "../../primitives/src";
import { BlockSyncMessage } from "./blockSyncMessage";
import {
    Ack,
    fromBytes,
    MessageType,
    NegotiationRequest,
    SignedMessage,
    Sync1,
    Unencrypted
} from "./message";
import { TendermintMessage } from "./tendermintMessage";
import { TransactionSyncMessage } from "./transactionSyncMessage";

// Note: This function is needed becasue the pure javascript ed25519 and curve25519
// implementation in "tweetnacl" does not support x25519 key generation for key exchanges.
const ed25519SkToCurve25519 = (skStr: string): X25519Private => {
    const sk = toArray(skStr);
    const seed = sk.slice(0, 32);
    const seedHash = createHash("sha512")
        .update(seed)
        .digest();
    seedHash[0] &= 248;
    seedHash[31] &= 127;
    seedHash[31] |= 64;
    return seedHash.slice(0, 32).toString("hex");
};

export class P2pLayer {
    private readonly ip: string;
    private readonly port: number;
    private readonly socket: Socket;
    private readonly arrivedExtensionMessage: (
        | BlockSyncMessage
        | TransactionSyncMessage
        | TendermintMessage
    )[];
    private tcpBuffer: Buffer;
    private genesisHash: H256;
    private recentHeaderNonce: U256;
    private recentBodyNonce: U256;
    private log: boolean;
    private readonly networkId: string;
    private readonly localKey: X25519Private;
    private sharedSecret?: string;
    private nonce?: U128;

    constructor(ip: string, port: number, networkId: string) {
        this.socket = new Socket();
        this.socket.setMaxListeners(0);
        this.ip = ip;
        this.port = port;
        this.arrivedExtensionMessage = [];
        this.tcpBuffer = Buffer.alloc(0);
        this.genesisHash = new H256(
            "0000000000000000000000000000000000000000000000000000000000000000"
        );
        this.recentHeaderNonce = new U256(0);
        this.recentBodyNonce = new U256(0);
        this.log = false;
        this.networkId = networkId;
        this.localKey = ed25519SkToCurve25519(generatePrivateKey());
    }

    public enableLog() {
        this.log = true;
    }

    public getGenesisHash(): H256 {
        return this.genesisHash;
    }

    public getArrivedExtensionMessage(): (
        | BlockSyncMessage
        | TransactionSyncMessage
        | TendermintMessage
    )[] {
        return this.arrivedExtensionMessage;
    }

    public getHeaderNonce(): U256 {
        return this.recentHeaderNonce;
    }

    public getBodyNonce(): U256 {
        return this.recentBodyNonce;
    }

    public async connect(): Promise<{}> {
        return new Promise(async resolve => {
            this.socket.connect({ port: this.port, host: this.ip }, () => {
                if (this.log) {
                    console.log("Start TCP connection");
                    console.log(
                        "   local = %s:%s",
                        this.socket.localAddress,
                        this.socket.localPort
                    );
                    console.log(
                        "   remote = %s:%s",
                        this.socket.remoteAddress,
                        this.socket.remotePort
                    );
                }
                this.sendP2pMessage(MessageType.SYNC1_ID);

                this.socket.on("data", (data: Buffer) => {
                    try {
                        this.tcpBuffer = Buffer.concat([this.tcpBuffer, data]);
                        while (this.tcpBuffer.length !== 0) {
                            const len = this.tcpBuffer.readUIntBE(0, 1);
                            if (len >= 0xf8) {
                                const lenOfLen = len - 0xf7;
                                const dataLen = this.tcpBuffer
                                    .slice(1, 1 + lenOfLen)
                                    .readUIntBE(0, lenOfLen);
                                if (
                                    this.tcpBuffer.length >=
                                    dataLen + lenOfLen + 1
                                ) {
                                    const rlpPacket = this.tcpBuffer.slice(
                                        0,
                                        dataLen + lenOfLen + 1
                                    );
                                    this.tcpBuffer = this.tcpBuffer.slice(
                                        dataLen + lenOfLen + 1,
                                        this.tcpBuffer.length
                                    );
                                    if (this.nonce == null) {
                                        this.onHandshakeMessage(rlpPacket);
                                    } else {
                                        this.onSignedMessage(rlpPacket);
                                    }
                                    resolve();
                                } else {
                                    throw Error(
                                        "The rlp data has not arrived yet"
                                    );
                                }
                            } else if (len >= 0xc0) {
                                const dataLen = len - 0xc0;
                                if (this.tcpBuffer.length >= dataLen + 1) {
                                    const rlpPacket = this.tcpBuffer.slice(
                                        0,
                                        dataLen + 1
                                    );
                                    this.tcpBuffer = this.tcpBuffer.slice(
                                        dataLen + 1,
                                        this.tcpBuffer.length
                                    );
                                    if (this.nonce == null) {
                                        this.onHandshakeMessage(rlpPacket);
                                    } else {
                                        this.onSignedMessage(rlpPacket);
                                    }
                                    resolve();
                                } else {
                                    throw Error(
                                        "The rlp data has not arrived yet"
                                    );
                                }
                            } else {
                                throw Error("Invalid RLP data");
                            }
                        }
                    } catch (err) {
                        console.error(err);
                    }
                });
                this.socket.on("end", () => {
                    if (this.log) {
                        console.log("TCP disconnected");
                    }
                });
                this.socket.on("error", (err: any) => {
                    if (this.log) {
                        console.log("Socket Error: ", JSON.stringify(err));
                    }
                });
                this.socket.on("close", () => {
                    if (this.log) {
                        console.log("Socket Closed");
                    }
                });
            });
        });
    }

    public async sendExtensionMessage(
        extensionName: string,
        data: Buffer,
        needEncryption: boolean
    ) {
        let msg;
        if (this.nonce == null) {
            throw Error("Nonce is not set yet");
        }
        if (needEncryption) {
            throw Error("Not implemented");
        } else {
            msg = new Unencrypted(extensionName, data);
        }
        const signedMsg = new SignedMessage(msg, this.nonce);
        await this.writeData(signedMsg.rlpBytes());
    }

    public onHandshakeMessage(data: Buffer) {
        try {
            const msg = fromBytes(data);

            switch (msg.protocolId()) {
                case MessageType.SYNC1_ID: {
                    if (this.log) {
                        console.log("Got SYNC_ID message");
                    }
                    throw Error("Sync1 message is not implemented");
                    break;
                }
                case MessageType.SYNC2_ID: {
                    if (this.log) {
                        console.log("Got SYNC_ID message");
                    }
                    throw Error("Sync2 message is not implemented");
                    break;
                }
                case MessageType.ACK_ID: {
                    if (this.log) {
                        console.log("Got ACK_ID message");
                    }
                    const ack = msg as Ack;
                    const recipientPubKey = ack.recipientPubKey.toString();
                    this.sharedSecret = exchange(
                        recipientPubKey,
                        this.localKey
                    );

                    const ALGORITHM = "AES-256-CBC";
                    const key = Buffer.from(this.sharedSecret!, "hex");
                    const ivd = Buffer.from(
                        "00000000000000000000000000000000",
                        "hex"
                    );
                    const decryptor = createDecipheriv(ALGORITHM, key, ivd);
                    decryptor.write(ack.encryptedNonce);
                    decryptor.end();
                    this.nonce = new U128(
                        `0x${decryptor.read().toString("hex")}`
                    );

                    this.sendP2pMessage(MessageType.REQUEST_ID);
                    break;
                }
                default:
                    throw Error(
                        `${
                            MessageType[msg.protocolId()]
                        } is not one of the handshake message`
                    );
            }
        } catch (err) {
            console.error(err);
        }
    }

    public onSignedMessage(data: any) {
        if (this.nonce == null) {
            throw Error("Nonce is not specified");
        }
        const msg = SignedMessage.fromBytes(data, this.nonce);

        switch (msg.protocolId()) {
            case MessageType.REQUEST_ID: {
                if (this.log) {
                    console.log("Got REQUEST_ID message");
                }
                throw Error("Request message is not implemented");
                break;
            }
            case MessageType.RESPONSE_ID: {
                if (this.log) {
                    console.log("Got REQUEST_ID message");
                }
                break;
            }
            case MessageType.ENCRYPTED_ID: {
                if (this.log) {
                    console.log("Got ENCRYPTED_ID message");
                }
                throw Error("Encrypted message is not implemented");
            }
            case MessageType.UNENCRYPTED_ID: {
                if (this.log) {
                    console.log("Got UNENCRYPTED_ID message");
                }
                const unencrypted = msg.message as Unencrypted;
                this.onExtensionMessage(
                    unencrypted.extensionName,
                    unencrypted.data
                );
                break;
            }
            default:
                throw Error(
                    `${msg.protocolId()} is not a valid protocol id for signed messaged`
                );
        }
    }

    public onExtensionMessage(extensionName: string, msg: Buffer) {
        switch (extensionName) {
            case "block-propagation": {
                const extensionMsg = BlockSyncMessage.fromBytes(msg);
                this.arrivedExtensionMessage.push(extensionMsg);
                const body = extensionMsg.getBody();
                if (body.type === "status") {
                    this.genesisHash = body.genesisHash;
                } else if (body.type === "request") {
                    const message = body.message.getBody();
                    if (message.type === "headers") {
                        this.recentHeaderNonce = body.id;
                    } else if (message.type === "bodies") {
                        this.recentBodyNonce = body.id;
                        if (this.log) {
                            console.log(message.data);
                        }
                    }
                }
                if (this.log) {
                    console.log(extensionMsg);
                    console.log(extensionMsg.getBody());
                }

                break;
            }
            case "transaction-propagation": {
                const extensionMsg = TransactionSyncMessage.fromBytes(msg);
                this.arrivedExtensionMessage.push(extensionMsg);
                if (this.log) {
                    console.log(extensionMsg);
                }
                break;
            }
            case "tendermint": {
                const extensionMsg = TendermintMessage.fromBytes(msg);
                this.arrivedExtensionMessage.push(extensionMsg);
                if (this.log) {
                    console.log(extensionMsg);
                }
                break;
            }
            default:
                throw Error("Not implemented");
        }
    }

    public async close() {
        await this.socket.end();
    }

    private async sendP2pMessage(messageType: MessageType): Promise<void> {
        switch (messageType) {
            case MessageType.SYNC1_ID: {
                if (this.log) {
                    console.log("Send SYNC_ID Message");
                }
                const { localKey, port, networkId } = this;
                const localPubKey = x25519GetPublicFromPrivate(localKey);
                const msg = new Sync1(new H256(localPubKey), networkId, port);
                await this.writeData(msg.rlpBytes());
                break;
            }
            case MessageType.REQUEST_ID: {
                if (this.log) {
                    console.log("Send REQUEST_ID Message");
                }
                const extensions = [
                    "block-propagation",
                    "transaction-propagation",
                    "tendermint"
                ];
                const messages = extensions.map(
                    extensionName => new NegotiationRequest(extensionName, [0])
                );
                const signedMessages = messages.map(
                    msg => new SignedMessage(msg, this.nonce!)
                );
                const sends = signedMessages.map(msg =>
                    this.writeData(msg.rlpBytes())
                );
                await Promise.all(sends);
                break;
            }
            default:
                throw Error(
                    `Sending ${MessageType[messageType]} is not implemented`
                );
        }
    }

    private async writeData(data: Buffer) {
        const success = await !this.socket.write(data);
        if (!success) {
            await this.socket.once("drain", () => this.writeData(data));
        }
    }
}
