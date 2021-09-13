# List of Built-in functions
## $.abs
> ## Description:
> Calculates the absolute value of a number <div>
> ## Example:
> ```spwn
> $.assert($.abs(-100) == 100)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.acos
> ## Description:
> Calculates the arccos of a number <div>
> ## Example:
> ```spwn
> $.acos(-1)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.acosh
> ## Description:
> Calculates the arccosh of a number <div>
> ## Example:
> ```spwn
> $.acosh(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.add
> ## Description:
> Adds a Geometry Dash object or trigger to the target level <div>
> ## Example:
> ```spwn
> extract obj_props
> $.add(obj {
>     OBJ_ID: 1,
>     X: 45,
>     Y: 45,
> })
> ```
> **Allowed by default:** yes
> ## Arguments: 
>  **The object or trigger to add**
## $.append
> ## Description:
> Appends a value to the end of an array. You can also use `array.push(value)` <div>
> ## Example:
> ```spwn
> let arr = []
> $.append(arr, 1)
> $.assert(arr == [1])
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `arr` | _mutable_ _Array_ |
> | 2 | `val` |  |
## $.asin
> ## Description:
> Calculates the arcsin of a number <div>
> ## Example:
> ```spwn
> $.asin(1)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.asinh
> ## Description:
> Calculates the arcsinh of a number <div>
> ## Example:
> ```spwn
> $.asinh(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.assert
> ## Description:
> Throws an error if the argument is not `true` <div>
> ## Example:
> ```spwn
> $.assert(true)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `b` | _Bool_ |
## $.atan
> ## Description:
> Calculates the arctan of a number <div>
> ## Example:
> ```spwn
> $.atan(1)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.atan2
> ## Description:
> Calculates the arctan^2 of a number <div>
> ## Example:
> ```spwn
> $.atan2(a, b)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `x` | _Number_ |
> | 2 | `y` | _Number_ |
## $.atanh
> ## Description:
> Calculates the arctanh of a number <div>
> ## Example:
> ```spwn
> $.atanh(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.b64decode
> ## Description:
> Returns the input string decoded from base64 encoding (useful for text objects) <div>
> ## Example:
> ```spwn
> $.b64decode("aGVsbG8gdGhlcmU=")
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `s` | _Str_ |
## $.b64encode
> ## Description:
> Returns the input string encoded with base64 encoding (useful for text objects) <div>
> ## Example:
> ```spwn
> $.b64encode("hello there")
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `s` | _Str_ |
## $.cbrt
> ## Description:
> Calculates the cube root of a number <div>
> ## Example:
> ```spwn
> $.assert($.cbrt(27) == 3)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.ceil
> ## Description:
> Calculates the ceil of a number, AKA the number rounded up to the nearest integer <div>
> ## Example:
> ```spwn
> $.assert($.ceil(1.5) == 2)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.cos
> ## Description:
> Calculates the cos of an angle in radians <div>
> ## Example:
> ```spwn
> $.cos(3.1415)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.cosh
> ## Description:
> Calculates the cosh of a number <div>
> ## Example:
> ```spwn
> $.cosh(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.display
> ## Description:
> returns the value display string for the given value <div>
> ## Example:
> ```spwn
> $.display(counter()) // "counter(?i, bits = 16)"
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
## $.edit\_obj
> ## Description:
> Changes the value of an object key. You can also use `object.set(key, value)` <div>
> ## Example:
> ```spwn
> $.edit_obj(object, ROTATION, 180)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `o, m` | _mutable_ _Obj_ |
> | 2 | `key` |  |
> | 3 | `value` |  |
## $.exp
> ## Description:
> Calculates the e^x of a number <div>
> ## Example:
> ```spwn
> $.exp(x)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.exp2
> ## Description:
> Calculates the 2^x of a number <div>
> ## Example:
> ```spwn
> $.assert($.exp2(10) == 1024)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.exp\_m1
> ## Description:
> Calculates e^x - 1 in a way that is accurate even if the number is close to zero <div>
> ## Example:
> ```spwn
> $.exp_m1(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.extend\_trigger\_func
> ## Description:
> Executes a macro in a specific trigger function context <div>
> ## Example:
> ```spwn
> $.extend_trigger_func(10g, () {
>     11g.move(10, 0, 0.5) // will add a move trigger in group 10
> })
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `group` |  |
> | 2 | `mac` | _Macro_ |
## $.floor
> ## Description:
> Calculates the floor of a number, AKA the number rounded down to the nearest integer <div>
> ## Example:
> ```spwn
> $.assert($.floor(1.5) == 1)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.fract
> ## Description:
> Gets the fractional part of a number <div>
> ## Example:
> ```spwn
> $.assert($.fract(123.456) == 0.456)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.get\_input
> ## Description:
> Gets some input from the user <div>
> ## Example:
> ```spwn
> inp = $.get_input()
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `prompt` | _Str_ |
## $.http\_request
> ## Description:
> Sends a HTTP request <div>
> **Allowed by default:** no
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `method` | _Str_ |
> | 2 | `url` | _Str_ |
> | 3 | `headers` | _Dict_ |
> | 4 | `body` | _Str_ |
## $.hypot
> ## Description:
> Calculates the hypothenuse in a right triangle with sides a and b <div>
> ## Example:
> ```spwn
> $.assert($.hypot(3, 4) == 5) // because 3^2 + 4^2 = 5^2
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.ln
> ## Description:
> Calculates the ln (natural log) of a number <div>
> ## Example:
> ```spwn
> $.ln(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.log
> ## Description:
> Calculates the log base x of a number <div>
> ## Example:
> ```spwn
> $.log(num, base)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
> | 2 | `base` | _Number_ |
## $.matches
> ## Description:
> Returns `true` if the value matches the pattern, otherwise it returns `false` <div>
> ## Example:
> ```spwn
> $.matches([1, 2, 3], [@number])
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `val` |  |
> | 2 | `pattern` |  |
## $.max
> ## Description:
> Calculates the max of two numbers <div>
> ## Example:
> ```spwn
> $.assert($.max(1, 2) == 2)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.min
> ## Description:
> Calculates the min of two numbers <div>
> ## Example:
> ```spwn
> $.assert($.min(1, 2) == 1)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.mutability
> ## Description:
> Checks if a value reference is mutable <div>
> ## Example:
> ```spwn
> const = 1
> $.assert(!$.mutability(const))
> let mut = 1
> $.assert($.mutability(mut))
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `var` |  |
## $.pop
> ## Description:
> Removes a value from the end of an array, and returns it. You can also use `array.pop()` <div>
> ## Example:
> ```spwn
> let arr = [1, 2, 3]
> $.assert($.pop(arr) == 3)
> $.assert(arr == [1, 2])
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `arr` | _mutable_  |
## $.print
> ## Description:
> Prints value(s) to the console <div>
> ## Example:
> ```spwn
> $.print("Hello world!")
> ```
> **Allowed by default:** yes
> ## Arguments: 
>  **any**
## $.random
> ## Description:
> Generates random numbers, or picks a random element of an array <div>
> ## Example:
> ```spwn
> $.random() // a completely random number
> $.random([1, 2, 3, 6]) // returns either 1, 2, 3, or 6
> $.random(1, 10) // returns a random integer between 1 and 10
> ```
> **Allowed by default:** yes
> ## Arguments: 
>  **see example**
## $.readfile
> ## Description:
> Returns the contents of a file in the local system <div>
> ## Example:
> ```spwn
> data = $.readfile("file.txt")
> ```
> **Allowed by default:** no
> ## Arguments: 
>  **Path of file to read, and the format it's in ("text", "bin", "json", "toml" or "yaml")**
## $.regex
> ## Description:
> Performs a regex operation on a string <div>
> **Allowed by default:** yes
> ## Arguments: 
>  **`mode` can be either "match", "replace" or "findall"**
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `regex` | _Str_ |
> | 2 | `s` | _Str_ |
> | 3 | `mode` | _Str_ |
> | 4 | `replace` |  |
## $.remove\_index
> ## Description:
> Removes a specific value from an array. You can also use `array.remove(index)` <div>
> ## Example:
> ```spwn
> $.remove_index(names, 2)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `arr` | _mutable_  |
> | 2 | `index` | _Number_ |
## $.round
> ## Description:
> Rounds a number <div>
> ## Example:
> ```spwn
> $.assert($.round(1.2) == 1)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.sin
> ## Description:
> Calculates the sin of an angle in radians <div>
> ## Example:
> ```spwn
> $.sin(3.1415)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.sinh
> ## Description:
> Calculates the hyperbolic sin of a number <div>
> ## Example:
> ```spwn
> $.sinh(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.split\_str
> ## Description:
> Returns an array from the split string. You can also use `string.split(delimiter)` <div>
> ## Example:
> ```spwn
> $.assert($.split_str("1,2,3", ",") == ["1", "2", "3"])
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `s` | _Str_ |
> | 2 | `substr` | _Str_ |
## $.spwn\_version
> ## Description:
> Gets the current version of spwn <div>
> ## Example:
> ```spwn
> $.spwn_version()
> ```
> **Allowed by default:** yes
> ## Arguments: 
>  **none**
## $.sqrt
> ## Description:
> Calculates the square root of a number <div>
> ## Example:
> ```spwn
> $.sqrt(2)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.substr
> ## Description:
> Returns a specified part of the input string <div>
> ## Example:
> ```spwn
> $.substr("hello there", 1, 5)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `val` | _Str_ |
> | 2 | `start_index` | _Number_ |
> | 3 | `end_index` | _Number_ |
## $.tan
> ## Description:
> Calculates the tan of an angle in radians <div>
> ## Example:
> ```spwn
> $.tan(3.1415)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.tanh
> ## Description:
> Calculates the hyperbolic tan of a number <div>
> ## Example:
> ```spwn
> $.tanh(num)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.time
> ## Description:
> Gets the current system time in seconds <div>
> ## Example:
> ```spwn
> now = $.time()
> ```
> **Allowed by default:** yes
> ## Arguments: 
>  **none**
## $.trigger\_fn\_context
> ## Description:
> Returns the start group of the current trigger function context <div>
> ## Example:
> ```spwn
> $.trigger_fn_context()
> ```
> **Allowed by default:** yes
> ## Arguments: 
>  **none**
## $.writefile
> ## Description:
> Writes a string to a file in the local system (any previous content will be overwritten, and a new file will be created if it does not already exist) <div>
> ## Example:
> ```spwn
> $.write_file("file.txt", "Hello")
> ```
> **Allowed by default:** no
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `path` | _Str_ |
> | 2 | `data` | _Str_ |
# Default Implementations for Operators
## $.\_add\_
> ## Description:
> Default implementation of the `+=` operator <div>
> ## Example:
> ```spwn
> $._add_(val, 10)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` |  |
## $.\_and\_
> ## Description:
> Default implementation of the `&&` operator <div>
> ## Example:
> ```spwn
> $._and_(true, true)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Bool_ |
> | 2 | `b` | _Bool_ |
## $.\_as\_
> ## Description:
> Default implementation of the `as` operator <div>
> ## Example:
> ```spwn
> $._as_(1000, @string)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `t` | _TypeIndicator_ |
## $.\_assign\_
> ## Description:
> Default implementation of the `=` operator <div>
> ## Example:
> ```spwn
> $._assign_(val, 64)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` |  |
## $.\_decrement\_
> ## Description:
> Default implementation of the `n--` operator <div>
> ## Example:
> ```spwn
> _decrement_(n)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_display\_
> ## Description:
> returns the default value display string for the given value <div>
> ## Example:
> ```spwn
> $._display_(counter()) // "@counter::{ item: ?i, bits: 16 }"
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
## $.\_divide\_
> ## Description:
> Default implementation of the `/=` operator <div>
> ## Example:
> ```spwn
> $._divide_(val, 3)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_divided\_by\_
> ## Description:
> Default implementation of the `/` operator <div>
> ## Example:
> ```spwn
> $._divided_by_(64, 8)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_either\_
> ## Description:
> Default implementation of the `|` operator <div>
> ## Example:
> ```spwn
> $._either_(@number, @counter)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_equal\_
> ## Description:
> Default implementation of the `==` operator <div>
> ## Example:
> ```spwn
> $._equal_("hello", "hello")
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_exponate\_
> ## Description:
> Default implementation of the `^=` operator <div>
> ## Example:
> ```spwn
> $._exponate_(val, 3)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_has\_
> ## Description:
> Default implementation of the `has` operator <div>
> ## Example:
> ```spwn
> $._has_([1,2,3], 2)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_increment\_
> ## Description:
> Default implementation of the `n++` operator <div>
> ## Example:
> ```spwn
> $._increment_(n)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_intdivide\_
> ## Description:
> Default implementation of the `/%=` operator <div>
> ## Example:
> ```spwn
> $._intdivide_(val, 3)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_intdivided\_by\_
> ## Description:
> Default implementation of the `/%` operator <div>
> ## Example:
> ```spwn
> $._intdivided_by_(64, 8)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_less\_or\_equal\_
> ## Description:
> Default implementation of the `<=` operator <div>
> ## Example:
> ```spwn
> $._less_or_equal_(100, 100)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_less\_than\_
> ## Description:
> Default implementation of the `<` operator <div>
> ## Example:
> ```spwn
> $._less_than_(50, 100)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_minus\_
> ## Description:
> Default implementation of the `-` operator <div>
> ## Example:
> ```spwn
> $._minus_(128, 64)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_mod\_
> ## Description:
> Default implementation of the `%` operator <div>
> ## Example:
> ```spwn
> $._mod_(70, 8)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_modulate\_
> ## Description:
> Default implementation of the `%=` operator <div>
> ## Example:
> ```spwn
> $._modulate_(val, 3)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_more\_or\_equal\_
> ## Description:
> Default implementation of the `>=` operator <div>
> ## Example:
> ```spwn
> $._more_or_equal_(100, 100)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_more\_than\_
> ## Description:
> Default implementation of the `>` operator <div>
> ## Example:
> ```spwn
> $._more_than_(100, 50)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_multiply\_
> ## Description:
> Default implementation of the `*=` operator <div>
> ## Example:
> ```spwn
> $._multiply_(val, 10)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` | _Number_ |
## $.\_negate\_
> ## Description:
> Default implementation of the `-n` operator <div>
> ## Example:
> ```spwn
> $._negate_(n)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
## $.\_not\_
> ## Description:
> Default implementation of the `!b` operator <div>
> ## Example:
> ```spwn
> $._not_(b)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Bool_ |
## $.\_not\_equal\_
> ## Description:
> Default implementation of the `!=` operator <div>
> ## Example:
> ```spwn
> $._not_equal_("hello", "bye")
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_or\_
> ## Description:
> Default implementation of the `||` operator <div>
> ## Example:
> ```spwn
> $._or_(true, false)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Bool_ |
> | 2 | `b` | _Bool_ |
## $.\_plus\_
> ## Description:
> Default implementation of the `+` operator <div>
> ## Example:
> ```spwn
> $._plus_(32, 32)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_pow\_
> ## Description:
> Default implementation of the `^` operator <div>
> ## Example:
> ```spwn
> $._pow_(8, 2)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_pre\_decrement\_
> ## Description:
> Default implementation of the `--n` operator <div>
> ## Example:
> ```spwn
> $._pre_decrement_(n)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_pre\_increment\_
> ## Description:
> Default implementation of the `++n` operator <div>
> ## Example:
> ```spwn
> $._pre_increment_(n)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_range\_
> ## Description:
> Default implementation of the `..` operator <div>
> ## Example:
> ```spwn
> $._range_(0, 10)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `val_a` |  |
> | 2 | `b` | _Number_ |
## $.\_subtract\_
> ## Description:
> Default implementation of the `-=` operator <div>
> ## Example:
> ```spwn
> $._subtract_(val, 10)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_swap\_
> ## Description:
> Default implementation of the `<=>` operator <div>
> ## Example:
> ```spwn
> $._swap_(a, b)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` | _mutable_  |
## $.\_times\_
> ## Description:
> Default implementation of the `*` operator <div>
> ## Example:
> ```spwn
> $._times_(8, 8)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` | _Number_ |
## $.\_unary\_range\_
> ## Description:
> Default implementation of the `..n` operator <div>
> ## Example:
> ```spwn
> $._unary_range_(n)
> ```
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
