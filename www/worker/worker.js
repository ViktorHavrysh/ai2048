import registerPromiseWorker from "promise-worker/register";

const mod = import("../../ai2048-wasm/pkg").then(m => {
  m.init();
  return m;
});

registerPromiseWorker(async message => {
  const ai = await mod;
  const mv = ai.evaluate_position(
    message.grid,
    message.minProb,
    message.maxDepth
  );
  return mv;
});
