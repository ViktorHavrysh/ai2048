import { Tile, SerializableTile } from "./tile";
import Position from "./position";

export interface SerializableGrid {
  tiles: (SerializableTile | null)[][];
}

export class Grid {
  private readonly size: number = 4;
  public readonly tiles: (Tile | null)[][];

  public constructor(previousState?: SerializableGrid) {
    this.tiles = previousState ? this.fromState(previousState) : this.empty();
  }

  // Build a grid of the specified size
  private empty(): (Tile | null)[][] {
    const tiles: (Tile | null)[][] = [];
    for (let x = 0; x < this.size; x++) {
      const row: (Tile | null)[] = [];
      tiles[x] = row;
      for (let y = 0; y < this.size; y++) {
        row.push(null);
      }
    }
    return tiles;
  }

  private fromState(state: SerializableGrid): (Tile | null)[][] {
    const tiles: (Tile | null)[][] = [];
    for (let x = 0; x < this.size; x++) {
      const row: (Tile | null)[] = [];
      tiles[x] = row;
      for (var y = 0; y < this.size; y++) {
        const tile = state.tiles[x][y];
        if (tile) {
          row.push(new Tile(tile.position, tile.value));
        } else {
          row.push(null);
        }
      }
    }
    return tiles;
  }

  // Find the first available random position
  public randomAvailablePosition(): Position | null {
    const tiles = this.availablePositions();
    if (tiles.length) {
      return tiles[Math.floor(Math.random() * tiles.length)];
    } else {
      return null;
    }
  }
  private availablePositions(): Position[] {
    const cells: Position[] = [];
    this.eachTile((x, y, tile) => {
      if (!tile) {
        cells.push({ x: x, y: y });
      }
    });
    return cells;
  }
  // Call callback for every cell
  public eachTile(
    callback: ((x: number, y: number, tile: Tile | null) => void)
  ): void {
    for (let x = 0; x < this.size; x++) {
      for (let y = 0; y < this.size; y++) {
        callback(x, y, this.tiles[x][y]);
      }
    }
  }
  // Check if there are any cells available
  public tilesAvailable(): boolean {
    return !!this.availablePositions().length;
  }
  // Check if the specified cell is taken
  public tileAvailable(position: Position): boolean {
    return !this.tileAtPosition(position);
  }
  public tileAtPosition(position: Position): Tile | null {
    if (this.withinBounds(position)) {
      return this.tiles[position.x][position.y];
    } else {
      return null;
    }
  }
  // Inserts a tile at its position
  public insertTile(tile: Tile): void {
    this.tiles[tile.x][tile.y] = tile;
  }
  public removeTileAtPosition(tile: Position): void {
    this.tiles[tile.x][tile.y] = null;
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
        const tile = this.tiles[x][y];
        row.push(tile ? tile.serialize() : null);
      }
    }
    return {
      tiles: cellState
    };
  }
  public forAi(): Uint32Array {
    const b: number[] = [];
    for (let row of this.tiles) {
      for (let tile of row) {
        b.push(tile ? tile.value : 0);
      }
    }
    return new Uint32Array(b);
  }
}
