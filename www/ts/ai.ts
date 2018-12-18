import InputManager from "./input_manager";

const ai2048 = import("../ai2048-wasm/pkg");
ai2048.then(m => {
  m.init();
});

export default class Ai {
  private readonly inputManager: InputManager;
  private readonly events: { [index: string]: ((data: any) => void)[] } = {};
  private readonly minProb: number;
  private maxDepth: number;
  private evaluate_position:
    | ((grid: Uint32Array, minProb: number, maxDepth: number) => number)
    | null = null;

  public constructor(
    inputManager: InputManager,
    minProb: number,
    maxDepth: number
  ) {
    this.inputManager = inputManager;
    this.minProb = minProb;
    this.maxDepth = maxDepth;
    this.inputManager.on("plus", this.plus.bind(this));
    this.inputManager.on("minus", this.minus.bind(this));
    const self = this;
    ai2048.then(m => (self.evaluate_position = m.evaluate_position));
  }
  public on(event: string, callback: (data: any) => void): void {
    if (!this.events[event]) {
      this.events[event] = [];
    }
    this.events[event].push(callback);
  }
  private emit(event: string, data?: any): void {
    const callbacks = this.events[event];
    if (callbacks) {
      for (const callback of callbacks) {
        callback(data);
      }
    }
  }
  private plus() {
    if (this.maxDepth < 10) {
      this.maxDepth++;
      this.emit("update_strength");
    }
  }
  private minus() {
    if (this.maxDepth > 3) {
      this.maxDepth--;
      this.emit("update_strength");
    }
  }
  public strength(): number {
    return this.maxDepth;
  }
  public evaluatePosition(grid: Uint32Array): number {
    return this.evaluate_position!(grid, this.minProb, this.maxDepth);
  }
}
