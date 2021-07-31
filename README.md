# firstname-chooser

[![Build status](https://github.com/desbma/firstname-chooser/actions/workflows/ci.yml/badge.svg)](https://github.com/desbma/firstname-chooser/actions)
[![License](https://img.shields.io/github/license/desbma/firstname-chooser.svg?style=flat)](https://github.com/desbma/firstname-chooser/blob/master/LICENSE)

> There are only two hard things in Computer Science: cache invalidation and naming things.
>
> -- Phil Karlton

Expecting a child, and feeling overwhelmed by the task of finding a name that he/she will bear for the rest of his/her life?

I have you covered. Actually Rust & graph theory do.

## Overview

- This is a tool to explore name ideas interactively, rather than pick a single name for you
- `firstname-chooser` suggests names, **you** rate them and the next suggestions will be taylored according to your previous choices
- No machine learning or opaque algorithms here, just good old graph theory
- Choices are stored on filesystem, so you can stop and resume later

## Algorithm explanation

- Build a graph where nodes are names, and edges are [Levenshtein distances](https://en.wikipedia.org/wiki/Levenshtein_distance) between names
- The first name is picked randomly in the graph
- Pick the next name by maximizing or minimizing distance to names previously chosen, depending on whether they were liked or not
- "commonness factor": this is a float value in the [0.0; 1.0] range to tweak behavior around name commonness (according to previous years statistics)
  - 0 will only consider your previous choices ignoring name commonness
  - 1 will ignore your choices and suggest names only according to their commonness
  - obviously values in between will consider both, to a degree that depends on the value
  - in my experience the sweet spot is between 0.4 and 0.5.

## Implementation

- Graph in memory is a 2D vector of distances, to represent a "half matrix" (distance between a and b is the same as between b and a, so no need to store it twice)
- Graph is built using a thread pool for performance
- Once the graph construction is fast enough, most of the tuning is actually in the optional source filtering, ie:
  - computing name weighting from source data
  - removing names given before year X
  - removing compound names
  - removing too short names

## Data source

For now this only contains a [source for French names](https://www.insee.fr/fr/statistiques/2540004?sommaire=4767262).

Before adding other languages or data source, I should probably add a `NameSource` trait. PR welcome if you care enough.

## License

[GPLv3](https://www.gnu.org/licenses/gpl-3.0-standalone.html)
