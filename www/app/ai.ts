import EventManager from "./event_manager";
import { Direction } from "./direction";
import PromiseWorker from "promise-worker";

interface MessageForAi {
  grid: Uint32Array;
  minProb: number;
  maxDepth: number;
}

export default class Ai {
  private readonly worker: PromiseWorker;
  private readonly minProb: number;
  private maxDepth: number;
  public constructor(
    eventManager: EventManager,
    minProb: number,
    maxDepth: number
  ) {
    this.worker = new PromiseWorker(new Worker("./worker.js"));
    this.minProb = minProb;
    this.maxDepth = maxDepth;
  }
  public strength(): number {
    return this.maxDepth;
  }
  public increaseStrength(): number {
    if (this.maxDepth < 10) {
      this.maxDepth++;
    }
    return this.maxDepth;
  }
  public decreaseStrength(): number {
    if (this.maxDepth > 3) {
      this.maxDepth--;
    }
    return this.maxDepth;
  }
  public async chooseDirection(grid: Uint32Array): Promise<Direction> {
    let message: MessageForAi = {
      grid: grid,
      minProb: this.minProb,
      maxDepth: this.maxDepth
    };
    const reply = await this.worker.postMessage(message);
    return reply;
  }
}
