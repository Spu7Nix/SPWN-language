# List of Built-in functions
## $.abs
> ## Description:
> Calculates the absolute value of a number<div>
> ## Example:
> ```spwn
> $.assert($.abs(-100) == 100)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.acos
> ## Description:
> Calculates the arccos of a number<div>
> ## Example:
> ```spwn
> $.acos(-1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.acosh
> ## Description:
> Calculates the arccosh of a number<div>
> ## Example:
> ```spwn
> $.acosh(1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.add
> ## Description:
> Adds a Geometry Dash object or trigger to the target level<div>
> ## Example:
> ```spwn
> 
> extract obj_props
> $.add(obj {
>     OBJ_ID: 1,
>     X: 45,
>     Y: 45,
> })
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> **The object or trigger to add**
## $.append
> ## Description:
> Appends a value to the end of an array. You can also use `array.push(value)`<div>
> ## Example:
> ```spwn
> 
> let arr = []
> $.append(arr, 1)
> $.assert(arr == [1])
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | arr | _mutable_ _Array_ |
> | val |  |
## $.asin
> ## Description:
> Calculates the arcsin of a number<div>
> ## Example:
> ```spwn
> $.asin(1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.asinh
> ## Description:
> Calculates the arcsinh of a number<div>
> ## Example:
> ```spwn
> $.asinh(0)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.assert
> ## Description:
> Throws an error if the argument is not `true`<div>
> ## Example:
> ```spwn
> $.assert(true)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | b | _Bool_ |
## $.atan
> ## Description:
> Calculates the arctan of a number<div>
> ## Example:
> ```spwn
> $.atan(1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.atan2
> ## Description:
> Calculates the arctan^2 of a number<div>
> ## Example:
> ```spwn
> $.atan2(0, -1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | x | _Number_ |
> | y | _Number_ |
## $.atanh
> ## Description:
> Calculates the arctanh of a number<div>
> ## Example:
> ```spwn
> $.atanh(0.996)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.b64decode
> ## Description:
> Returns the input string decoded from base64 encoding (useful for text objects)<div>
> ## Example:
> ```spwn
> $.b64decode("aGVsbG8gdGhlcmU=")
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | s | _Str_ |
## $.b64encode
> ## Description:
> Returns the input string encoded with base64 encoding (useful for text objects)<div>
> ## Example:
> ```spwn
> $.b64encode("hello there")
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | s | _Str_ |
## $.cbrt
> ## Description:
> Calculates the cube root of a number<div>
> ## Example:
> ```spwn
> $.cbrt(27)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.ceil
> ## Description:
> Calculates the ceil of a number, AKA the number rounded up to the nearest integer<div>
> ## Example:
> ```spwn
> $.assert($.ceil(1.5) == 2)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.cos
> ## Description:
> Calculates the cos of an angle in radians<div>
> ## Example:
> ```spwn
> $.cos(3.1415)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.cosh
> ## Description:
> Calculates the cosh of a number<div>
> ## Example:
> ```spwn
> $.cosh(0)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.display
> ## Description:
> returns the value display string for the given value<div>
> ## Example:
> ```spwn
> $.display(counter()) // "counter(?i, bits = 16)"
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $.edit_obj
> ## Description:
> Changes the value of an object key. You can also use `object.set(key, value)`<div>
> ## Example:
> ```spwn
> 
> extract obj_props
> let object = color_trigger(BG, 0, 0, 0, 0.5)
> $.edit_obj(object, X, 600)
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | o, m | _mutable_ _Obj_ |
> | key |  |
> | value |  |
## $.exp
> ## Description:
> Calculates the e^x of a number<div>
> ## Example:
> ```spwn
> $.exp(5) // e^5
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.exp2
> ## Description:
> Calculates the 2^x of a number<div>
> ## Example:
> ```spwn
> $.assert($.exp2(10) == 1024)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.exp_m1
> ## Description:
> Calculates e^x - 1 in a way that is accurate even if the number is close to zero<div>
> ## Example:
> ```spwn
> $.exp_m1(0.002)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.extend_trigger_func
> ## Description:
> Executes a macro in a specific trigger function context<div>
> ## Example:
> ```spwn
> 
> $.extend_trigger_func(10g, () {
>     11g.move(10, 0, 0.5) // will add a move trigger in group 10
> })
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | group |  |
> | mac | _Macro_ |
## $.floor
> ## Description:
> Calculates the floor of a number, AKA the number rounded down to the nearest integer<div>
> ## Example:
> ```spwn
> $.assert($.floor(1.5) == 1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.fract
> ## Description:
> Gets the fractional part of a number<div>
> ## Example:
> ```spwn
> $.fract(1.23)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.get_input
> ## Description:
> Gets some input from the user<div>
> ## Example:
> ```spwn
> // inp = $.get_input('What is your name?')
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | prompt | _Str_ |
## $.hash
> ## Description:
> Calculates the numerical hash of a value<div>
> ## Example:
> ```spwn
> $.hash("hello")
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n |  |
## $.http_request
> ## Description:
> Sends an HTTP request<div>
> ## Example:
> ```spwn
> 
> ```
> **Allowed by default:** false
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | method | _Str_ |
> | url | _Str_ |
> | headers | _Dict_ |
> | body | _Str_ |
## $.hypot
> ## Description:
> Calculates the hypothenuse in a right triangle with sides a and b<div>
> ## Example:
> ```spwn
> $.assert($.hypot(3, 4) == 5) // because 3^2 + 4^2 = 5^2
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $.level_objects
> ## Description:
> Returns a array of the objects in the level being written to, or an empty array if there is no output level<div>
> ## Example:
> ```spwn
> level = $.level_objects()
> ```
> **Allowed by default:** true
> ## Arguments: 
## $.level_string
> ## Description:
> Returns the level string of the level being written to, or nothing if there is no output level<div>
> ## Example:
> ```spwn
> level_string = $.level_string()
> ```
> **Allowed by default:** true
> ## Arguments: 
## $.ln
> ## Description:
> Calculates the ln (natural log) of a number<div>
> ## Example:
> ```spwn
> $.ln(2.71828)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.log
> ## Description:
> Calculates the log base x of a number<div>
> ## Example:
> ```spwn
> $.assert($.log(1024, 2) == 10)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
> | base | _Number_ |
## $.max
> ## Description:
> Calculates the max of two numbers<div>
> ## Example:
> ```spwn
> $.assert($.max(1, 2) == 2)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $.min
> ## Description:
> Calculates the min of two numbers<div>
> ## Example:
> ```spwn
> $.assert($.min(1, 2) == 1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $.mutability
> ## Description:
> Checks if a value reference is mutable<div>
> ## Example:
> ```spwn
> 
> const = 1
> $.assert(!$.mutability(const))
> let mut = 1
> $.assert($.mutability(mut))
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | var |  |
## $.pop
> ## Description:
> Removes a value from the end of an array, and returns it. You can also use `array.pop()`<div>
> ## Example:
> ```spwn
> 
> let arr = [1, 2, 3]
> $.assert($.pop(arr) == 3)
> $.assert(arr == [1, 2])
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | arr | _mutable_  |
## $.print
> ## Description:
> Prints value(s) to the console<div>
> ## Example:
> ```spwn
> $.print("Hello world!")
> ```
> **Allowed by default:** true
> ## Arguments: 
> **any**
## $.random
> ## Description:
> Generates random numbers, or picks a random element of an array<div>
> ## Example:
> ```spwn
> 
> $.random() // a completely random number
> $.random([1, 2, 3, 6]) // returns either 1, 2, 3, or 6
> $.random(1..11) // returns a random integer between 1 and 10
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> **see example**
## $.readfile
> ## Description:
> Returns the contents of a file in the local system (uses the current directory as base for relative paths)<div>
> ## Example:
> ```spwn
> data = $.readfile("file.txt")
> ```
> **Allowed by default:** false
> ## Arguments: 
> **Path of file to read, and the format it's in ("text", "bin", "json", "toml" or "yaml")**
## $.regex
> ## Description:
> Performs a regex operation on a string<div>
> ## Example:
> ```spwn
> 
> ```
> **Allowed by default:** true
> ## Arguments: 
> **`mode` can be either "match", "replace", "find_all" or "find_groups"**
> | **Name** | **Type** |
> |-|-|
> | regex | _Str_ |
> | s | _Str_ |
> | mode | _Str_ |
> | replace |  |
## $.remove_index
> ## Description:
> Removes a specific value from an array, string or dictionary. You can also use `array.remove(index)` or `dict.remove(key)`<div>
> ## Example:
> ```spwn
> 
> let names = ['James', 'Sophia', 'Romulus', 'Remus', 'Tiberius']
> $.remove_index(names, 2)
> $.assert(names == ['James', 'Sophia', 'Remus', 'Tiberius'])
> 
> let name_age = {
>     'Sophia': 34, 
>     'Romulus': 14, 
>     'Remus': 15, 
> }
> $.remove_index(name_age, 'Romulus')
> $.assert(name_age == {
>     'Sophia': 34, 
>     'Remus': 15, 
> })
>     
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | arr | _mutable_  |
> | index |  |
## $.round
> ## Description:
> Rounds a number<div>
> ## Example:
> ```spwn
> $.assert($.round(1.2) == 1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.sin
> ## Description:
> Calculates the sin of an angle in radians<div>
> ## Example:
> ```spwn
> $.sin(3.1415)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.sinh
> ## Description:
> Calculates the hyperbolic sin of a number<div>
> ## Example:
> ```spwn
> $.sinh(0)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.split_str
> ## Description:
> Returns an array from the split string. You can also use `string.split(delimiter)`<div>
> ## Example:
> ```spwn
> $.assert($.split_str("1,2,3", ",") == ["1", "2", "3"])
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | s | _Str_ |
> | substr | _Str_ |
## $.spwn_version
> ## Description:
> Gets the current version of spwn<div>
> ## Example:
> ```spwn
> $.spwn_version()
> ```
> **Allowed by default:** true
> ## Arguments: 
> **none**
## $.sqrt
> ## Description:
> Calculates the square root of a number<div>
> ## Example:
> ```spwn
> $.sqrt(2)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.substr
> ## Description:
> Returns a specified part of the input string<div>
> ## Example:
> ```spwn
> $.substr("hello there", 1, 5)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | val | _Str_ |
> | start_index | _Number_ |
> | end_index | _Number_ |
## $.tan
> ## Description:
> Calculates the tan of an angle in radians<div>
> ## Example:
> ```spwn
> $.tan(3.1415)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.tanh
> ## Description:
> Calculates the hyperbolic tan of a number<div>
> ## Example:
> ```spwn
> $.tanh(0.549)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | n | _Number_ |
## $.time
> ## Description:
> Gets the current system time in seconds<div>
> ## Example:
> ```spwn
> now = $.time()
> ```
> **Allowed by default:** true
> ## Arguments: 
> **none**
## $.trigger_fn_context
> ## Description:
> Returns the start group of the current trigger function context<div>
> ## Example:
> ```spwn
> $.trigger_fn_context()
> ```
> **Allowed by default:** true
> ## Arguments: 
> **none**
## $.writefile
> ## Description:
> Writes a string to a file in the local system (any previous content will be overwritten, and a new file will be created if it does not already exist)<div>
> ## Example:
> ```spwn
> $.write_file("file.txt", "Hello")
> ```
> **Allowed by default:** false
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | path | _Str_ |
> | data | _Str_ |
# Default Implementations for Operators
## $._add_
> ## Description:
> Default implementation of the `+=` operator<div>
> ## Example:
> ```spwn
> let val = 25
> $._add_(val, 10)
> $.assert(val == 35)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_  |
> | b |  |
## $._and_
> ## Description:
> Default implementation of the `&&` operator<div>
> ## Example:
> ```spwn
> $._and_(true, true)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Bool_ |
> | b | _Bool_ |
## $._as_
> ## Description:
> Default implementation of the `as` operator<div>
> ## Example:
> ```spwn
> $._as_(1000, @string)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | t | _TypeIndicator_ |
## $._assign_
> ## Description:
> Default implementation of the `=` operator<div>
> ## Example:
> ```spwn
> let val = 0
> $._assign_(val, 64)
> $.assert(val == 64)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_  |
> | b |  |
## $._both_
> ## Description:
> Default implementation of the `&` operator<div>
> ## Example:
> ```spwn
> $._both_(@number, @counter)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b |  |
## $._decrement_
> ## Description:
> Default implementation of the `n--` operator<div>
> ## Example:
> ```spwn
> let n = 1
> $._decrement_(n)
> $.assert(n == 0)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
## $._display_
> ## Description:
> returns the default value display string for the given value<div>
> ## Example:
> ```spwn
> $._display_(counter()) // "@counter::{ item: ?i, bits: 16 }"
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._divide_
> ## Description:
> Default implementation of the `/=` operator<div>
> ## Example:
> ```spwn
> let val = 9
> $._divide_(val, 3)
> $.assert(val == 3)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
> | b | _Number_ |
## $._divided_by_
> ## Description:
> Default implementation of the `/` operator<div>
> ## Example:
> ```spwn
> $._divided_by_(64, 8)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._either_
> ## Description:
> Default implementation of the `|` operator<div>
> ## Example:
> ```spwn
> $._either_(@number, @counter)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b |  |
## $._eq_pattern_
> ## Description:
> Default implementation of the `==a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._eq_pattern_(10))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._equal_
> ## Description:
> Default implementation of the `==` operator<div>
> ## Example:
> ```spwn
> $._equal_("hello", "hello")
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b |  |
## $._exponate_
> ## Description:
> Default implementation of the `^=` operator<div>
> ## Example:
> ```spwn
> let val = 3
> $._exponate_(val, 3)
> $.assert(val == 27)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
> | b | _Number_ |
## $._in_
> ## Description:
> Default implementation of the `in` operator<div>
> ## Example:
> ```spwn
> $._in_(2, [1,2,3])
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b |  |
## $._in_pattern_
> ## Description:
> Default implementation of the `in a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._in_pattern_([8, 10, 12]))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._increment_
> ## Description:
> Default implementation of the `n++` operator<div>
> ## Example:
> ```spwn
> let n = 0
> $._increment_(n)
> $.assert(n == 1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
## $._intdivide_
> ## Description:
> Default implementation of the `/%=` operator<div>
> ## Example:
> ```spwn
> let val = 10
> $._intdivide_(val, 3)
> $.assert(val == 3)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
> | b | _Number_ |
## $._intdivided_by_
> ## Description:
> Default implementation of the `/%` operator<div>
> ## Example:
> ```spwn
> $._intdivided_by_(64, 8)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._is_
> ## Description:
> Default implementation of the `is` operator<div>
> ## Example:
> ```spwn
> $._is_([1, 2, 3], [@number])
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | val |  |
> | pattern |  |
## $._less_or_eq_pattern_
> ## Description:
> Default implementation of the `<=a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._less_or_eq_pattern_(10))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._less_or_equal_
> ## Description:
> Default implementation of the `<=` operator<div>
> ## Example:
> ```spwn
> $._less_or_equal_(100, 100)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._less_pattern_
> ## Description:
> Default implementation of the `<a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._less_pattern_(11))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._less_than_
> ## Description:
> Default implementation of the `<` operator<div>
> ## Example:
> ```spwn
> $._less_than_(50, 100)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._minus_
> ## Description:
> Default implementation of the `-` operator<div>
> ## Example:
> ```spwn
> $._minus_(128, 64)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._mod_
> ## Description:
> Default implementation of the `%` operator<div>
> ## Example:
> ```spwn
> $._mod_(70, 8)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._modulate_
> ## Description:
> Default implementation of the `%=` operator<div>
> ## Example:
> ```spwn
> let val = 10
> $._modulate_(val, 3)
> $.assert(val == 1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
> | b | _Number_ |
## $._more_or_eq_pattern_
> ## Description:
> Default implementation of the `>=a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._more_or_eq_pattern_(10))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._more_or_equal_
> ## Description:
> Default implementation of the `>=` operator<div>
> ## Example:
> ```spwn
> $._more_or_equal_(100, 100)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._more_pattern_
> ## Description:
> Default implementation of the `>a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._more_pattern_(9))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._more_than_
> ## Description:
> Default implementation of the `>` operator<div>
> ## Example:
> ```spwn
> $._more_than_(100, 50)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._multiply_
> ## Description:
> Default implementation of the `*=` operator<div>
> ## Example:
> ```spwn
> let val = 5
> $._multiply_(val, 10)
> $.assert(val == 50)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_  |
> | b | _Number_ |
## $._negate_
> ## Description:
> Default implementation of the `-n` operator<div>
> ## Example:
> ```spwn
> $.assert($._negate_(1) == -1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
## $._not_
> ## Description:
> Default implementation of the `!b` operator<div>
> ## Example:
> ```spwn
> $.assert($._not_(false))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._not_eq_pattern_
> ## Description:
> Default implementation of the `!=a` operator<div>
> ## Example:
> ```spwn
> $.assert(10 is $._not_eq_pattern_(5))
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
## $._not_equal_
> ## Description:
> Default implementation of the `!=` operator<div>
> ## Example:
> ```spwn
> $._not_equal_("hello", "bye")
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b |  |
## $._or_
> ## Description:
> Default implementation of the `||` operator<div>
> ## Example:
> ```spwn
> $._or_(true, false)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Bool_ |
> | b | _Bool_ |
## $._plus_
> ## Description:
> Default implementation of the `+` operator<div>
> ## Example:
> ```spwn
> $._plus_(32, 32)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b |  |
## $._pow_
> ## Description:
> Default implementation of the `^` operator<div>
> ## Example:
> ```spwn
> $._pow_(8, 2)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _Number_ |
> | b | _Number_ |
## $._pre_decrement_
> ## Description:
> Default implementation of the `--n` operator<div>
> ## Example:
> ```spwn
> let n = 1
> $.assert($._pre_decrement_(n) == 0)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
## $._pre_increment_
> ## Description:
> Default implementation of the `++n` operator<div>
> ## Example:
> ```spwn
> let n = 0
> $.assert($._pre_increment_(n) == 1)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
## $._range_
> ## Description:
> Default implementation of the `..` operator<div>
> ## Example:
> ```spwn
> $._range_(0, 10)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | val_a |  |
> | b | _Number_ |
## $._subtract_
> ## Description:
> Default implementation of the `-=` operator<div>
> ## Example:
> ```spwn
> let val = 25
> $._subtract_(val, 10)
> $.assert(val == 15)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_ _Number_ |
> | b | _Number_ |
## $._swap_
> ## Description:
> Default implementation of the `<=>` operator<div>
> ## Example:
> ```spwn
> let a = 10
> let b = 5
> $._swap_(a, b)
> $.assert(a == 5)
> $.assert(b == 10)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a | _mutable_  |
> | b | _mutable_  |
## $._times_
> ## Description:
> Default implementation of the `*` operator<div>
> ## Example:
> ```spwn
> $._times_(8, 8)
> ```
> **Allowed by default:** true
> ## Arguments: 
> | **Name** | **Type** |
> |-|-|
> | a |  |
> | b | _Number_ |
