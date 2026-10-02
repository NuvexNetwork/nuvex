const CALLBACK_IX_DISCRIMINATOR = Uint8Array.from([245, 250, 10, 62, 218, 252, 239, 91]);

export interface CallbackConfig {
  output: Uint8Array;
  data: Uint8Array;
}

export function buildCallbackInstruction(config: CallbackConfig): Uint8Array {
  if (config.output.length !== 64) {
    throw new Error("VRF output must be 64 bytes");
  }
  if (config.data.length > 128) {
    throw new Error("callback data exceeds 128 bytes");
  }
  const data = new Uint8Array(8 + 64 + 4 + config.data.length);
  data.set(CALLBACK_IX_DISCRIMINATOR, 0);
  data.set(config.output, 8);
  new DataView(data.buffer).setUint32(72, config.data.length, true);
  data.set(config.data, 76);
  return data;
}
