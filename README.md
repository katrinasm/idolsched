
# idolsched: SIFAS schedule optimizer
A program to create play schedules for SIFAS.
Currently the term "schedule" is aspirational; it basically creates teams you can use for autoplay,
but in the future I hope to add new types of schedule featuring timings for swapping and SP activations.

# how use
The program needs two files, `api.json` and `account.json`, to run. You should probably never change `api.json`, it's basically just there because I felt like it would be rude to hardcode another person's website into my program. `account.json` contains info on your cards and accessories.
Most of the formatting for `account.json` should be obvious from the example file in this repository, except cards, which are formatted like so:
```{ "ordinal": 1, "lb": 0, "fed": false }```
where `ordinal` is the card's school idol number (you can see this by sorting your card list), `lb` is the card's limit break, and `fed` should be `true` if you have invested in the card's skill tree, `false` otherwise. Any card you put in your list is assumed to be at the maximum level for its rarity.

Note that when you are filling out your account data, it may be a good idea to skip cards you are particularly unlikely to use (like most Rs). The algorithm is highly randomized, and basically the more bad cards there are to sort through the longer it will take to find a good setup.

If your account contains less than 9 cards, it will be padded using the 27 starter Rs, at LB0 and unfed.

Once you have your account set up, you can basically just run `idolsched` from a command line to play around with it. It doesn't accept any argument to specify a song or anything because it currently only supports one song, an extremely stripped down version of No Exit Orion (Advanced difficulty), since I nor anyone else has datamined the kind of song info the program needs yet.

In order to ameliorate the current lack of songs, there is now an attribute override: you can run `idolsched -c1` for "NEO but Smile", `idolsched -c2` for "NEO but Pure", up through `idolsched -c6` for "NEO but Elegant".

If the program is giving you bad results, try `idolsched -n100000` or `idolsched -n1000000`, etc, to increase its runtime. If those do not help, let Katrina know I guess.

Most of the other options are boring technical stuff; you can learn about them with `idolsched --help`.

# missing features
Currently idolsched is missing a huge number of game features. Most notably:
- Support for any song that's not NEO Adv
- Support for most skills in the game (and by extension most buffs and debuffs)
- Support for ACs
- Insight skills
- the Kizuna board

Additionally, since it always assumes autoplay, player-activated behaviors like SPs and strategy swapping are missing.
The poor support for skills means the program is presently quite bad at choosing accessories, since so much of their impact comes from their skill.
The list of supported skills is much shorter than the list of unsupported skills:
- healing tap skills
- shielding tap skills
- Vo+ tap skills
- tap voltage up tap skills
- appeal up tap skills (important to note that actives, including start-of-song, start-of-AC, and the "own appeal" active that acts like a second tap skill, are missing)
- all non-insight passives (the code exists for insight passives but since there is no way to put an insight on a card it is useless)

In theory I could add support for most skills based on a previous version of the program relatively quickly, but that version ran roughly 100 times as slowly as the current version and the code was so bad that `rustc` actually warned me that being able to compile it was considered a bug so I am trying to optimize and organize things up-front this time. Apologies for the resulting delay; skills are pretty complicated.

Additionally, this program is experimental and extremely cavalier. It has bad error messages and probably a lot of other unsupported stuff I forgot to list here. Also I haven't bothered to extract the game's database for myself, so data not retrieved from Kirara is likely a bit inaccurate.

# how it made
Note: you can skip this section if you aren't a giganerd.

The current algorithm is a form of simulated annealing, treating schedules as states, and using -E[*voltage*] from a simulated live show as energy. It is mostly "vanilla" simulated annealing, except that since running a live is somewhat expensive, idolsched's annealer uses a cache of previously visited states' energy to avoid recalculating. In recent versions the cache hit rate has gone down substantially, so the cache may be removed in the future.

The "moves" allowed to transform a schedule are:
- exchanging a card from the green strategy with a card from one of the other strategies (this move is included instead of exchanging any two cards since exchanging two backliners has no significance for autoplay, currently the only thing idolsched is good for)
- replacing any card on the team with a card in the user's album that is not on the team
- replacing any accessory on the team with an accessory in the user's accessory inventory not on the team
- removing any accessory on the team (as you might expect, this move tends to produce bad results and will likely be removed in the future)

As an additional hack, if a live simulation ends by running out of stamina, the reported voltage is the voltage prior to running out of stamina divided by 10,000. This is not part of the annealer's code, but it is essential to the annealer's operation: searching starts with a randomized team, which usually dies quickly, and if every such team reports 0 voltage the annealer has no information it can use to find a usable team.
