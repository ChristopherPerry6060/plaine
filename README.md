# Motivations
Excel is a not designed for long term record-keeping, it is even less intended
to be a system for collaboration. Those two things make up the majority of what
we actually do here. Nothing we do is very complicated, trying to maintain
our volume while relying on Excel is also far from trivial.

Plaine is an effort to make everyone's job boring, you could even say it is
meant to make things very plain. 

## Deterministic Vs. Declarative
Deterministic state refers to a system where the current state
can be uniquely determined by the sum of its previous states. In other words, 
the outcome of the system is completely predictable.

Declarative state, on the other hand, describes the current state of the system
to be the desired outcome, without relating to its previous iterations.

| Deterministic | Declarative |
| ------------- | ----------- |
| Apples: 0     | Apples: 0   |
| Apples: +2    | Apples: 2   |
| Apples: +3    | Apples: 5   |


Majority of our tools in the past have been declarative, we need to move towards
deterministic records.

