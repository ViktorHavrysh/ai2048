import { Tile, SerializableTile } from "./tile";
import Position from "./position";

export interface SerializableGrid {
  cells: (SerializableTile | null)[][];
}

export class Grid {
  private readonly size: number = 4;
  public readonly cells: (Tile | null)[][];

  public constructor(previousState?: SerializableGrid) {
    this.cells = previousState ? this.fromState(previousState) : this.empty();
  }

  // Build a grid of the specified size
  private empty(): (Tile | null)[][] {
    const cells: (Tile | null)[][] = [];
    for (let x = 0; x < this.size; x++) {
      const row: (Tile | null)[] = [];
      cells[x] = row;
      for (let y = 0; y < this.size; y++) {
        row.push(null);
      }
    }
    return cells;
  }

  private fromState(state: SerializableGrid): (Tile | null)[][] {
    const cells: (Tile | null)[][] = [];
    for (let x = 0; x < this.size; x++) {
      const row: (Tile | null)[] = [];
      cells[x] = row;
      for (var y = 0; y < this.size; y++) {
        const tile = state.cells[x][y];
        if (tile) {
          row.push(new Tile(tile.position, tile.value));
        } else {
          row.push(null);
        }
      }
    }
    return cells;
  }

  // Find the first available random position
  public randomAvailableCell(): Position | null {
    const cells = this.availableCells();
    if (cells.length) {
      return cells[Math.floor(Math.random() * cells.length)];
    } else {
      return null;
    }
  }
  private availableCells(): Position[] {
    const cells: Position[] = [];
    this.eachCell((x, y, tile) => {
      if (!tile) {
        cells.push({ x: x, y: y });
      }
    });
    return cells;
  }
  // Call callback for every cell
  public eachCell(
    callback: ((x: number, y: number, tile: Tile | null) => void)
  ): void {
    for (let x = 0; x < this.size; x++) {
      for (let y = 0; y < this.size; y++) {
        callback(x, y, this.cells[x][y]);
      }
    }
  }
  // Check if there are any cells available
  public cellsAvailable(): boolean {
    return !!this.availableCells().length;
  }
  // Check if the specified cell is taken
  public cellAvailable(cell: Position): boolean {
    return !this.cellContent(cell);
  }
  public cellContent(position: Position): Tile | null {
    if (this.withinBounds(position)) {
      return this.cells[position.x][position.y];
    } else {
      return null;
    }
  }
  // Inserts a tile at its position
  public insertTile(tile: Tile): void {
    this.cells[tile.x][tile.y] = tile;
  }
  public removeTile(tile: Position): void {
    this.cells[tile.x][tile.y] = null;
  }
  public withinBounds(position: Position): boolean {
    return (
      position.x >= 0 &&
      position.x < this.size &&
      position.y >= 0 &&
      position.y < this.size
    );
  }
  public serialize(): SerializableGrid {
    const cellState: (SerializableTile | null)[][] = [];
    for (let x = 0; x < this.size; x++) {
      const row: (SerializableTile | null)[] = [];
      cellState[x] = row;
      for (var y = 0; y < this.size; y++) {
        const tile = this.cells[x][y];
        row.push(tile ? tile.serialize() : null);
      }
    }
    return {
      cells: cellState
    };
  }
  public forAi(): Uint32Array {
    const b: number[] = [];
    for (let row of this.cells) {
      for (let tile of row) {
        b.push(tile ? tile.value : 0);
      }
    }
    return new Uint32Array(b);
  }
}
