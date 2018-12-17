const ai2048 = import("../ai2048-wasm/pkg");
ai2048.then(m => { m.init() });

export class Ai {
    constructor(inputManager, minProb, maxDepth) {
        this.events = {};
        this.inputManager = inputManager;
        this.minProb = minProb;
        this.maxDepth = maxDepth;
        this.inputManager.on("plus", this.plus.bind(this));
        this.inputManager.on("minus", this.minus.bind(this));
        let self = this;
        ai2048.then(m => self.evaluate_position = grid => m.evaluate_position(grid, self.minProb, self.maxDepth));
    }

    on(event, callback) {
        if (!this.events[event]) {
            this.events[event] = [];
        }
        this.events[event].push(callback);
    }
    emit(event, data) {
        var callbacks = this.events[event];
        if (callbacks) {
            callbacks.forEach(function (callback) {
                callback(data);
            });
        }
    }
    plus() {
        if (this.maxDepth < 10) {
            this.maxDepth++;
            this.emit("update_strength");
        }
    }

    minus() {
        if (this.maxDepth > 3) {
            this.maxDepth--;
            this.emit("update_strength");
        }
    }

    strength() {
        return this.maxDepth;
    }
}
