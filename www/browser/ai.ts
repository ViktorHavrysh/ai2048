import EventManager from "./event_manager";

interface MessageForAi {
  grid: Uint32Array;
  minProb: number;
  maxDepth: number;
}

export default class Ai {
  private readonly aiWorker: Worker;
  private readonly eventManager: EventManager;
  private readonly minProb: number;
  private maxDepth: number;
  private aiIsOn: boolean = false;
  public constructor(
    eventManager: EventManager,
    minProb: number,
    maxDepth: number
  ) {
    this.aiWorker = new Worker("./worker.js");
    this.eventManager = eventManager;
    this.minProb = minProb;
    this.maxDepth = maxDepth;
    this.eventManager.on("plus", this.plus.bind(this));
    this.eventManager.on("minus", this.minus.bind(this));
    this.eventManager.on("moved", this.decideMove.bind(this));
    this.aiWorker.addEventListener("message", ev =>
      this.eventManager.emit("move", ev.data)
    );
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
  public run(grid: Uint32Array): void {
    this.aiIsOn = true;
    this.decideMove(grid);
  }
  public stop(): void {
    this.aiIsOn = false;
  }
  public isOn(): boolean {
    return this.aiIsOn;
  }
  public strength(): number {
    return this.maxDepth;
  }
  public decideMove(grid: Uint32Array) {
    if (this.aiIsOn) {
      let message: MessageForAi = {
        grid: grid,
        minProb: this.minProb,
        maxDepth: this.maxDepth
      };
      this.aiWorker.postMessage(message);
    }
  }
}
