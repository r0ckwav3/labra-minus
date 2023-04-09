# labra-minus

An esolang inspired by [labra](https://esolangs.org/wiki/Labra), which tries to remove some of the operations which I found extraneous. I ended up with only using the characters `[] ()` and digits, but I had to do some slightly strange formatting to fit all of the neccesary Applied (2-input) operations. Labra-minus uses two types: lists and numbers. Programs mainly consist of brackets and use a lot of list manipulation. This repo will eventually have a compiler for labra (coded in rust), but for now it just has the syntax documented below.

You may notice that there is no way to define a function in this language, and two operations that use functions. Those two operations (Induction and Map) interpret the codeblock in their brackets as a function and feed the input into `()`, the input operator.

The program can accept a number or a string as input, which will be translated into a list of numbers.

## Syntax Overview

Expressions in labra are always of the four possible forms (where `{` represents either bracket):
* Number: some combination of digits
  * Note that "-" is not a valid character, so negative numbers have to be made with the negation operator
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
