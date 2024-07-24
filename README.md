# CuYAt

CuYAt stands for `Curb Your Attitude` and is intended to be a game to help understand
[attitude](https://en.wikipedia.org/wiki/Spacecraft_attitude_control).

It is currently in a very rudimentary shape: a lot of work is needed to get it
to be educational (let alone funny).

The idea is that you control a spacecrafts attitude and your objective is to make
it point as needed with the least possible number of moves.

So far, there is only one mode for playing.
The screen is split into two.
On the left you see the stars through the window. On the right, the needed attitude (target).
Your mission is to make left be as close as you can to right.
For that, you use your keyboard like this:

| **key** | **action**     |
|-----|--------------------|
| r/R | do a roll          |
| p/P | do a pitch         |
| y/Y | do a yaw           |
| z/Z | zoom               |
| s/S | scale              |
| d   | show/hide distance |
| space | score this game and start another |
| q | end playing the game |

- See definitions of [Roll, Pitch and Yaw](https://en.wikipedia.org/wiki/Aircraft_principal_axes).
- Zoom makes your window narrower/wider (as if it was the zoom of a camera)
- Scale is the step with which the spacecraft moves. The bigger the scale, the fastest you will rotate it.

The score at the end is the average of the individual scores of each game you played.
The goal is to get the smallest score possible.
The score in each game increases with the number of moves that you make and
decreases with the distance to the target that you reach.
