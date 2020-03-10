const RLP = require("rlp");

export class ChanOpenTryDatagram {
    private order: number;
    private connection: string;
    private channelIdentifier: string;
    private counterpartyChannelIdentifier: string;
    private version: string;
    private counterpartyVersion: string;
    private proofInit: Buffer;
    private proofHeight: number;

    public constructor({
        order,
        connection,
        channelIdentifier,
        counterpartyChannelIdentifier,
        version,
        counterpartyVersion,
        proofInit,
        proofHeight
    }: {
        order: number;
        connection: string;
        channelIdentifier: string;
        counterpartyChannelIdentifier: string;
        version: string;
        counterpartyVersion: string;
        proofInit: Buffer;
        proofHeight: number;
    }) {
        this.order = order;
        this.connection = connection;
        this.channelIdentifier = channelIdentifier;
        this.counterpartyChannelIdentifier = counterpartyChannelIdentifier;
        this.version = version;
        this.counterpartyVersion = counterpartyVersion;
        this.proofInit = proofInit;
        this.proofHeight = proofHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            8,
            this.order,
            this.connection,
            this.channelIdentifier,
            this.counterpartyChannelIdentifier,
            this.version,
            this.counterpartyVersion,
            this.proofInit,
            this.proofHeight
        ];
    }
}
