import { Direction } from "./direction";

export default class InputManager {
  private readonly events: { [index: string]: ((data: any) => void)[] } = {};
  private readonly eventTouchstart: string;
  private readonly eventTouchmove: string;
  private readonly eventTouchend: string;
  private readonly map: { [index: number]: Direction | undefined } = {
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
  public constructor() {
    if (window.navigator.msPointerEnabled) {
      //Internet Explorer 10 style
      this.eventTouchstart = "MSPointerDown";
      this.eventTouchmove = "MSPointerMove";
      this.eventTouchend = "MSPointerUp";
    } else {
      this.eventTouchstart = "touchstart";
      this.eventTouchmove = "touchmove";
      this.eventTouchend = "touchend";
    }
    this.listen();
  }
  public on(event: string, callback: (data: any) => void): void {
    if (!this.events[event]) {
      this.events[event] = [];
    }
    this.events[event].push(callback);
  }
  private emit(event: string, data?: any): void {
    const callbacks = this.events[event];
    if (callbacks) {
      for (const callback of callbacks) {
        callback(data);
      }
    }
  }
  private listen(): void {
    const self = this;
    // Respond to direction keys
    document.addEventListener("keydown", event => {
      const modifiers =
        event.altKey || event.ctrlKey || event.metaKey || event.shiftKey;
      const mapped = this.map[event.which];
      if (!modifiers) {
        if (mapped) {
          event.preventDefault();
          self.emit("move", mapped);
        }
      }
      // R key restarts the game
      if (!modifiers && event.which === 82) {
        self.restart.call(self, event);
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
    gameContainer.addEventListener(this.eventTouchmove, event => {
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
        self.emit("move", direction);
      }
    });
  }
  private restart(event: Event) {
    event.preventDefault();
    this.emit("restart");
  }
  private run(event: Event) {
    event.preventDefault();
    this.emit("run");
  }
  private plus(event: Event) {
    event.preventDefault();
    this.emit("plus");
  }
  private minus(event: Event) {
    event.preventDefault();
    this.emit("minus");
  }
  private keepPlaying(event: Event) {
    event.preventDefault();
    this.emit("keepPlaying");
  }
  private bindButtonPress(selector: string, fn: (event: Event) => void) {
    var button = document.querySelector(selector)!;
    button.addEventListener("click", fn.bind(this));
    button.addEventListener(this.eventTouchend, fn.bind(this));
  }
}
