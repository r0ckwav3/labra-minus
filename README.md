# labra-minus

An esolang inspired by [labra](https://esolangs.org/wiki/Labra), which tries to remove some of the operations which I found extraneous. I ended up with only using the characters `[] ()` and digits, but I had to do some slightly strange formatting to fit all of the neccesary applied (2-input) operations. This repo includes a full description of the syntax as well as an interpreter.

labra-minus uses two types: lists and numbers. Programs mainly consist of brackets and use a lot of list manipulation.

You may notice that there is no way to define a function in this language, and two operations that use functions. Those two operations (Induction and Map) interpret the codeblock in their brackets as a function and feed the input into `()`, the input operator.

## Using the interpreter

Put a file `XXX.txt` in the root directory of the project and run:
```
cargo run -- XXX.txt
or
cargo run -- XXX.txt input
```
Inputs can only be integers at the moment. If no input is given, 0 is the default input.

## Syntax Overview
Newlines are ignored, and `#` makes a comment.

Expressions in labra-minus are always of the four possible forms (where `{` represents either bracket):
* Number: some combination of digits
  * Note that "-" is not a valid character, so negative numbers have to be made with the subtraction operator
* Nullary: `{}`
* Unary: `X{}`
* Applied: `X{Y}`
  * Note that in this form the opening and closing bracket may be of different types.

```
()    Input          - Used by Induction and Map. Can also be used to take the user's input.
[]    Empty list     - The empty list.
X()   Length/Abs     - Gives the length of a list, or the absolute value of a number.
X[]   Encapsulate    - A list containing X.
X(Y)  Addition       - Adds numbers and concatenates lists. Throws an error on mismatched types.
X[Y]  Index/Subtract - The Yth element of the list X or X - Y. Throws an error if Y is a list.
X(Y]  Induction      - Returns the lazy-evaluated list {X, Y(X), Y(Y(X)), ...}.
X[Y)  Map            - Returns {Y(X[0]), Y(X[1]), ...}.
```

## Example Code
#### Test for equality to 0
Returns 1 if input is 0 (or length 0), 0 otherwise.

```
1(0][()()]
```

#### Accumulate/Flatten a list
Assumes that `()` is a list containing all numbers or all lists.
```
# We input some list l
0[](()[0][])(()[])
# A list containing {0, l[0], l}
(
    ()[0](1)[]
    (()[1](()[2][()[0](1)])[])
    (()[2][])
]
# A list L such that L[i] = {i, sum from l[0] to l[i], l}
[()()[1]]
# L[l.size()-1] = {l.size()-1, sum of l, l}
[1]
# extract sum of l
```
Since we are lazy evaluating the induction operator, it's fine if some of the elements of it are invalid as long as we don't access them (for example computing `L[l.size()]` would access an out-of-bounds element).

## Todo
 * improve user input
    * string i/o
    * Arbitrary list
 * Better Error Messages
    * Line/Character numbers
 * Code cleanup
   * Import organization
 * Try converting Box pointers to normal pointers
