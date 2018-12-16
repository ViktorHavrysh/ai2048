const ai2048 = import("../ai2048-wasm/pkg");
ai2048.then(m => { m.init() });

export class Ai {
    constructor() {
        let self = this;
        ai2048.then(m => self.evaluate_position = m.evaluate_position);
    }
}
