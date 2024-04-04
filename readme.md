A nontrivial math game written in ggez as a Rust language exercise.

To run the game, just clone the repository and execute ```cargo run``` in the base directory.

**Rules**
The player will be presented with a "board" of five numbers. The four on the bottom are "input" or "hotbar" numbers. The one on top is the "target". Using any four combinations of any of the four basic arithmetic operators (addition, subtraction, multiplication, and division), make the four inputs total the target value.

**Controls**
Number keys 1-4 are used to move "hotbar slots" to the "workbench". The "1" key corresponds to the leftmost "hotbar" slot. The 2 key corresponds to the second from the left, and so on and so on.

Arithmetic operators can be selected by pressing the corresponding keys:
Plus (+)
Minus (-)
Multiply (x -or- *)
Divide (/)

Backspace will undo your most recent action.

Enter will attempt to compute the values that have been set in the workbench. The computed value will be moved to the leftmost workbench slot for further manipulation.

The board is completed when all four hotbar slots have been consumed and the remaining number is equivalent to the specified target. On completion, a message is printed to the console and a new board configuration is loaded.

**Notes**
Both the game interaction and game data generation have been implemented in the same repository. The ```data_generator``` module (util/data_generator.rs) generates valid input states (rsc/data/difficulty_pools.json) to later be consumed by the main game code.