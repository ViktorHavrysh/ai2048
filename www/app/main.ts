import "./favicon.ico";
import "./style/main.scss";

import Ai from "./ai";
import GameManager from "./game_manager";
import { HTMLActuator as Actuator } from "./html_actuator";
import InputManager from "./input_manager";
import StorageManager from "./local_storage_manager";

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
