import PromiseWorker from "promise-worker";
import { Direction } from "./direction";

interface MessageForAi {
  grid: Uint32Array;
  minProb: number;
}

const StrengthMap: { [index: number]: number } = {
  1: 0.02,
  2: 0.01,
  3: 0.005,
  4: 0.003,
  5: 0.002,
  6: 0.001,
  7: 0.0005,
  8: 0.0003,
  9: 0.0002,
  10: 0.0001
};
const MinStrength = 1;
const MaxStrength = 10;

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
    const minProb = StrengthMap[this.strength];
    const message: MessageForAi = {
      grid: grid,
      minProb: minProb
    };
    const reply = await this.worker.postMessage(message);
    return reply;
  }
}
