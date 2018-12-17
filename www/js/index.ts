import '../style/main.scss';
import '../favicon.ico';
import { GameManager } from './game_manager.js';
import { InputManager } from './input_manager.js';
import { HTMLActuator as Actuator } from './html_actuator.js';
import { LocalStorageManager as StorageManager } from './local_storage_manager.js';
import { Ai } from './ai.js'; import { Grid } from './grid.js';

const size = 4;
const minProb = 0.0001;
const maxDepth = 6;

let gameManager;

function init() {
    let inputManager = new InputManager;
    let storageManager = new StorageManager;
    let actuator = new Actuator;
    let ai = new Ai(inputManager, minProb, maxDepth);
    gameManager = new GameManager(size, inputManager, storageManager, actuator, ai);
}

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(init);
