import { Tile } from "./tile";
import { Grid } from "./grid";
import Position from "./position";

export interface ActuatorMetadata {
  score: number;
  over: boolean;
  won: boolean;
  bestScore: number;
  terminated: boolean;
  strength: number;
  aiIsOn: () => boolean;
}

export class HTMLActuator {
  private readonly tileContainer = document.querySelector(".tile-container")!;
  private readonly scoreContainer = document.querySelector(".score-container")!;
  private readonly bestContainer = document.querySelector(".best-container")!;
  private readonly strengthContainer = document.querySelector(
    ".strength-container"
  )!;
  private readonly runButton = document.querySelector(".run-button")!;
  private readonly messageContainer = document.querySelector(".game-message")!;
  private score: number = 0;
  public actuate(grid: Grid, metadata: ActuatorMetadata): Promise<void> {
    const self = this;
    return new Promise((resolve, _reject) => {
      window.requestAnimationFrame(() => {
        self.clearContainer(self.tileContainer);
        for (const column of grid.tiles) {
          for (const cell of column) {
            if (cell) {
              self.addTile(cell);
            }
          }
        }
        self.updateScore(metadata.score);
        self.updateBestScore(metadata.bestScore);
        self.updateStrength(metadata.strength);
        self.updateRunButton(metadata.aiIsOn());
        if (metadata.terminated) {
          if (metadata.over) {
            self.message(false); // You lose
          } else if (metadata.won) {
            self.message(true); // You win!
          }
        }
        resolve();
      });
    });
  }
  // Continues the game (both restart and keep playing)
  public continueGame(): void {
    this.clearMessage();
  }
  private clearContainer(container: Element): void {
    while (container.firstChild) {
      container.removeChild(container.firstChild);
    }
  }
  private addTile(tile: Tile): void {
    const self = this;
    const wrapper = document.createElement("div");
    const inner = document.createElement("div");
    const position = tile.previousPosition || { x: tile.x, y: tile.y };
    const positionClass = this.positionClass(position);
    // We can't use classlist because it somehow glitches when replacing classes
    const classes = ["tile", "tile-" + tile.value, positionClass];
    if (tile.value > 2048) classes.push("tile-super");
    this.applyClasses(wrapper, classes);
    inner.classList.add("tile-inner");
    inner.textContent = tile.value.toString();
    if (tile.previousPosition) {
      // Make sure that the tile gets rendered in the previous position first
      window.requestAnimationFrame(() => {
        classes[2] = self.positionClass({ x: tile.x, y: tile.y });
        self.applyClasses(wrapper, classes); // Update the position
      });
    } else if (tile.mergedFrom) {
      classes.push("tile-merged");
      this.applyClasses(wrapper, classes);
      // Render the tiles that merged
      for (const merged of tile.mergedFrom) {
        self.addTile(merged);
      }
    } else {
      classes.push("tile-new");
      this.applyClasses(wrapper, classes);
    }
    // Add the inner part of the tile to the wrapper
    wrapper.appendChild(inner);
    // Put the tile on the board
    this.tileContainer.appendChild(wrapper);
  }
  private applyClasses(element: Element, classes: string[]): void {
    element.setAttribute("class", classes.join(" "));
  }
  private normalizePosition(position: Position): Position {
    return { x: position.x + 1, y: position.y + 1 };
  }
  private positionClass(position: Position): string {
    position = this.normalizePosition(position);
    return "tile-position-" + position.x + "-" + position.y;
  }
  private updateScore(score: number): void {
    this.clearContainer(this.scoreContainer);
    const difference = score - this.score;
    this.score = score;
    this.scoreContainer.textContent = this.score.toString();
    if (difference > 0) {
      const addition = document.createElement("div");
      addition.classList.add("score-addition");
      addition.textContent = "+" + difference;
      this.scoreContainer.appendChild(addition);
    }
  }
  private updateBestScore(bestScore: number): void {
    this.bestContainer.textContent = bestScore.toString();
  }
  public updateStrength(strength: number): void {
    this.strengthContainer.textContent = strength.toString();
  }
  public updateRunButton(aiIsOn: boolean): void {
    if (!aiIsOn) {
      this.runButton.textContent = "Start AI";
    } else {
      this.runButton.textContent = "Stop AI";
    }
  }
  private message(won: boolean): void {
    const type = won ? "game-won" : "game-over";
    const message = won ? "You win!" : "Game over!";
    this.messageContainer.classList.add(type);
    this.messageContainer.getElementsByTagName("p")[0].textContent = message;
  }
  private clearMessage(): void {
    // IE only takes one value to remove at a time.
    this.messageContainer.classList.remove("game-won");
    this.messageContainer.classList.remove("game-over");
  }
}
