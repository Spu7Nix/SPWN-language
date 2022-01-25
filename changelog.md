# Current Version: 0.8 beta

## Breaking Changes

- Keyword `switch` is now `match`
- Keyword `case` is removed (in favor of `==` patterns)
- Unary `..` operator is no longer allowed (for example `..10` now needs to be written as `0..10`)
- The `has` operator has been replaced with `in` (with the order flipped)

## New Features

- Return types for macros with `(arguments) -> return_type { ... }` syntax
- You can spread an array in another array with `..` syntax:

```rs
b = [3, 4]
$.assert([1, 2, ..b, 5] == [1, 2, 3, 4, 5])
```

- Zip arrays together with `*[...]` syntax:

```rs
a = [1, "a"]
b = [2, "b"]
c = [3, "c"]
$.assert([*[a, b, c]] == [[1, 2, 3], ["a", "b", "c"]])
```

- Destructuring additions, including dictionary destruction, and destruction of the syntax mentioned above (`..` and `*[...]`)
- **Big expansion to the pattern system**:
  - `is` operator for matching any value to a pattern (`10 is @number // true`)
  - using most boolean operators as unary operators now yield a pattern:
    - `==val` will match any value that equals `val`
    - `!=val` will match any value that does not equal `val`
    - `>val` will match any value that is greater than `val`
    - `<val` will match any value that is less than `val`
    - `>=val` will match any value that is greater than or equal to `val`
    - `<=val` will match any value that is less than or equal to `val`
    - `in val` will match any value that is in `val`
  - `&` operator for combining two patterns, so that the resulting pattern requires the value to match both patterns
  - `_` pattern, which matches any value (wildcard)
  - pattern ternary operator, which returns a value if it matches a pattern, and otherwise returns the default value:
  ```rs
  10 if is >5 else 0 // 10
  4 if is >5 else 0 // 0
  ```

## STD Library Features

- `@chroma` type for color values, this type is now used in for example color triggers instead of RGB arguments
- `level` global variable for reading the objects in the current level
- `@log` and `@runtime_log` types for debug logging to the console or at runtime
- Changes and improvements to many existing types (see the [docs](https://spu7nix.net/spwn/#/std-docs/std-docs))
