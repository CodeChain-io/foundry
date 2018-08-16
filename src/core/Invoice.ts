/**
 * An Invoice is used to know whether a transaction or a parcel succeeded or
 * failed.
 */
export class Invoice {
    readonly success: boolean;
    readonly error?: { type: string, content?: any };

    /**
     * @param success Whether a transaction or a parcel succeeded or failed.
     * @param error.type The type of the error.
     * @param error.content An explanation of the error.
     */
    constructor(success: boolean, error?: { type: string, content?: any }) {
        this.success = !!success;
        this.error = error;
    }

    /**
     * Create an Invoice from an Invoice JSON object.
     * @param data An Invoice JSON object.
     * @returns An Invoice.
     */
    static fromJSON(data: { success: boolean, error?: { type: string, content?: any } }) {
        const { success, error } = data;
        return new this(success, error);
    }

    /**
     * Convert to an Invoice JSON object.
     * @returns An Invoice JSON object.
     */
    toJSON() {
        const { success, error } = this;
        return {
            success,
            error
        };
    }
}
