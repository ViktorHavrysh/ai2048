import GameManager from "game_manager";
import { Direction } from "./direction";

const KeyMap: { [index: number]: Direction } = {
  38: Direction.Up,
  39: Direction.Right,
  40: Direction.Down,
  37: Direction.Left,
  75: Direction.Up,
  76: Direction.Right,
  74: Direction.Down,
  72: Direction.Left,
  87: Direction.Up,
  68: Direction.Right,
  83: Direction.Down,
  65: Direction.Left // A
};

export default class InputManager {
  private readonly gameManager: GameManager;
  private readonly eventTouchstart: string;
  private readonly eventTouchmove: string;
  private readonly eventTouchend: string;

  public constructor(gameManager: GameManager) {
    this.gameManager = gameManager;
    if (window.navigator.msPointerEnabled) {
      // Internet Explorer 10 style
      this.eventTouchstart = "MSPointerDown";
      this.eventTouchmove = "MSPointerMove";
      this.eventTouchend = "MSPointerUp";
    } else {
      this.eventTouchstart = "touchstart";
      this.eventTouchmove = "touchmove";
      this.eventTouchend = "touchend";
    }
  }
  public listen(): void {
    // Respond to direction keys
    document.addEventListener("keydown", event => {
      const modifiers =
        event.altKey || event.ctrlKey || event.metaKey || event.shiftKey;
      const mapped = KeyMap[event.which];
      if (!modifiers) {
        if (mapped) {
          event.preventDefault();
          this.gameManager.move(mapped);
        }
      }
      // R key restarts the game
      if (!modifiers && event.which === 82) {
        this.restart(event);
      }
    });
    // Respond to button presses
    this.bindButtonPress(".retry-button", this.restart);
    this.bindButtonPress(".restart-button", this.restart);
    this.bindButtonPress(".run-button", this.run);
    this.bindButtonPress(".plus-button", this.plus);
    this.bindButtonPress(".minus-button", this.minus);
    this.bindButtonPress(".keep-playing-button", this.keepPlaying);
    // Respond to swipe events
    let touchStartClientX: number, touchStartClientY: number;
    const gameContainer = document.getElementsByClassName("game-container")[0];
    gameContainer.addEventListener(this.eventTouchstart, (event: any) => {
      if (
        (!window.navigator.msPointerEnabled && event.touches.length > 1) ||
        event.targetTouches.length > 1
      ) {
        return; // Ignore if touching with more than 1 finger
      }
      if (window.navigator.msPointerEnabled) {
        touchStartClientX = event.pageX;
        touchStartClientY = event.pageY;
      } else {
        touchStartClientX = event.touches[0].clientX;
        touchStartClientY = event.touches[0].clientY;
      }
      event.preventDefault();
    });
    gameContainer.addEventListener(this.eventTouchmove, (event: Event) => {
      event.preventDefault();
    });
    gameContainer.addEventListener(this.eventTouchend, (event: any) => {
      if (
        (!window.navigator.msPointerEnabled && event.touches.length > 0) ||
        event.targetTouches.length > 0
      ) {
        return; // Ignore if still touching with one or more fingers
      }
      let touchEndClientX: number, touchEndClientY: number;
      if (window.navigator.msPointerEnabled) {
        touchEndClientX = event.pageX;
        touchEndClientY = event.pageY;
      } else {
        touchEndClientX = event.changedTouches[0].clientX;
        touchEndClientY = event.changedTouches[0].clientY;
      }
      const dx = touchEndClientX - touchStartClientX;
      const absDx = Math.abs(dx);
      const dy = touchEndClientY - touchStartClientY;
      const absDy = Math.abs(dy);
      if (Math.max(absDx, absDy) > 10) {
        let direction: Direction;
        if (absDx > absDy) {
          direction = dx > 0 ? Direction.Right : Direction.Left;
        } else {
          direction = dy > 0 ? Direction.Down : Direction.Up;
        }
        this.gameManager.move(direction);
      }
    });
  }
  private restart(event: Event) {
    event.preventDefault();
    this.gameManager.restart();
  }
  private run(event: Event) {
    event.preventDefault();
    this.gameManager.toggleAi();
  }
  private plus(event: Event) {
    event.preventDefault();
    this.gameManager.plus();
  }
  private minus(event: Event) {
    event.preventDefault();
    this.gameManager.minus();
  }
  private keepPlaying(event: Event) {
    event.preventDefault();
    this.gameManager.continuePlaying();
  }
  private bindButtonPress(selector: string, fn: (event: Event) => void) {
    const button = document.querySelector(selector)!;
    button.addEventListener("click", fn.bind(this));
    button.addEventListener(this.eventTouchend, fn.bind(this));
  }
}
