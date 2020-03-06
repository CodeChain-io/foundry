import { Address } from "foundry-primitives";

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
            certifier: Address.ensure(certifier)
        });
    }

    public readonly content: string;
    public readonly certifier: Address;

    constructor(data: { content: string; certifier: Address }) {
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
