const ctx: Worker = self as any;

const ai2048 = import("../ai2048-wasm/pkg").then(m => {
  console.log("test");
  m.init();
  return m;
});

ctx.addEventListener("message", async ent => {
  const message = ent.data;
  console.log(ent);
  let ai = await ai2048;
  let mv = ai.evaluate_position(
    message.grid,
    message.minProb,
    message.maxDepth
  );
  ctx.postMessage(mv);
});
export default null as any;
