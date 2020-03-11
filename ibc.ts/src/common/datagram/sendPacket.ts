import { Packet } from ".";

const RLP = require("rlp");

export class SendPacketDatagram {
    private packet: Packet;

    public constructor({ packet }: { packet: Packet }) {
        this.packet = packet;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [13, this.packet.toEncodeObject()];
    }
}
