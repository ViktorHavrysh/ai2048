import EventManager from "./event_manager";
import MessageForAi from "./ai_itnterop";
import AiWorker from "./ai.worker";

export default class Ai {
  private readonly aiWorker: Worker;
  private readonly eventManager: EventManager;
  private readonly minProb: number;
  private maxDepth: number;
  public constructor(
    eventManager: EventManager,
    minProb: number,
    maxDepth: number
  ) {
    this.aiWorker = new AiWorker("ai");
    this.eventManager = eventManager;
    this.minProb = minProb;
    this.maxDepth = maxDepth;
    this.eventManager.on("plus", this.plus.bind(this));
    this.eventManager.on("minus", this.minus.bind(this));
    this.aiWorker.addEventListener("message", ev =>
      this.eventManager.emit("aiMove", ev.data)
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
  public strength(): number {
    return this.maxDepth;
  }
  public evaluatePosition(grid: Uint32Array): void {
    let message: MessageForAi = {
      grid: grid,
      minProb: this.minProb,
      maxDepth: this.maxDepth
    };
    this.aiWorker.postMessage(message);
  }
}
