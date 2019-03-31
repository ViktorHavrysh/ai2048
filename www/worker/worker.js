import registerPromiseWorker from "promise-worker/register";

const mod = import("../../ai2048-wasm/pkg").then(m => {
  m.init();
  return m;
});

registerPromiseWorker(async message => {
  const ai = await mod;
  return ai.evaluate_position(message.grid, message.minProb);
});
