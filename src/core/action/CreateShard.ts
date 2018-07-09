export class CreateShard {
    constructor() { }

    toEncodeObject(): Array<any> {
        return [4];
    }

    toJSON() {
        return {
            action: "createShard",
        };
    }
}
