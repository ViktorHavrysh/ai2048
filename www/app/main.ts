import "./style/main.scss";
import "./favicon.ico";

import InputManager from "./input_manager";
import StorageManager from "./local_storage_manager";
import { HTMLActuator as Actuator } from "./html_actuator";
import GameManager from "./game_manager";
import Ai from "./ai";

const minProb = 0.0001;
const initialStrength = 8;

function init() {
  const storageManager = new StorageManager();
  const ai = new Ai(minProb, initialStrength);
  const actuator = new Actuator();
  const gameManager = new GameManager(storageManager, actuator, ai);
  gameManager.setup();
  const inputManager = new InputManager(gameManager);
  inputManager.listen();
}

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(init);
