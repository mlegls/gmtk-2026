## "To Nothing"

The world is ending, and there's nothing to do about it. A reverse metroidvania in a crumbling world, where you have to lose all your actions to win.

The map is an isometric 2.5d grid. Time is in turns - your move is what advances the world forward. The world is decaying, and you lose (restart) when the world ends. The player is a cube, who starts with 10 buttons:

- wasd - roll left/right/forward/back
- q/e/x - turn left/right/around
- z/c/space - idk yet

The world is arranged with 10 nodes, like the [tree of life](https://en.wikipedia.org/wiki/Sefirot). In each node there's an altar to sacrifice one of your buttons. The world is semi-open, but some regions are inaccessible without certain buttons. There's also a "true ending" you can only get to by sacrificing your buttons in a particular order - roughly the most useful first (the canonical reverse of the tree of life's "emanation" order, symbolizing a reverse cosmogenesis).

Some buttons are "redundant" in the sense of compositional equivalence. E.g., turn left + roll forward = roll left. But turns matter, because there's a global and some local turn timers, where parts of the world will collapse/change as the timer counts down. E.g., a bridge collapses, a chasm opens, entire parts of the world fall away into the void...

The BGM is in 10 layers for the 10 buttons. The layers disappear as you sacrifice your skills, reducing from something cheerful and "standard" to something much more solemn and melancholic as you're left with fewer and fewer skills towards the endgame.

Aesthetically, low poly 3d resembling antichamber, with thematic inspiration from classical ruins, nier automata, clair obscur, ruins in souls games (crumbling farum azula etc), ghost towns. The sense of somewhere civilized and familiar decaying away. The contrast between structure and chaos/nothingness. A sense of loss.

## Ideas/Maybes

- an action changes your active collision layer. the world has "dark" and "light" (or otherwise split) parts that you can toggle between
- certain terrain elements do the same thing as an action. for instance, a "mirror portal" that flips some polarity state each time you cross it. and you can do the same thing via an action while you have that action
- world evolution by a global timer, such that you have to wait until the right time to go certain places. planning across multiple runs, a la outer wilds
- gates that can only be crossed when you don't have certain actions anymore
- facing-sensitive terrain. activation/passibility only when facing a certain way on a block (think bloxorz). still unsure if the player should be 1x1x1 or 1x1x2
- a difference in what kind of structure "matters" as the world decays. e.g., earlier in the run, certain elements are sensitive to your facing. then later on your faces wear off into generic textures, and nothing is facing-sensitive anymore
