import { GameManager } from './game_manager.js';
import { KeyboardInputManager } from './keyboard_input_manager.js';
import { HTMLActuator } from './html_actuator.js';
import { LocalStorageManager } from './local_storage_manager.js';

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(function () {
  new GameManager(4, KeyboardInputManager, HTMLActuator, LocalStorageManager);
});
