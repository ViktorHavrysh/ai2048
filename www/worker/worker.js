import("../../ai2048-wasm/pkg").then(ai2048 => {
  ai2048.init();
  self.addEventListener("message", async ent => {
    const message = ent.data;
    let mv = ai2048.evaluate_position(
      message.grid,
      message.minProb,
      message.maxDepth
    );
    self.postMessage(mv);
  });
});
