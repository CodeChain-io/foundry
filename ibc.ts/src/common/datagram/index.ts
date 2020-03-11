export interface Datagram {
    rlpBytes(): Buffer;
    toEncodeObject(): any[];
}

export const ChannelOrdered = 0;
export const ChannelUnordered = 1;

export interface PacketJSON {
    sequence: number;
    timeoutHeight: number;
    sourcePort: string;
    sourceChannel: string;
    destPort: string;
    destChannel: string;
    data: string;
}

export class Packet {
    public readonly sequence: number;
    public readonly timeoutHeight: number;
    public readonly sourcePort: string;
    public readonly sourceChannel: string;
    public readonly destPort: string;
    public readonly destChannel: string;
    public readonly data: Buffer;

    public constructor({
        sequence,
        timeoutHeight,
        sourcePort,
        sourceChannel,
        destPort,
        destChannel,
        data
    }: PacketJSON) {
        this.sequence = sequence;
        this.timeoutHeight = timeoutHeight;
        this.sourcePort = sourcePort;
        this.sourceChannel = sourceChannel;
        this.destPort = destPort;
        this.destChannel = destChannel;
        this.data = Buffer.from(data, "hex");
    }

    public toEncodeObject(): any[] {
        return [
            this.sequence,
            this.timeoutHeight,
            this.sourcePort,
            this.sourceChannel,
            this.destPort,
            this.destChannel,
            this.data
        ];
    }
}
