import { GameManager } from './game_manager.js';
import '../style/main.scss';
import '../favicon.ico';

// Wait till the browser is ready to render the game (avoids glitches)
window.requestAnimationFrame(function () {
    new GameManager(4);
});
