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
## $.print
> ## Description:
> Prints value(s) to the console <div> 
> ## Example:
> ```spwn
> $.print("Hello world!")
> ```
> **Allowed by default:** yes
## $.time
> ## Description:
> Gets the current system time in seconds <div> 
> ## Example:
> ```spwn
> now = $.time()
> ```
> **Allowed by default:** yes
## $.spwn\_version
> ## Description:
> Gets the current version of spwn <div> 
> ## Example:
> ```spwn
> $.spwn_version()
> ```
> **Allowed by default:** yes
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
## $.http\_request
> ## Description:
> Sends an HTTP request <div> 
> **Allowed by default:** no
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `method` | _Str_ |
> | 2 | `url` | _Str_ |
> | 3 | `headers` | _Dict_ |
> | 4 | `body` | _Str_ |
## $.sin
> ## Description:
> Calculates the sin of an angle in radians <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.cos
> ## Description:
> Calculates the cos of an angle in radians <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.tan
> ## Description:
> 	Calculates the tan of an angle in radians <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.asin
> ## Description:
> Calculates the arcsin of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.acos
> ## Description:
> Calculates the arccos of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.atan
> ## Description:
> Calculates the arctan of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.floor
> ## Description:
> Calculates the floor of a number, AKA the number rounded down to the nearest integer <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.ceil
> ## Description:
> Calculates the ceil of a number, AKA the number rounded up to the nearest integer <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.abs
> ## Description:
> Calculates the absolute value of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.acosh
> ## Description:
> Calculates the arccosh of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.asinh
> ## Description:
> Calculates the arcsinh of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.atan2
> ## Description:
> Calculates the arctan^2 of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `x` | _Number_ |
> | 2 | `y` | _Number_ |
## $.atanh
> ## Description:
> Calculates the arctanh of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.cbrt
> ## Description:
> Calculates the cube root of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.cosh
> ## Description:
> Calculates the cosh of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.exp
> ## Description:
> Calculates the e^x of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.exp2
> ## Description:
> Calculates the 2^x of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.exp\_m1
> ## Description:
> Calculates e^x - 1 in a way that is accurate even if the number is close to zero <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.fract
> ## Description:
> Gets the fractional part of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.sqrt
> ## Description:
> Calculates the square root of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.sinh
> ## Description:
> Calculates the hyperbolic sin of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.tanh
> ## Description:
> Calculates the hyperbolic tan of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.ln
> ## Description:
> Calculates the ln (natural log) of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.log
> ## Description:
> Calculates the log base x of a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
> | 2 | `base` | _Number_ |
## $.min
> ## Description:
> Calculates the min of two numbers <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.max
> ## Description:
> Calculates the max of two numbers <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.round
> ## Description:
> Rounds a number <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `n` | _Number_ |
## $.hypot
> ## Description:
> Calculates the hypothenuse in a right triangle with sides a and b <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.add
> ## Description:
> Adds a Geometry Dash object or trigger to the target level <div> 
> **Allowed by default:** yes
## $.append
> ## Description:
> Appends a value to the end of an array. You can also use `array.push(value)` <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `arr` | _mutable_ _Array_ |
> | 2 | `val` |  |
## $.split\_str
> ## Description:
> Returns an array from the split string. You can also use `string.split(delimiter)` <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `s` | _Str_ |
> | 2 | `substr` | _Str_ |
## $.edit\_obj
> ## Description:
> Changes the value of an object key. You can also use `object.set(key, value)` <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `o, m` | _mutable_ _Obj_ |
> | 2 | `key` |  |
> | 3 | `value` |  |
## $.mutability
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `var` |  |
## $.extend\_trigger\_func
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `group` |  |
> | 2 | `mac` | _Macro_ |
## $.trigger\_fn\_context
> **Allowed by default:** yes
## $.random
> **Allowed by default:** yes
## $.readfile
> **Allowed by default:** no
## $.writefile
> **Allowed by default:** no
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `path` | _Str_ |
> | 2 | `data` | _Str_ |
## $.pop
> ## Description:
> Removes a value from the end of an array. You can also use `array.pop()` <div> 
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `arr` | _mutable_  |
## $.substr
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `val` | _Str_ |
> | 2 | `start_index` | _Number_ |
> | 3 | `end_index` | _Number_ |
## $.remove\_index
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `arr` | _mutable_  |
> | 2 | `index` | _Number_ |
## $.regex
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `regex` | _Str_ |
> | 2 | `s` | _Str_ |
> | 3 | `mode` | _Str_ |
> | 4 | `replace` |  |
## $.\_range\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `val_a` |  |
> | 2 | `b` | _Number_ |
## $.\_increment\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_decrement\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_pre\_increment\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_pre\_decrement\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
## $.\_negate\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
## $.\_not\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Bool_ |
## $.\_unary\_range\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
## $.\_or\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Bool_ |
> | 2 | `b` | _Bool_ |
## $.\_and\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Bool_ |
> | 2 | `b` | _Bool_ |
## $.\_more\_than\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_less\_than\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_more\_or\_equal\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_less\_or\_equal\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_equal\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_not\_equal\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_divided\_by\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_intdivided\_by\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_times\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` | _Number_ |
## $.\_mod\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_pow\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_plus\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_minus\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _Number_ |
> | 2 | `b` | _Number_ |
## $.\_assign\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` |  |
## $.\_swap\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` | _mutable_  |
## $.\_has\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
## $.\_as\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `t` | _TypeIndicator_ |
## $.\_subtract\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_add\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` |  |
## $.\_multiply\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_  |
> | 2 | `b` | _Number_ |
## $.\_divide\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_intdivide\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_exponate\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_modulate\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` | _mutable_ _Number_ |
> | 2 | `b` | _Number_ |
## $.\_either\_
> **Allowed by default:** yes
> ## Arguments: 
> | # | **Name** | **Type** |
> |-|-|-|
> | 1 | `a` |  |
> | 2 | `b` |  |
