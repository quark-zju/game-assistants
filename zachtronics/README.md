Solver for a subset of [Zachtronics](https://www.zachtronics.com/) games.

## [Last Call BBS](https://www.zachtronics.com/last-call-bbs/)

### Dungeons & Diagrams

Check the files in `data/dungeons` for the input format. Example run:

```
$ cargo run dungeons < data/dungeons/45135238.in
Selected game: dungeons
Found solution:
#.......
#.######
........
##.#.###
....####
#.#.....
#...#.##
#.###..#
Searches: 195
```

## [Solitaire Collection](https://www.zachtronics.com/solitaire-collection/)

### Cjul

Check the files in `data/cjul` for the input format. Example run:

```
$ cargo run --release cjul < data/cjul/7.in
...
Step   1. Move [9] from 1 -> 5. [T 9 8 T 8 9] to [K D 10 K 9 6].
Step   2. Move [8] from 1 -> 3. [T 9 8 T 8] to [V 10 6 V T 9].
...
Step  47. Move [K D V 10 9 8 7 6] from 5 -> 1. [K D V 10 9 8 7 6] to [T].
Step  48. Move [V 10 9 8 7 6] from 3 -> 4. [V 10 9 8 7 6] to [T K D].
```

Some heuristics are used. The solver does not optimize for step count.

### Cribbage

Check the files in `data/cribbage` for the input format. Example run:

```
% time cargo run --release cribbage < data/cribbage/a.in
Selected game: cribbage
Best Score: 112
 1.   0 Take    [4 6 Q K]
        Columns [1 1 1 1]
 2.   2 Take    [J 2 10 8]
        Columns [4 1 2 4]
...
11. 104 Take    [10 10 A K]
        Columns [1 2 1 1]
12. 112 Take    [Q Q Q]
        Columns [3 4 4]
Searched states: 28121. Cache hit: 17667307.
cargo run --release cribbage < data/cribbage/a.in  2.05s user 0.01s system 99% cpu 2.068 total
```

Finding the optimal solution is computationally expensive. You can set `Q` (quality) to balance computation time and solution quality. Example:

```
% time Q=3 cargo run --release cribbage < data/cribbage/a.in
Selected game: cribbage
Score: 79
 1.   8 Take    [4 6 6 6 3]
        Columns [1 1 3 3 3]
...
12.  79 Take    [10 A K]
        Columns [1 1 1]
Searched states: 802. Cache hit: 4223.
Search Quality ($Q): 3. May miss better solutions.
Q=3 cargo run --release cribbage < data/cribbage/a.in  0.03s user 0.02s system 98% cpu 0.049 total
```
