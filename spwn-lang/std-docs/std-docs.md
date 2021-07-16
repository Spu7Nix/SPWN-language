# Documentation for `std` 
_Generated using `spwn doc [file name]`_
## Info:

- Uses 1 groups
- Uses 0 colors
- Uses 0 block IDs
- Uses 1 item IDs

- Adds 0 objects
# Type Implementations:
- [**@group**](std-docs/group.md)
- [**@color**](std-docs/color.md)
- [**@block**](std-docs/block.md)
- [**@item**](std-docs/item.md)
- [**@dictionary**](std-docs/dictionary.md)
- [**@string**](std-docs/string.md)
- [**@array**](std-docs/array.md)
- [**@object**](std-docs/object.md)
- [**@event**](std-docs/event.md)
- [**@obj_set**](std-docs/obj_set.md)
- [**@counter**](std-docs/counter.md)
- [**@file**](std-docs/file.md)
- [**@regex**](std-docs/regex.md)
# Exports:
 **Type:** `@dictionary` 

## Macros:

## **call\_with\_delay**:

> **Value:** 
>```spwn
>(time: @number | @epsilon = @epsilon::{}, function: @trigger_function) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Call a function after a delay_
>### Example: 
>```spwn
> BG.set(255, 0, 0) // turn background red
>call_with_delay(2, !{
>	BG.set(0, 255, 0) // turn background green 2 seconds later
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `time` | @number or @epsilon | `@epsilon::{}` |Delay time in seconds (leave empty for minimum delay) |
>| 2 | **`function`** | @trigger_function | |Function to call after the delay |
>

## **collision**:

> **Value:** 
>```spwn
>(a: @block, b: @block) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the collision trigger (returns an event)_
>### Example: 
>```spwn
> on(collision(1b, 2b), !{
>    BG.set(255, 0, 0)
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`a`** | @block | |Block A ID |
>| 2 | **`b`** | @block | |Block B ID |
>

## **collision\_exit**:

> **Value:** 
>```spwn
>(a: @block, b: @block) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns an event for when a collision exits_
>### Example: 
>```spwn
> on(collision_exit(1b, 2b), !{
>    BG.set(0, 0, 0)
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`a`** | @block | |Block A ID |
>| 2 | **`b`** | @block | |Block B ID |
>

## **counter**:

> **Value:** 
>```spwn
>(source: @number | @item | @bool = 0, delay: @bool = true) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Creates a new counter_
>### Example: 
>```spwn
> @counter::new()     // creates a new counter with a starting value of 0
>@counter::new(10)   // creates a new counter with a starting value of 10
>@counter::new(5i)   // creates a new counter thaat uses item ID 5
>@counter::new(true)   // creates a new counter with a starting value of true (1)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `source` | @number or @item or @bool | `0` |Source (can be a number, item ID or boolean) |
>| 2 | `delay` | @bool | `true` |Adds a delay if a value gets added to the new item (to avoid confusing behavior) |
>

## **death**:

> **Value:** 
>```spwn
>() { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns an event for when the player dies_
>### Example: 
>```spwn
> on(death(), !{
>    BG.set(0, 0, 0)
>})
>```
>

## **disable\_trail**:

> **Value:** 
>```spwn
>() { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Disables the player's trail_
>### Example: 
>```spwn
> disable_trail()
>```
>

## **do\_while\_loop**:

> **Value:** 
>```spwn
>(expr: @macro, code: @macro, delay: @number | @epsilon = @epsilon::{}) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of a conditional spawn loop_
>### Example: 
>```spwn
> c = counter(4)
>
>do_while_loop(() => c > 10, () {
>	c -= 2
>})
>
>// c is now 2
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`expr`** | @macro | |While loop condition, should -> return a boolean |
>| 2 | **`code`** | @macro | |Macro of the code that gets looped |
>| 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>

## **enable\_trail**:

> **Value:** 
>```spwn
>() { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Enables the player's trail_
>### Example: 
>```spwn
> enable_trail()
>```
>

## **for\_loop**:

> **Value:** 
>```spwn
>(range: @range, code: @macro, delay: @number | @epsilon = @epsilon::{}, reset: @bool = true, reset_speed: @number = 1) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of a spawn loop with a counter_
>### Example: 
>```spwn
> for_loop(0..10, (i) {
>	if i < 5 {
>		10g.move(-10, 0)
>	} else {
>		10g.move(10, 0)
>	}
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`range`** | @range | |Range of values (for example 0..10) |
>| 2 | **`code`** | @macro | |Macro of the code that gets looped, should take the iterator (a counter) as the first argument. |
>| 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>| 4 | `reset` | @bool | `true` |Weather to reset the iterator after looping (only disable if the loop is only triggered once) |
>| 5 | `reset_speed` | @number | `1` |Operation speed of the reset of the iterator, if enabled |
>

## **hide\_player**:

> **Value:** 
>```spwn
>() { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Hides the player_
>### Example: 
>```spwn
> hide_player()
>```
>

## **obj\_set**:

> **Value:** 
>```spwn
>(objects: @array = [], group: @group = ?g) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Creates a new object set_
>### Example: 
>```spwn
> my_objects = @obj_set::new()
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `objects` | @array | `[]` | |
>| 2 | `group` | @group | `?g` |The center group to use for rotation |
>

## **on**:

> **Value:** 
>```spwn
>(event: @event, function: @trigger_function) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Triggers a function every time an event fires_
>### Example: 
>```spwn
> on(touch(), !{
>    10g.move(10, 0)
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`event`** | @event | |Event to trigger on |
>| 2 | **`function`** | @trigger_function | |Function to trigger |
>

## **open**:

> **Value:** 
>```spwn
>(path: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Creates a new file IO object_
>### Example: 
>```spwn
> @file::new('C:/path/to/file.txt')
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`path`** | @string | | |
>

## **regex**:

> **Value:** 
>```spwn
>(re: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Create a new instance of regex_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`re`** | @string | |A regex string. Make sure to use two backslashes to escape selectors instead of one or it will error |
>

## **shake**:

> **Value:** 
>```spwn
>(strength: @number = 1, interval: @number = 0, duration: @number = 0.5) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the shake trigger_
>### Example: 
>```spwn
> shake()
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `strength` | @number | `1` |Strength value |
>| 2 | `interval` | @number | `0` |Interval value |
>| 3 | `duration` | @number | `0.5` |Duration of shake |
>

## **show\_player**:

> **Value:** 
>```spwn
>() { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Shows the player_
>### Example: 
>```spwn
> show_player()
>```
>

## **supress\_signal**:

> **Value:** 
>```spwn
>(delay: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Stops signal from coming past for some time_
>### Example: 
>```spwn
> f = !{
>	supress_signal(1)
>	10g.move(10, 0)
>}
>
>f! // moves
>wait(0.4)
>f! // does nothing
>wait(0.4)
>f! // does nothing
>wait(0.4)
>f! // moves
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`delay`** | @number | |Time to supress signal |
>

## **supress\_signal\_forever**:

> **Value:** 
>```spwn
>() { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Stops signal from coming past after call_
>### Example: 
>```spwn
> f = !{
>	supress_signal_forever()
>	10g.move(10, 0)
>}
>f! // moves
>wait(0.4)
>f! // does nothing
>wait(1000)
>f! // does nothing
>```
>

## **toggle\_bg\_effect**:

> **Value:** 
>```spwn
>(on: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the bg effect on/off triggers_
>### Example: 
>```spwn
> toggle_bg_effect(false)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `on` | @bool | `false` |Weather to toggle bg effect on or off |
>

## **touch**:

> **Value:** 
>```spwn
>(dual_side: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the touch trigger (returns an event)_
>### Example: 
>```spwn
> on(touch(), !{
>    10g.move(10, 0)
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `dual_side` | @bool | `false` |Dual mode (only check for touch on the dual side) |
>

## **touch\_end**:

> **Value:** 
>```spwn
>(dual_side: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns an event for when a touch ends_
>### Example: 
>```spwn
> on(touch_end(), !{
>    10g.move(-10, 0)
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `dual_side` | @bool | `false` |Dual mode (only check for touch on the dual side) |
>

## **wait**:

> **Value:** 
>```spwn
>(time: @number | @epsilon = @epsilon::{}) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Adds a delay before the next triggers_
>### Example: 
>```spwn
> BG.set(255, 0, 0) // turn background red
>wait(2) // wait 2 seconds
>BG.set(0, 255, 0) // turn background green
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `time` | @number or @epsilon | `@epsilon::{}` |Delay time in seconds (leave empty for minimum delay) |
>

## **while\_loop**:

> **Value:** 
>```spwn
>(expr: @macro, code: @macro, delay: @number | @epsilon = @epsilon::{}) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of a conditional spawn loop_
>### Example: 
>```spwn
> c = counter(11)
>
>while_loop(() => c > 4, () {
>	c -= 2
>})
>
>// c is now 3
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`expr`** | @macro | |While loop condition, should -> return a boolean |
>| 2 | **`code`** | @macro | |Macro of the code that gets looped |
>| 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>
## Other values:

## **BACK\_IN**:

> **Value:** 
>```spwn
>@easing_type::{id: 17}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>17
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **BACK\_IN\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 16}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>16
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **BACK\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 18}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>18
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **BG**:

> **Value:** 
>```spwn
>1000c
>``` 
>**Type:** `@color` 
>

## **BOUNCE\_IN**:

> **Value:** 
>```spwn
>@easing_type::{id: 8}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>8
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **BOUNCE\_IN\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 7}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>7
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **BOUNCE\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 9}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>9
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EASE\_IN**:

> **Value:** 
>```spwn
>@easing_type::{id: 2}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>2
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EASE\_IN\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 1}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>1
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EASE\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 3}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>3
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **ELASTIC\_IN**:

> **Value:** 
>```spwn
>@easing_type::{id: 5}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>5
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **ELASTIC\_IN\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 4}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>4
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **ELASTIC\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 6}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>6
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EQUAL\_TO**:

> **Value:** 
>```spwn
>@comparison::{id: 0}
>``` 
>**Type:** `@comparison` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>0
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@comparison
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EXPONENTIAL\_IN**:

> **Value:** 
>```spwn
>@easing_type::{id: 11}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>11
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EXPONENTIAL\_IN\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 10}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>10
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **EXPONENTIAL\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 12}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>12
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **GROUND**:

> **Value:** 
>```spwn
>1001c
>``` 
>**Type:** `@color` 
>

## **GROUND2**:

> **Value:** 
>```spwn
>1009c
>``` 
>**Type:** `@color` 
>

## **LARGER\_THAN**:

> **Value:** 
>```spwn
>@comparison::{id: 1}
>``` 
>**Type:** `@comparison` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>1
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@comparison
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **LINE**:

> **Value:** 
>```spwn
>1002c
>``` 
>**Type:** `@color` 
>

## **NONE**:

> **Value:** 
>```spwn
>@easing_type::{id: 0}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>0
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **OBJECT**:

> **Value:** 
>```spwn
>1004c
>``` 
>**Type:** `@color` 
>

## **SINE\_IN**:

> **Value:** 
>```spwn
>@easing_type::{id: 14}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>14
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **SINE\_IN\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 13}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>13
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **SINE\_OUT**:

> **Value:** 
>```spwn
>@easing_type::{id: 15}
>``` 
>**Type:** `@easing_type` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>15
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@easing_type
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **SMALLER\_THAN**:

> **Value:** 
>```spwn
>@comparison::{id: 2}
>``` 
>**Type:** `@comparison` 
>
>## **id**:
>
>> **Value:** 
>>```spwn
>>2
>>``` 
>>**Type:** `@number` 
>>
>
>## **type**:
>
>> **Value:** 
>>```spwn
>>@comparison
>>``` 
>>**Type:** `@type_indicator` 
>>
>

## **\_3DLINE**:

> **Value:** 
>```spwn
>1003c
>``` 
>**Type:** `@color` 
>

## **obj\_props**:

> **Type:** `@dictionary` 
>
>## **ACTIVATE\_GROUP**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 56}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>56
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ACTIVATE\_ON\_EXIT**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 93,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>93
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ACTIVE\_TRIGGER**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 36}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>36
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ANIMATION\_ID**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 76,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>76
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ANIMATION\_SPEED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 107}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>107
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **BLENDING**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 17}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>17
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **BLOCK\_A**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @block,id: 80}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>80
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@block
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **BLOCK\_B**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 95,pattern: @block}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>95
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@block
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **CENTER**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @group,id: 71}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>71
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@group
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COLOR**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 21,pattern: @color}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>21
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@color
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COLOR\_2**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @color,id: 22}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>22
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@color
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COLOR\_2\_HVS**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @string,id: 44}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>44
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@string
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COLOR\_2\_HVS\_ENABLED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 42}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>42
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COMPARISON**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 88,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>88
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COPIED\_COLOR\_HVS**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 49,pattern: @string}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>49
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@string
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COPIED\_COLOR\_ID**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @color,id: 50}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>50
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@color
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COPY\_OPACITY**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 60,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>60
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COUNT**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 77,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>77
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **COUNT\_MULTI\_ACTIVATE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 104}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>104
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DELAY**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 91,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>91
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DETAIL\_ONLY**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 66,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>66
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DISABLE\_ROTATION**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 98}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>98
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DONT\_ENTER**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 67,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>67
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DONT\_FADE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 64}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>64
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DUAL\_MODE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 89}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>89
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DURATION**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 10}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>10
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **DYNAMIC\_BLOCK**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 94}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>94
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **EASING**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 30,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>30
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **EASING\_RATE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 85,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>85
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **EDITOR\_DISABLE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 102,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>102
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **EDITOR\_LAYER\_1**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 20,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>20
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **EDITOR\_LAYER\_2**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 61}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>61
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **EXCLUSIVE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 86,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>86
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **FADE\_IN**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 45}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>45
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **FADE\_OUT**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 47}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>47
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **FOLLOW**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 71,pattern: @group}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>71
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@group
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **GLOW\_DISABLED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 96}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>96
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **GROUPS**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 57,pattern: [@group] | @group}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>57
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>[@group] | @group
>>>``` 
>>>**Type:** `@pattern` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **GROUP\_PARENT**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 34,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>34
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **HIGH\_DETAIL**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 103}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>103
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **HOLD**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 46,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>46
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **HOLD\_MODE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 81,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>81
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **HORIZONTAL\_FLIP**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 4}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>4
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **HVS**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @string,id: 43}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>43
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@string
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **HVS\_ENABLED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 41,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>41
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **INTERVAL**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 84}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>84
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ITEM**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @item,id: 80}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>80
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@item
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **LINKED\_GROUP**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 108,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>108
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **LOCK\_OBJECT\_ROTATION**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 70,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>70
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **LOCK\_TO\_PLAYER\_X**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 58}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>58
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **LOCK\_TO\_PLAYER\_Y**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 59}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>59
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **MAIN\_ONLY**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 65}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>65
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **MAX\_SPEED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 105,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>105
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **MOVE\_X**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 28,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>28
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **MOVE\_Y**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 29}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>29
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **MULTI\_TRIGGER**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 87,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>87
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **OBJ\_ID**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 1}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>1
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **OPACITY**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 35}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>35
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **PICKUP\_MODE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 79}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>79
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **PLAYER\_COLOR\_1**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 15}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>15
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **PLAYER\_COLOR\_2**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 16,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>16
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **PORTAL\_CHECKED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 13,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>13
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **PULSE\_HSV**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 48,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>48
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **RANDOMIZE\_START**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @bool,id: 106}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>106
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ROTATE\_DEGREES**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 68}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>68
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ROTATION**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 6}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>6
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **ROTATION\_SPEED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 97}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>97
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **SCALING**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 32,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>32
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **SPAWN\_DURATION**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 63,pattern: @number | @epsilon}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>63
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number | @epsilon
>>>``` 
>>>**Type:** `@pattern` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **SPAWN\_TRIGGERED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 62,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>62
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **SPEED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 90,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>90
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **STRENGTH**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 75}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>75
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **SUBTRACT\_COUNT**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 78}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>78
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TARGET**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @color | @group | @trigger_function,id: 51}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>51
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@color | @group | @trigger_function
>>>``` 
>>>**Type:** `@pattern` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TARGET\_COLOR**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 23,pattern: @color}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>23
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@color
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TARGET\_POS**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @group,id: 71}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>71
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@group
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TARGET\_POS\_AXES**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 101}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>101
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TARGET\_TYPE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 52,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>52
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TEXT**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 31,pattern: @string}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>31
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@string
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TIMES\_360**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 69,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>69
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TOGGLE\_MODE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 82,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>82
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TOUCH\_TRIGGERED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 11,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>11
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TRIGGER\_BLUE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 9,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>9
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TRIGGER\_GREEN**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 8,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>8
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **TRIGGER\_RED**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 7,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>7
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **USE\_TARGET**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 100,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>100
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **VERTICAL\_FLIP**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 5,pattern: @bool}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>5
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@bool
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **X**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 2,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>2
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **X\_MOD**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 72,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>72
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **Y**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 3}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>3
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **YELLOW\_TELEPORTATION\_PORTAL\_DISTANCE**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 54,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>54
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **Y\_MOD**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 73,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>73
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **Y\_OFFSET**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 92}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>92
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **Z\_LAYER**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 24,pattern: @number}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>24
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
>## **Z\_ORDER**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @number,id: 25}
>>``` 
>>**Type:** `@object_key` 
>>
>>## **id**:
>>
>>> **Value:** 
>>>```spwn
>>>25
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **pattern**:
>>
>>> **Value:** 
>>>```spwn
>>>@number
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>>## **type**:
>>
>>> **Value:** 
>>>```spwn
>>>@object_key
>>>``` 
>>>**Type:** `@type_indicator` 
>>>
>>
>
