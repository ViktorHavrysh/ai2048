const ai2048 = import("../ai2048-wasm/pkg");
ai2048.then(m => { m.init() });

export class Ai {
    constructor(inputManager) {
        this.events = {};
        this.inputManager = inputManager;
        this.min_prob = 0.0001;
        this.max_depth = 5;
        this.inputManager.on("plus", this.plus.bind(this));
        this.inputManager.on("minus", this.minus.bind(this));
        let self = this;
        ai2048.then(m => self.evaluate_position = grid => m.evaluate_position(grid, self.min_prob, self.max_depth));
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
        if (this.max_depth < 10) {
            this.max_depth++;
            this.emit("update_strength");
        }
    }

    minus() {
        if (this.max_depth > 3) {
            this.max_depth--;
            this.emit("update_strength");
        }
    }

    strength() {
        return this.max_depth;
    }
}
