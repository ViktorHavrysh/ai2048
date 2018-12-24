import PromiseWorker from "promise-worker";
import { Direction } from "./direction";

interface MessageForAi {
  grid: Uint32Array;
  minProb: number;
  maxDepth: number;
}

interface Strength {
  minProb: number;
  maxDepth: number;
}

const minStrength = 1;
const maxStrength = 6;
const strengthMap: { [index: number]: Strength } = {
  1: { minProb: 0.001, maxDepth: 8 },
  2: { minProb: 0.0005, maxDepth: 10 },
  3: { minProb: 0.0004, maxDepth: 12 },
  4: { minProb: 0.0003, maxDepth: 12 },
  5: { minProb: 0.0002, maxDepth: 12 },
  6: { minProb: 0.0001, maxDepth: 12 }
};

export default class Ai {
  private readonly worker: PromiseWorker;
  private strength = 5;
  public constructor() {
    this.worker = new PromiseWorker(new Worker("./worker.js"));
  }
  public getStrength(): number {
    return this.strength;
  }
  public setStrength(strength: number): void {
    strength = Math.round(strength);
    if (strength > maxStrength) strength = maxStrength;
    if (strength < minStrength) strength = minStrength;
    this.strength = strength;
  }
  public increaseStrength(): number {
    if (this.strength < maxStrength) {
      this.strength++;
    }
    return this.strength;
  }
  public decreaseStrength(): number {
    if (this.strength > minStrength) {
      this.strength--;
    }
    return this.strength;
  }
  public async chooseDirection(grid: Uint32Array): Promise<Direction> {
    const strength = strengthMap[this.strength];
    const message: MessageForAi = {
      grid: grid,
      minProb: strength.minProb,
      maxDepth: strength.maxDepth
    };
    const reply = await this.worker.postMessage(message);
    return reply;
  }
}
