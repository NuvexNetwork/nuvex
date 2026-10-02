# Adversarial

The LiteSVM tests reject a forged VRF proof, a late VRF key, a node without the VRF capability, a second fulfill, an expired request, a callback that returns an error, a fulfill without stake or a fresh heartbeat, a withdraw before cooldown, and a slash above the locked stake.

Still checklist only:

- substituted PDA
- second reward claim, once a reward instruction exists
- callback that is given oracle accounts
- unknown job discriminant on fulfill of a non-VRF request, which the program also rejects if one is ever stored
