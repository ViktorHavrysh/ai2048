import GameState from "./game_state";

export default class LocalStorageManager {
  private readonly bestScoreKey = "bestScore";
  private readonly gameStateKey = "gameState";
  private readonly storage = window.localStorage;
  // Best score getters/setters
  public getBestScore(): number {
    return Number(this.storage.getItem(this.bestScoreKey)) || 0;
  }
  public setBestScore(score: number): void {
    this.storage.setItem(this.bestScoreKey, score.toString());
  }
  // Game state getters/setters and clearing
  getGameState(): GameState | null {
    var stateJSON = this.storage.getItem(this.gameStateKey);
    return stateJSON ? JSON.parse(stateJSON) : null;
  }
  setGameState(gameState: GameState): void {
    this.storage.setItem(this.gameStateKey, JSON.stringify(gameState));
  }
  clearGameState(): void {
    this.storage.removeItem(this.gameStateKey);
  }
}
