import { C2W, W2C, Fin } from "session-typed-worker";

type AiMessanging = C2W<Uint32Array, number, number, W2C<number, Fin>>;

export { AiMessanging };
