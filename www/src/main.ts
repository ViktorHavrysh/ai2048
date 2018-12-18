import "../style/main.scss";
import "../favicon.ico";

import InputManager from "./input_manager";
import StorageManager from "./local_storage_manager";
import { HTMLActuator as Actuator } from "./html_actuator";
import { GameManager } from "./game_manager";
import Ai from "./ai";
import EventManager from "./event_manager";

const minProb = 0.0001;
const maxDepth = 6;

function init() {
  const eventManager = new EventManager();
  const inputManager = new InputManager(eventManager);
  const storageManager = new StorageManager();
  const actuator = new Actuator(eventManager);
  const ai = new Ai(eventManager, minProb, maxDepth);
  const gameManager = new GameManager(
    eventManager,
    storageManager,
    actuator,
    ai
  );
  inputManager.listen();
  gameManager.setup();
}

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(init);
