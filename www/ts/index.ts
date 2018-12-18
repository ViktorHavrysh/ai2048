import InputManager from "./input_manager";
import StorageManager from "./local_storage_manager";
import { HTMLActuator as Actuator } from "./html_actuator";
import { GameManager } from "./game_manager";
import Ai from "./ai";

import "../style/main.scss";
import "../favicon.ico";

const minProb = 0.0001;
const maxDepth = 6;

let gameManager;

function init() {
  const inputManager = new InputManager();
  const storageManager = new StorageManager();
  const actuator = new Actuator();
  const ai = new Ai(inputManager, minProb, maxDepth);
  gameManager = new GameManager(inputManager, storageManager, actuator, ai);
  gameManager.setup();
}

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(init);
