import EventManager from "./event_manager";

const ai2048 = import("../ai2048-wasm/pkg");
ai2048.then(m => {
  m.init();
});

export default class Ai {
  private readonly eventManager: EventManager;
  private readonly minProb: number;
  private maxDepth: number;
  private evaluate_position:
    | ((grid: Uint32Array, minProb: number, maxDepth: number) => number)
    | null = null;

  public constructor(
    eventManager: EventManager,
    minProb: number,
    maxDepth: number
  ) {
    this.eventManager = eventManager;
    this.minProb = minProb;
    this.maxDepth = maxDepth;
    this.eventManager.on("plus", this.plus.bind(this));
    this.eventManager.on("minus", this.minus.bind(this));
    const self = this;
    ai2048.then(m => (self.evaluate_position = m.evaluate_position));
  }
  private plus() {
    if (this.maxDepth < 10) {
      this.maxDepth++;
      this.eventManager.emit("update_strength", this.maxDepth);
    }
  }
  private minus() {
    if (this.maxDepth > 3) {
      this.maxDepth--;
      this.eventManager.emit("update_strength", this.maxDepth);
    }
  }
  public strength(): number {
    return this.maxDepth;
  }
  public evaluatePosition(grid: Uint32Array): number {
    return this.evaluate_position!(grid, this.minProb, this.maxDepth);
  }
}
