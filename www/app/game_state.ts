import { SerializableGrid } from "./grid";

export default interface GameState {
  grid: SerializableGrid;
  score: number;
  over: boolean;
  won: boolean;
  keepPlaying: boolean;
}
