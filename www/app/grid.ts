import Position from "./position";
import { SerializableTile, Tile } from "./tile";

export interface SerializableGrid {
  tiles: (SerializableTile | null)[][];
}

const Size = 4;

export class Grid {
  public readonly tiles: (Tile | null)[][];

  public constructor(previousState?: SerializableGrid) {
    this.tiles = previousState ? this.fromState(previousState) : this.empty();
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
  // Call callback for every tile
  public eachTile(
    callback: ((x: number, y: number, tile: Tile | null) => void)
  ): void {
    for (let x = 0; x < Size; x++) {
      for (let y = 0; y < Size; y++) {
        callback(x, y, this.tiles[x][y]);
      }
    }
  }
  // Check if there are any tiles available
  public tilesAvailable(): boolean {
    return !!this.availablePositions().length;
  }
  // Check if the specified tile is taken
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
      position.x < Size &&
      position.y >= 0 &&
      position.y < Size
    );
  }
  public serialize(): SerializableGrid {
    const state: (SerializableTile | null)[][] = [];
    for (let x = 0; x < Size; x++) {
      const row: (SerializableTile | null)[] = [];
      state[x] = row;
      for (let y = 0; y < Size; y++) {
        const tile = this.tiles[x][y];
        row.push(tile ? tile.serialize() : null);
      }
    }
    return {
      tiles: state
    };
  }
  public forAi(): Uint32Array {
    const b: number[] = [];
    for (const row of this.tiles) {
      for (const tile of row) {
        b.push(tile ? tile.value : 0);
      }
    }
    return new Uint32Array(b);
  }

  // Build a grid of the specified size
  private empty(): (Tile | null)[][] {
    const tiles: (Tile | null)[][] = [];
    for (let x = 0; x < Size; x++) {
      const row: (Tile | null)[] = [];
      tiles[x] = row;
      for (let y = 0; y < Size; y++) {
        row.push(null);
      }
    }
    return tiles;
  }

  private fromState(state: SerializableGrid): (Tile | null)[][] {
    const tiles: (Tile | null)[][] = [];
    for (let x = 0; x < Size; x++) {
      const row: (Tile | null)[] = [];
      tiles[x] = row;
      for (let y = 0; y < Size; y++) {
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
  private availablePositions(): Position[] {
    const positions: Position[] = [];
    this.eachTile((x, y, tile) => {
      if (!tile) {
        positions.push({ x: x, y: y });
      }
    });
    return positions;
  }
}
