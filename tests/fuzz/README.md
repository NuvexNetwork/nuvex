# Fuzzing

No fuzz target is wired. Instruction bytes, account bytes, proof bytes, and callback config become targets when those parsers exist. An empty harness would not exercise them.
