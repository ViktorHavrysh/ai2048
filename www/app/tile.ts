import Position from "./position";

export interface SerializableTile {
  position: Position;
  value: number;
}

export class Tile {
  public readonly value: number;
  public x: number;
  public y: number;
  public previousPosition: Position | null = null;
  public mergedFrom: Tile[] | null = null;
  public constructor(position: Position, value = 2) {
    this.x = position.x;
    this.y = position.y;
    this.value = value;
  }
  public savePosition(): void {
    this.previousPosition = { x: this.x, y: this.y };
  }
  public updatePosition(position: Position): void {
    this.x = position.x;
    this.y = position.y;
  }
  public serialize(): SerializableTile {
    return {
      position: {
        x: this.x,
        y: this.y
      },
      value: this.value
    };
  }
}
