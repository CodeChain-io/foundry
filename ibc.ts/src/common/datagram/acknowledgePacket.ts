import { Packet } from ".";

const RLP = require("rlp");

export class AcknowledgePacketDatagram {
    private packet: Packet;
    private ack: Buffer;
    private proof: Buffer;
    private proofHeight: number;

    public constructor({
        packet,
        ack,
        proof,
        proofHeight
    }: {
        packet: Packet;
        ack: Buffer;
        proof: Buffer;
        proofHeight: number;
    }) {
        this.packet = packet;
        this.ack = ack;
        this.proof = proof;
        this.proofHeight = proofHeight;
    }

    public rlpBytes(): Buffer {
        return RLP.encode(this.toEncodeObject());
    }

    public toEncodeObject(): any[] {
        return [
            15,
            this.packet.toEncodeObject(),
            this.ack,
            this.proof,
            this.proofHeight
        ];
    }
}
