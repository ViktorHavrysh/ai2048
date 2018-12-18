import { Grid } from "./grid";
import { Tile } from "./tile";
import InputManager from "./input_manager";
import StorageManager from "./local_storage_manager";
import { HTMLActuator as Actuator } from "./html_actuator";
import Ai from "./ai";
import GameState from "./game_state";
import { Direction } from "./direction";
import Position from "./position";

interface Vector {
  x: number;
  y: number;
}

export class GameManager {
  private readonly size: number = 4;
  private readonly inputManager: InputManager;
  private readonly storageManager: StorageManager;
  private readonly actuator: Actuator;
  private readonly ai: Ai;
  private readonly startTiles: number = 2;
  private aiIsOn: boolean = false;
  private keepPlaying: boolean = false;
  private grid: Grid | null = null;
  private over: boolean = false;
  private won: boolean = false;
  private score: number = 0;

  public constructor(
    inputManager: InputManager,
    storageManager: StorageManager,
    actuator: Actuator,
    ai: Ai
  ) {
    this.inputManager = inputManager;
    this.storageManager = storageManager;
    this.actuator = actuator;
    this.ai = ai;
    this.inputManager.on("move", this.move.bind(this));
    this.inputManager.on("restart", this.restart.bind(this));
    this.inputManager.on("keepPlaying", this.continuePlaying.bind(this));
    this.inputManager.on("run", this.run.bind(this));
    this.ai.on("update_strength", this.update_strength.bind(this));
  }
  private update_strength(): void {
    const strengthElement = document.getElementsByClassName(
      "strength-container"
    )[0];
    strengthElement.innerHTML = this.ai.strength().toString();
  }
  // Restart the game
  private restart(): void {
    this.aiIsOn = false;
    this.storageManager.clearGameState();
    this.actuator.continueGame(); // Clear the game won/lost message
    this.setup();
  }
  // Keep playing after winning (allows going over 2048)
  private continuePlaying(): void {
    this.keepPlaying = true;
    this.actuator.continueGame(); // Clear the game won/lost message
  }
  private run(): void {
    this.aiIsOn = !this.aiIsOn;
    this.setRunButton();
    this.runLoop();
  }
  private setRunButton() {
    const runButton = document.getElementsByClassName("run-button")[0];
    if (this.aiIsOn) {
      runButton.innerHTML = "Stop AI";
    } else {
      runButton.innerHTML = "Run AI";
    }
  }
  private runLoop() {
    if (!this.aiIsOn) return;
    const mv = this.ai.evaluatePosition(this.grid!.forAi());
    if (!this.aiIsOn) return;
    this.move(mv);
    setTimeout(() => this.runLoop(), 100);
  }
  // Return true if the game is lost, or has won and the user hasn't kept playing
  private isGameTerminated(): boolean {
    return this.over || (this.won && !this.keepPlaying);
  }
  // Set up the game
  public setup(): void {
    const previousState = this.storageManager.getGameState();
    // Reload the game from a previous game if present
    if (previousState) {
      this.grid = new Grid(previousState.grid); // Reload grid
      this.score = previousState.score;
      this.over = previousState.over;
      this.won = previousState.won;
      this.keepPlaying = previousState.keepPlaying;
    } else {
      this.grid = new Grid();
      this.score = 0;
      this.over = false;
      this.won = false;
      this.keepPlaying = false;
      this.aiIsOn = false;
      // Add the initial tiles
      this.addStartTiles();
    }
    this.setRunButton();
    this.update_strength();
    // Update the actuator
    this.actuate();
  }
  // Set up the initial tiles to start the game with
  private addStartTiles(): void {
    for (let i = 0; i < this.startTiles; i++) {
      this.addRandomTile();
    }
  }
  // Adds a tile in a random position
  private addRandomTile(): void {
    if (this.grid!.cellsAvailable()) {
      const value = Math.random() < 0.9 ? 2 : 4;
      const tile = new Tile(this.grid!.randomAvailableCell()!, value);
      this.grid!.insertTile(tile);
    }
  }
  // Sends the updated grid to the actuator
  private actuate(): void {
    if (this.storageManager.getBestScore() < this.score) {
      this.storageManager.setBestScore(this.score);
    }
    // Clear the state when the game is over (game over only, not win)
    if (this.over) {
      this.storageManager.clearGameState();
    } else {
      this.storageManager.setGameState(this.serialize());
    }
    this.actuator.actuate(this.grid!, {
      score: this.score,
      over: this.over,
      won: this.won,
      bestScore: this.storageManager.getBestScore(),
      terminated: this.isGameTerminated()
    });
  }
  // Represent the current game as an object
  private serialize(): GameState {
    return {
      grid: this.grid!.serialize(),
      score: this.score,
      over: this.over,
      won: this.won,
      keepPlaying: this.keepPlaying
    };
  }
  // Save all tile positions and remove merger info
  private prepareTiles(): void {
    this.grid!.eachCell((_x, _y, tile) => {
      if (tile) {
        tile.mergedFrom = null;
        tile.savePosition();
      }
    });
  }
  // Move a tile and its representation
  private moveTile(tile: Tile, cell: Position): void {
    const cells: any = this.grid!.cells;
    cells[tile.x][tile.y] = null;
    cells[cell.x][cell.y] = tile;
    tile.updatePosition(cell);
  }
  // Move tiles on the grid in the specified direction
  private move(direction: Direction): void {
    // 0: up, 1: right, 2: down, 3: left
    const self = this;
    if (this.isGameTerminated()) return; // Don't do anything if the game's over
    const vector = this.getVector(direction);
    const traversals = this.buildTraversals(vector);
    let moved = false;
    // Save the current tile positions and remove merger information
    this.prepareTiles();
    // Traverse the grid in the right direction and move tiles
    for (const x of traversals.x) {
      for (const y of traversals.y) {
        const cell: Position = { x: x, y: y };
        const tile = self.grid!.cellContent(cell);
        if (tile) {
          const positions = self.findFarthestPosition(cell, vector);
          const next = self.grid!.cellContent(positions.next);
          // Only one merger per row traversal?
          if (next && next.value === tile.value && !next.mergedFrom) {
            const merged = new Tile(positions.next, tile.value * 2);
            merged.mergedFrom = [tile, next];
            self.grid!.insertTile(merged);
            self.grid!.removeTile(tile);
            // Converge the two tiles' positions
            tile.updatePosition(positions.next);
            // Update the score
            self.score += merged.value;
            // The mighty 65536 tile
            if (merged.value === 65536) self.won = true;
          } else {
            self.moveTile(tile, positions.farthest);
          }
          if (!self.positionsEqual(cell, tile)) {
            moved = true; // The tile moved from its original cell!
          }
        }
      }
    }
    if (moved) {
      this.addRandomTile();
      if (!this.movesAvailable()) {
        this.over = true; // Game over!
        this.aiIsOn = false;
      }
      this.actuate();
    }
  }
  // Get the vector representing the chosen direction
  private getVector(direction: Direction): Vector {
    let map = new Map<Direction, Vector>([
      [Direction.Up, { x: 0, y: -1 }],
      [Direction.Right, { x: 1, y: 0 }],
      [Direction.Down, { x: 0, y: 1 }],
      [Direction.Left, { x: -1, y: 0 }]
    ]);
    return map.get(direction)!;
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
    cell: Position,
    vector: Vector
  ): { farthest: Position; next: Position } {
    let previous;
    // Progress towards the vector direction until an obstacle is found
    do {
      previous = cell;
      cell = { x: previous.x + vector.x, y: previous.y + vector.y };
    } while (this.grid!.withinBounds(cell) && this.grid!.cellAvailable(cell));
    return {
      farthest: previous,
      next: cell // Used to check if a merge is required
    };
  }
  private movesAvailable(): boolean {
    return this.grid!.cellsAvailable() || this.tileMatchesAvailable();
  }
  // Check for available matches between tiles (more expensive check)
  private tileMatchesAvailable(): boolean {
    const self = this;
    for (let x = 0; x < this.size; x++) {
      for (let y = 0; y < this.size; y++) {
        const tile = this.grid!.cellContent({ x: x, y: y });
        if (tile) {
          for (let direction = 0; direction < 4; direction++) {
            const vector = self.getVector(direction);
            const cell = { x: x + vector.x, y: y + vector.y };
            const other = self.grid!.cellContent(cell);
            if (other && other.value === tile.value) {
              return true; // These two tiles can be merged
            }
          }
        }
      }
    }
    return false;
  }
  private positionsEqual(first: Vector, second: Vector): boolean {
    return first.x === second.x && first.y === second.y;
  }
}
