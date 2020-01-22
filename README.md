# Overview

This library is designed to rank randomizer racers. It's based on the Glicko-2
rating system with some experimental modifications. You can read about Mark
Glickman's Glicko-2 system [here](http://www.glicko.net/glicko/glicko2.pdf).
The multiplayer implementation was partially borrowed from [ms2300's implementation](https://github.com/ms2300/multiplayer-glicko2).

# Usage

You can install RandoRank as a python package with pip by running
`pip install randorank`. The first thing you'll want to do is determine a
period length(e.g. four weeks) and a set of constants to use for your
game/category.

To use this library to rank randomizer players, you can instantiate a
`MultiPeriod` class, set the system variables with the `add_constants()`
method, add races with the `add_race()` method, then export a dictionary of
players with their ranking with the `rank()` method. When a period has
concluded, you can feed this dict into a new MultiPeriod instance with the
`add_players()` method and continue ranking.

## Setting Constants and Multiplayer Glicko Implementation

The MultiPeriod class will initialize with a default set of constants
designed for the A Link to the Past Randomizer. To set your own constants,
create a dict with the following keys:

```python
example_constants = {'tau': .02,
                     'multi_slope': .008,
                     'multi_cutoff': 8,
                     'norm_factor': 1.3
                     'initial_rating': 1500,
                     'initial_deviation': 300,
                     'initial_volatility': .22}
```
a MultiPeriod has an attribute `MultiPeriod.constants` with these values as a dict.
There are also `set_[constant name]` methods for each of these individually.
**tau** is the Glicko system constant. For randomizer races, it should be set
low, around .02. **multi\_slope**, **multi\_cutoff**, and **norm\_factor** are
part of the multiplayer implementation, which divides races into a series of
1v1 matches and applies a weight based on race size and how close two racers
finished compared to each other and the distribution of times in the race.

The cutoff determines how many runners a race must have for the library
to use the multiplayer implementation. Below that, it will use the stock
Glicko formula. The normalization factor is used to determine a "floor" time
for the race and is game-specific. The formula for scoring a normalized race
is first\_quartile + (IQR * norm\_factor). Times in between are assigned a
value between 0 and 1, 1 being the first place finisher. To determine a
normalization factor for your game or category, you should experiment with
different values for multiple races and use your best judgement to figure out
which value gives the best floor time. Since this formula is less accurate
with less racers, we use the cutoff to prevent it from skewing final rankings.
The slope is used later as part of the formula for determining a 1v1's weight.

In races above the cutoff, we take the original weight and multiply it by
the result of the following: 
```
let normed_diff = the absolute value of the difference between the two players'
normalized scores

1 - (multi_slope * (size ** (1 - abs(normed_diff)))) * (1 / (1 - multi_slope))
```

This generally means that the bigger the race, and the closer the two runners'
finish times, the lower the weight for their 1v1. But no matter the race size,
top finishers' scores against the very bottom will remain the same.

To determine the best variables for your game or category, you'll want to look
at the sorted results over several periods as well as the distribution of final
ratings. Make sure the final rankings look reasonably accurate that. The
distribution of scores should be somewhere between normal and skewed right.
Experiment using different values for the same data set.

## Adding Races

Races are passed to the `add_race()` method as a dictionary with names as keys
and times (in seconds) as values. If a runner does not finish a race, their
value should be NaN. You can use `nan` values in python by importing the type
from the math module(`from math import nan`). Suppose you want to add a race
with three runners, one of whom forfeited. You would pass this dictionary:

```python
example_race = {'runner 1': 1563,
                'runner 2': 1620,
                'runner 3': nan}
```
It's important that you convert all non-finishers to NaN and don't use a number
like 0. NaN is a special numerical type indicating that the value is "not a
number."

## End of Period Rankings

Using the `rank()` method of the MultiPeriod instance will export a dictionary
of dictionaries containing each runner's rating, deviation, volatility, inactive periods,
variance, and delta for the end of the period. The latter three values can be
discarded if you're done ranking players. If you want to continue ranking over
multiple periods, you can pass this to the `add_players()` method of a new
MultiPeriod instance.

# Practical Considerations

If you want to continuously calculate rankings throughout a period, it's
important that you always use players' pre-period variables, re-calculate the
entire period by adding all the races again, and do not pass a ranking dict
back to the same instance. You can, however, continue to add races and call
`rank()` again.

*(Experimental)* If you want to calculate new mid-period scores given a set
of new races that haven't been added yet, say from a database for example, 
you can pass `False` to the `rank()` method, instantiate a new MultiPeriod,
combine the **pre-period** rating, deviation and volatility with the **new**
delta and variance values, add only the players who have participated in the
new races with `add_players()` then the races themselves with `add_races()`.
