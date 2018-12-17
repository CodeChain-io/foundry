import { PlatformAddress } from "codechain-primitives";

export interface TextJSON {
    content: string;
    certifier: string;
}

/**
 * Object used when getting a text by chain_getText.
 */
export class Text {
    public static fromJSON(data: TextJSON) {
        const { content, certifier } = data;
        return new Text({
            content,
            certifier: PlatformAddress.ensure(certifier)
        });
    }

    public readonly content: string;
    public readonly certifier: PlatformAddress;

    constructor(data: { content: string; certifier: PlatformAddress }) {
        const { content, certifier } = data;
        this.content = content;
        this.certifier = certifier;
    }

    public toJSON(): TextJSON {
        const { content, certifier } = this;
        return {
            content,
            certifier: certifier.value
        };
    }
}
