import { GameState, version } from "./game_state";

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
  public getGameState(): GameState | null {
    const stateJSON = this.storage.getItem(this.gameStateKey);
    if (!stateJSON) return null;
    const store = JSON.parse(stateJSON);
    if (!store.version || store.version !== version) return null;
    return store.state;
  }
  public setGameState(gameState: GameState): void {
    const store = { version: version, state: gameState };
    this.storage.setItem(this.gameStateKey, JSON.stringify(store));
  }
  public clearGameState(): void {
    this.storage.removeItem(this.gameStateKey);
  }
}
