# simple-math

# Abandoned

`simple-math` is a badly-named messy piece of crap that doesn't even support compiling to AST before executing.

Alternatives:

- For a binary, check out [math-test-haskell](https://github.com/jD91mZM2/math-test-haskell).
  Even though Haskell is probably slower, for this particular project I don't really care.
- For a library, check out [calc](https://github.com/redox-os/calc), maintained by the Redox team
  and used within the ion shell.

-------------------------

# Original README

I was bored, so I made some simple math parser and calculator.

- [x] Arbitrary-length ("big") numbers. (Thanks to library "num")
- [x] Binary/Octal/Hexadecimal numbers
- [x] Bitwise operators
- [x] Factorial
- [x] Function system
- [x] Negative numbers
- [x] Non-whole numbers. (Thanks to library "bigdecimal-rs")
- [x] Orders of operations
- [ ] Actually implement some functions

----------------------------------

EDIT: Ugh, I can't even make a single project without [@tbodt](https://github.com/tbodt) knowing a better solution :P  
This time he told me about recursive parsers :O  
So yeah, huge thanks to him for being such an awesome person!

# simple-math vs GNU bc

After a lot of development, this actually turned out to be a pretty cool project.  
Let's compare it to GNU `bc`!

`simple-math` cons:
- New technology. Therefore it's currently less stable.
- Power using `pow(x, y)`, not `x^y`. This is due to the `^` operator doing something else.
- `bc` is more powerful.
- Probably some more things I don't know about.

`simple-math` pros:
- Supports factorial built-in.
- No need to struggle with `scale=`.
- Auto-inserts times where needed (e.g. `2(2 + 2)` is `8`).
- Supports bitwise operators and bitshifting.
