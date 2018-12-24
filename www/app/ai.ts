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

const StrengthMap: Strength[] = [
  { minProb: 0.001, maxDepth: 5 },
  { minProb: 0.0009, maxDepth: 6 },
  { minProb: 0.0008, maxDepth: 7 },
  { minProb: 0.0007, maxDepth: 8 },
  { minProb: 0.0006, maxDepth: 9 },
  { minProb: 0.0005, maxDepth: 10 },
  { minProb: 0.0004, maxDepth: 11 },
  { minProb: 0.0003, maxDepth: 12 },
  { minProb: 0.0002, maxDepth: 12 },
  { minProb: 0.0001, maxDepth: 12 }
];
const MinStrength = 1;
const MaxStrength = StrengthMap.length;

export default class Ai {
  private readonly worker: PromiseWorker;
  private strength = 8;
  public constructor() {
    this.worker = new PromiseWorker(new Worker("./worker.js"));
  }
  public getStrength(): number {
    return this.strength;
  }
  public setStrength(strength: number): void {
    strength = Math.round(strength);
    if (strength > MaxStrength) strength = MaxStrength;
    if (strength < MinStrength) strength = MinStrength;
    this.strength = strength;
  }
  public increaseStrength(): number {
    if (this.strength < MaxStrength) {
      this.strength++;
    }
    return this.strength;
  }
  public decreaseStrength(): number {
    if (this.strength > MinStrength) {
      this.strength--;
    }
    return this.strength;
  }
  public async chooseDirection(grid: Uint32Array): Promise<Direction> {
    const strength = StrengthMap[this.strength];
    const message: MessageForAi = {
      grid: grid,
      minProb: strength.minProb,
      maxDepth: strength.maxDepth
    };
    const reply = await this.worker.postMessage(message);
    return reply;
  }
}
