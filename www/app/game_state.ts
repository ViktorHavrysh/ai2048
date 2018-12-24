import { SerializableGrid } from "./grid";

// increment whenever the interface changes in a backwards incompatible way
export const Version = 1;

export interface GameState {
  grid: SerializableGrid;
  score: number;
  over: boolean;
  won: boolean;
  keepPlaying: boolean;
  aiStrength: number;
}
