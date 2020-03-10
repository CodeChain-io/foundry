const RLP = require("rlp");

export class ChanOpenInitDatagram {
    private order: number;
    private connection: string;
    private channelIdentifier: string;
    private counterpartyChannelIdentifier: string;
    private version: string;

    public constructor({
        order,
        connection,
        channelIdentifier,
        counterpartyChannelIdentifier,
        version
    }: {
        order: number;
        connection: string;
        channelIdentifier: string;
        counterpartyChannelIdentifier: string;
        version: string;
    }) {
        this.order = order;
        this.connection = connection;
        this.channelIdentifier = channelIdentifier;
        this.counterpartyChannelIdentifier = counterpartyChannelIdentifier;
        this.version = version;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            7,
            this.order,
            this.connection,
            this.channelIdentifier,
            this.counterpartyChannelIdentifier,
            this.version
        ];
    }
}
