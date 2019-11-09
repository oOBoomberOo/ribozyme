# About
### What is 'Ribozyme'?
Ribozyme or Ribonucleic Acid Enzyme is an enzyme that catalyst a certain chemical reaction.
Ribozyme is a Minecraft Resourcepack Merger for merging datapack's resourcepacks together, written completely in rust.
Similarly, this program help speed up the process of merging resourcepacks together just like a catalyst.

And we totally pick this name on purpose and not because it's sound cool or anything.
### What 'Ribozyme' can do
Ribozyme is capable of:
- Merging model files
- Merging language files
- Merging font files
- Merging json files
- Merging `sound.json`
- Notify you when there are model's predicate conflicts
- Compress the combined resourcepack

Ribozyme can be use on both resourcepack folder and resourcepack.zip file.
### What 'Ribozyme' can't do
Ribozyme cannot merge item model of two resourcepack if they use the same custom model data.
To prevent this please follow our official convention about custom model in [MC datapacks discord server](https://discord.gg/rMnEDDQ)

Ribozyme cannot merge two different model that located at the same location.
To prevent this you should heavily use namespace in your resourcepack, For example instead of putting your model inside `/assets/minecraft/models/block/foo.json`, you should put it in `/assets/<your_name>/models/<datapack_name>/block/foo.json` instead.

Ribozyme cannot merge `image`/`text`/`sound` file at all, it will be completely override.
To prevent this refer to the previous method.

Ribozyme cannot merge block state file, It's such a pain in the ass.