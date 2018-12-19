import Ai from "./ai";
import { Direction } from "./direction";
import { GameState } from "./game_state";
import { Grid } from "./grid";
import { HTMLActuator as Actuator } from "./html_actuator";
import StorageManager from "./local_storage_manager";
import Position from "./position";
import { Tile } from "./tile";
import timeout from "./timeout";

interface Vector {
  x: number;
  y: number;
}

export default class GameManager {
  private readonly size: number = 4;
  private readonly storageManager: StorageManager;
  private readonly actuator: Actuator;
  private readonly ai: Ai;
  private readonly startTiles: number = 2;
  private keepPlaying = false;
  private grid: Grid = new Grid();
  private over = false;
  private won = false;
  private score = 0;
  private aiIsRunning = false;

  private readonly directionMap = new Map<Direction, Vector>([
    [Direction.Up, { x: 0, y: -1 }],
    [Direction.Right, { x: 1, y: 0 }],
    [Direction.Down, { x: 0, y: 1 }],
    [Direction.Left, { x: -1, y: 0 }]
  ]);

  public constructor(
    storageManager: StorageManager,
    actuator: Actuator,
    ai: Ai
  ) {
    this.storageManager = storageManager;
    this.actuator = actuator;
    this.ai = ai;
  }
  // Set up the game
  public setup(): void {
    const previousState = this.storageManager.getGameState();
    if (previousState) {
      this.loadState(previousState);
    } else {
      this.clearState();
      this.addStartTiles();
    }
    this.actuate();
  }
  // Move tiles on the grid in the specified direction
  public move(direction: Direction): void {
    if (this.isGameTerminated()) return; // Don't do anything if the game's over
    const vector = this.getVector(direction);
    const traversals = this.buildTraversals(vector);
    let moved = false;
    // Save the current tile positions and remove merger information
    this.prepareTiles();
    // Traverse the grid in the right direction and move tiles
    for (const x of traversals.x) {
      for (const y of traversals.y) {
        const position: Position = { x: x, y: y };
        const tile = this.grid.tileAtPosition(position);
        if (tile) {
          const { farthest, next } = this.findFarthestPosition(
            position,
            vector
          );
          const nextTile = this.grid.tileAtPosition(next);
          // Only one merger per row traversal?
          if (
            nextTile &&
            nextTile.value === tile.value &&
            !nextTile.mergedFrom
          ) {
            const merged = new Tile(next, tile.value * 2);
            merged.mergedFrom = [tile, nextTile];
            this.grid.insertTile(merged);
            this.grid.removeTileAtPosition(tile);
            // Converge the two tiles' positions
            tile.updatePosition(next);
            // Update the score
            this.score += merged.value;
            // The mythical 65536 tile
            if (merged.value === 65536) this.won = true;
          } else {
            this.moveTile(tile, farthest);
          }
          if (!this.positionsEqual(position, tile)) {
            moved = true; // The tile moved from its original cell!
          }
        }
      }
    }
    if (moved) {
      this.addRandomTile();
      if (!this.movesAvailable()) {
        this.over = true; // Game over!
        this.aiIsRunning = false;
      }
      this.actuate();
    }
  }
  public plus(): void {
    const strength = this.ai.increaseStrength();
    this.storageManager.setGameState(this.serialize());
    this.actuator.updateStrength(strength);
  }
  public minus(): void {
    const strength = this.ai.decreaseStrength();
    this.storageManager.setGameState(this.serialize());
    this.actuator.updateStrength(strength);
  }
  // Restart the game
  public restart(): void {
    this.storageManager.clearGameState();
    this.actuator.continueGame(); // Clear the game won/lost message
    this.setup();
  }
  // Keep playing after winning (allows going over 2048)
  public continuePlaying(): void {
    this.keepPlaying = true;
    this.actuator.continueGame(); // Clear the game won/lost message
  }
  public toggleAi(): void {
    this.aiIsRunning = !this.aiIsRunning;
    this.actuator.updateRunButton(this.aiIsRunning);
    if (this.aiIsRunning) {
      this.ai.chooseDirection(this.grid.forAi()).then(d => this.move(d));
    }
  }
  private loadState(previousState: GameState) {
    this.grid = new Grid(previousState.grid);
    this.score = previousState.score;
    this.over = previousState.over;
    this.won = previousState.won;
    this.keepPlaying = previousState.keepPlaying;
    this.ai.setStrength(previousState.aiStrength);
  }
  private clearState(): void {
    this.grid = new Grid();
    this.score = 0;
    this.over = false;
    this.won = false;
    this.keepPlaying = false;
    this.aiIsRunning = false;
  }
  // Sends the updated grid to the actuator
  private async actuate(): Promise<void> {
    if (this.storageManager.getBestScore() < this.score) {
      this.storageManager.setBestScore(this.score);
    }
    // Clear the state when the game is over (game over only, not win)
    if (this.over) {
      this.storageManager.clearGameState();
    } else {
      this.storageManager.setGameState(this.serialize());
    }
    await this.actuator.actuate(this.grid, {
      score: this.score,
      over: this.over,
      won: this.won,
      bestScore: this.storageManager.getBestScore(),
      terminated: this.isGameTerminated(),
      strength: this.ai.strength(),
      aiIsOn: () => this.aiIsRunning
    });
    if (this.aiIsRunning) {
      const to = timeout(100);
      const direction = await this.ai.chooseDirection(this.grid.forAi());
      await to; // make sure moves are at least 100 milliseconds
      this.move(direction);
    }
  }
  // Return true if the game is lost, or has won and the user hasn't kept playing
  private isGameTerminated(): boolean {
    return this.over || (this.won && !this.keepPlaying);
  }
  // Set up the initial tiles to start the game with
  private addStartTiles(): void {
    for (let i = 0; i < this.startTiles; i++) {
      this.addRandomTile();
    }
  }
  // Adds a tile in a random position
  private addRandomTile(): void {
    if (this.grid.tilesAvailable()) {
      const value = Math.random() < 0.9 ? 2 : 4;
      const tile = new Tile(this.grid.randomAvailablePosition()!, value);
      this.grid.insertTile(tile);
    }
  }
  // Represent the current game as an object
  private serialize(): GameState {
    return {
      grid: this.grid.serialize(),
      score: this.score,
      over: this.over,
      won: this.won,
      keepPlaying: this.keepPlaying,
      aiStrength: this.ai.strength()
    };
  }
  // Save all tile positions and remove merger info
  private prepareTiles(): void {
    this.grid.eachTile((_x, _y, tile) => {
      if (tile) {
        tile.mergedFrom = null;
        tile.savePosition();
      }
    });
  }
  // Move a tile and its representation
  private moveTile(tile: Tile, cell: Position): void {
    const cells: any = this.grid.tiles;
    cells[tile.x][tile.y] = null;
    cells[cell.x][cell.y] = tile;
    tile.updatePosition(cell);
  }
  // Get the vector representing the chosen direction
  private getVector(direction: Direction): Vector {
    return this.directionMap.get(direction)!;
  }
  // Build a list of positions to traverse in the right order
  private buildTraversals(vector: Vector): { x: number[]; y: number[] } {
    const traversals: { x: number[]; y: number[] } = { x: [], y: [] };
    for (let pos = 0; pos < this.size; pos++) {
      traversals.x.push(pos);
      traversals.y.push(pos);
    }
    // Always traverse from the farthest cell in the chosen direction
    if (vector.x === 1) traversals.x = traversals.x.reverse();
    if (vector.y === 1) traversals.y = traversals.y.reverse();
    return traversals;
  }
  private findFarthestPosition(
    position: Position,
    vector: Vector
  ): { farthest: Position; next: Position } {
    let previous;
    // Progress towards the vector direction until an obstacle is found
    do {
      previous = position;
      position = { x: previous.x + vector.x, y: previous.y + vector.y };
    } while (
      this.grid.withinBounds(position) &&
      this.grid.tileAvailable(position)
    );
    return {
      farthest: previous,
      next: position // Used to check if a merge is required
    };
  }
  private movesAvailable(): boolean {
    return this.grid.tilesAvailable() || this.tileMatchesAvailable();
  }
  // Check for available matches between tiles (more expensive check)
  private tileMatchesAvailable(): boolean {
    for (let x = 0; x < this.size; x++) {
      for (let y = 0; y < this.size; y++) {
        const tile = this.grid.tileAtPosition({ x: x, y: y });
        if (tile) {
          for (let direction = 0; direction < 4; direction++) {
            const vector = this.getVector(direction);
            const cell = { x: x + vector.x, y: y + vector.y };
            const other = this.grid.tileAtPosition(cell);
            if (other && other.value === tile.value) {
              return true; // These two tiles can be merged
            }
          }
        }
      }
    }
    return false;
  }
  private positionsEqual(first: Position, second: Position): boolean {
    return first.x === second.x && first.y === second.y;
  }
}
