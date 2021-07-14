# Documentation for `std` 
_Generated using `spwn doc [file name]`_
## Info:

- Uses 1 groups
- Uses 0 colors
- Uses 0 block IDs
- Uses 2 item IDs

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

> **Value:** `(time: @number | @epsilon = @epsilon::{}, function: @trigger_function) { /* code omitted */ }` (`@macro`) 
>
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
>  | 2 | **`function`** | @trigger_function | |Function to call after the delay |
>  
>  
>

## **collision**:

> **Value:** `(a: @block, b: @block) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the collision trigger (returns an event)_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`a`** | @block | |Block A ID |
>  | 2 | **`b`** | @block | |Block B ID |
>  
>  
>

## **collision\_exit**:

> **Value:** `(a: @block, b: @block) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an event for when a collision exits_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`a`** | @block | |Block A ID |
>  | 2 | **`b`** | @block | |Block B ID |
>  
>  
>

## **counter**:

> **Value:** `(source: @number | @item | @bool = 0, delay: @bool = true) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Creates a new counter_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `source` | @number or @item or @bool | `0` |Source (can be a number, item ID or boolean) |
>  | 2 | `delay` | @bool | `true` |Adds a delay if a value gets added to the new item (to avoid confusing behavior) |
>  
>  
>

## **death**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an event for when the player dies_
>
>  
>

## **disable\_trail**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Disables the player's trail_
>
>  
>

## **do\_while\_loop**:

> **Value:** `(expr: @macro, code: @macro, delay: @number | @epsilon = @epsilon::{}) { /* code omitted */ }` (`@macro`) 
>
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
>  | 2 | **`code`** | @macro | |Macro of the code that gets looped |
>  | 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>  
>  
>

## **enable\_trail**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Enables the player's trail_
>
>  
>

## **for\_loop**:

> **Type:** `@macro` 
>
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
>  | 2 | **`code`** | @macro | |Macro of the code that gets looped, should take the iterator (a counter) as the first argument. |
>  | 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>  | 4 | `reset` | @bool | `true` |Weather to reset the iterator after looping (only disable if the loop is only triggered once) |
>  | 5 | `reset_speed` | @number | `1` |Operation speed of the reset of the iterator, if enabled |
>  
>  
>

## **hide\_player**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Hides the player_
>
>  
>

## **obj\_set**:

> **Value:** `(objects: @array, group: @group = ?g) { /* code omitted */ }` (`@macro`) 
>
>### Example: 
>```spwn
> @obj_set::new([]);
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`objects`** | @array | | |
>  | 2 | `group` | @group | `?g` |The group to use for rotation (?) |
>  
>  
>

## **on**:

> **Value:** `(event: @event, function: @trigger_function) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Triggers a function every time an event fires_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`event`** | @event | |Event to trigger on |
>  | 2 | **`function`** | @trigger_function | |Function to trigger |
>  
>  
>

## **open**:

> **Value:** `(path: @string) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`path`** | @string | | |
>  
>  
>

## **regex**:

> **Value:** `(re: @string) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Create a new instance of regex_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`re`** | @string | |A regex string. Make sure to use two backslashes to escape selectors instead of one or it will error |
>  
>  
>

## **shake**:

> **Value:** `(strength: @number = 1, interval: @number = 0, duration: @number = 0.5) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the shake trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `strength` | @number | `1` |Strength value |
>  | 2 | `interval` | @number | `0` |Interval value |
>  | 3 | `duration` | @number | `0.5` |Duration of shake |
>  
>  
>

## **show\_player**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Shows the player_
>
>  
>

## **supress\_signal**:

> **Value:** `(delay: @number) { /* code omitted */ }` (`@macro`) 
>
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
>  
>

## **supress\_signal\_forever**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
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
>  
>

## **toggle\_bg\_effect**:

> **Value:** `(on: @bool = false) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the bg effect on/off triggers_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `on` | @bool | `false` |Weather to toggle bg effect on or off |
>  
>  
>

## **touch**:

> **Value:** `(dual_side: @bool = false) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the touch trigger (returns an event)_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `dual_side` | @bool | `false` |Dual mode (only check for touch on the dual side) |
>  
>  
>

## **touch\_end**:

> **Value:** `(dual_side: @bool = false) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an event for when a touch ends_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `dual_side` | @bool | `false` |Dual mode (only check for touch on the dual side) |
>  
>  
>

## **wait**:

> **Value:** `(time: @number | @epsilon = @epsilon::{}) { /* code omitted */ }` (`@macro`) 
>
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
>  
>

## **while\_loop**:

> **Value:** `(expr: @macro, code: @macro, delay: @number | @epsilon = @epsilon::{}) { /* code omitted */ }` (`@macro`) 
>
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
>  | 2 | **`code`** | @macro | |Macro of the code that gets looped |
>  | 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>  
>  
>
## Other values:


## **BACK\_IN**:

> **Value:** `@easing_type::{id: 17}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `17` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **BACK\_IN\_OUT**:

> **Value:** `@easing_type::{id: 16}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `16` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **BACK\_OUT**:

> **Value:** `@easing_type::{id: 18}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `18` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **BG**:

> **Value:** `1000c` (`@color`) 
>
>
>  
>

## **BOUNCE\_IN**:

> **Value:** `@easing_type::{id: 8}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `8` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **BOUNCE\_IN\_OUT**:

> **Value:** `@easing_type::{id: 7}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `7` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **BOUNCE\_OUT**:

> **Value:** `@easing_type::{id: 9}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `9` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EASE\_IN**:

> **Value:** `@easing_type::{id: 2}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `2` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EASE\_IN\_OUT**:

> **Value:** `@easing_type::{id: 1}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `1` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EASE\_OUT**:

> **Value:** `@easing_type::{id: 3}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `3` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **ELASTIC\_IN**:

> **Value:** `@easing_type::{id: 5}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `5` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **ELASTIC\_IN\_OUT**:

> **Value:** `@easing_type::{id: 4}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `4` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **ELASTIC\_OUT**:

> **Value:** `@easing_type::{id: 6}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `6` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EQUAL\_TO**:

> **Value:** `@comparison::{id: 0}` (`@comparison`) 
>
>
>## **id**:
>
>> **Value:** `0` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@comparison` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EXPONENTIAL\_IN**:

> **Value:** `@easing_type::{id: 11}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `11` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EXPONENTIAL\_IN\_OUT**:

> **Value:** `@easing_type::{id: 10}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `10` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **EXPONENTIAL\_OUT**:

> **Value:** `@easing_type::{id: 12}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `12` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **GROUND**:

> **Value:** `1001c` (`@color`) 
>
>
>  
>

## **GROUND2**:

> **Value:** `1009c` (`@color`) 
>
>
>  
>

## **LARGER\_THAN**:

> **Value:** `@comparison::{id: 1}` (`@comparison`) 
>
>
>## **id**:
>
>> **Value:** `1` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@comparison` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **LINE**:

> **Value:** `1002c` (`@color`) 
>
>
>  
>

## **NONE**:

> **Value:** `@easing_type::{id: 0}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `0` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **OBJECT**:

> **Value:** `1004c` (`@color`) 
>
>
>  
>

## **SINE\_IN**:

> **Value:** `@easing_type::{id: 14}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `14` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **SINE\_IN\_OUT**:

> **Value:** `@easing_type::{id: 13}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `13` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **SINE\_OUT**:

> **Value:** `@easing_type::{id: 15}` (`@easing_type`) 
>
>
>## **id**:
>
>> **Value:** `15` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@easing_type` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **SMALLER\_THAN**:

> **Value:** `@comparison::{id: 2}` (`@comparison`) 
>
>
>## **id**:
>
>> **Value:** `2` (`@number`) 
>>
>>
>>  
>>
>
>## **type**:
>
>> **Value:** `@comparison` (`@type_indicator`) 
>>
>>
>>  
>>
>
>  
>

## **\_3DLINE**:

> **Value:** `1003c` (`@color`) 
>
>
>  
>

## **obj\_props**:

> **Type:** `@dictionary` 
>
>
>## **ACTIVATE\_GROUP**:
>
>> **Value:** `@object_key::{id: 56,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `56` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ACTIVATE\_ON\_EXIT**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 93}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `93` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ACTIVE\_TRIGGER**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 36}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `36` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ANIMATION\_ID**:
>
>> **Value:** `@object_key::{id: 76,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `76` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ANIMATION\_SPEED**:
>
>> **Value:** `@object_key::{id: 107,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `107` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **BLENDING**:
>
>> **Value:** `@object_key::{id: 17,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `17` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **BLOCK\_A**:
>
>> **Value:** `@object_key::{id: 80,pattern: @block}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `80` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@block` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **BLOCK\_B**:
>
>> **Value:** `@object_key::{pattern: @block,id: 95}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `95` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@block` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **CENTER**:
>
>> **Value:** `@object_key::{id: 71,pattern: @group}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `71` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@group` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COLOR**:
>
>> **Value:** `@object_key::{id: 21,pattern: @color}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `21` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@color` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COLOR\_2**:
>
>> **Value:** `@object_key::{id: 22,pattern: @color}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `22` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@color` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COLOR\_2\_HVS**:
>
>> **Value:** `@object_key::{id: 44,pattern: @string}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `44` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@string` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COLOR\_2\_HVS\_ENABLED**:
>
>> **Value:** `@object_key::{id: 42,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `42` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COMPARISON**:
>
>> **Value:** `@object_key::{id: 88,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `88` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COPIED\_COLOR\_HVS**:
>
>> **Value:** `@object_key::{id: 49,pattern: @string}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `49` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@string` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COPIED\_COLOR\_ID**:
>
>> **Value:** `@object_key::{pattern: @color,id: 50}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `50` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@color` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COPY\_OPACITY**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 60}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `60` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COUNT**:
>
>> **Value:** `@object_key::{id: 77,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `77` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **COUNT\_MULTI\_ACTIVATE**:
>
>> **Value:** `@object_key::{id: 104,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `104` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DELAY**:
>
>> **Value:** `@object_key::{id: 91,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `91` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DETAIL\_ONLY**:
>
>> **Value:** `@object_key::{id: 66,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `66` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DISABLE\_ROTATION**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 98}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `98` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DONT\_ENTER**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 67}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `67` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DONT\_FADE**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 64}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `64` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DUAL\_MODE**:
>
>> **Value:** `@object_key::{id: 89,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `89` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DURATION**:
>
>> **Value:** `@object_key::{id: 10,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `10` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **DYNAMIC\_BLOCK**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 94}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `94` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **EASING**:
>
>> **Value:** `@object_key::{pattern: @number,id: 30}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `30` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **EASING\_RATE**:
>
>> **Value:** `@object_key::{pattern: @number,id: 85}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `85` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **EDITOR\_DISABLE**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 102}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `102` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **EDITOR\_LAYER\_1**:
>
>> **Value:** `@object_key::{pattern: @number,id: 20}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `20` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **EDITOR\_LAYER\_2**:
>
>> **Value:** `@object_key::{pattern: @number,id: 61}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `61` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **EXCLUSIVE**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 86}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `86` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **FADE\_IN**:
>
>> **Value:** `@object_key::{id: 45,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `45` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **FADE\_OUT**:
>
>> **Value:** `@object_key::{pattern: @number,id: 47}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `47` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **FOLLOW**:
>
>> **Value:** `@object_key::{id: 71,pattern: @group}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `71` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@group` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **GLOW\_DISABLED**:
>
>> **Value:** `@object_key::{id: 96,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `96` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **GROUPS**:
>
>> **Value:** `@object_key::{pattern: [@group] | @group,id: 57}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `57` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `[@group] | @group` (`@pattern`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **GROUP\_PARENT**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 34}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `34` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **HIGH\_DETAIL**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 103}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `103` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **HOLD**:
>
>> **Value:** `@object_key::{id: 46,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `46` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **HOLD\_MODE**:
>
>> **Value:** `@object_key::{id: 81,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `81` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **HORIZONTAL\_FLIP**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 4}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `4` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **HVS**:
>
>> **Value:** `@object_key::{id: 43,pattern: @string}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `43` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@string` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **HVS\_ENABLED**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 41}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `41` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **INTERVAL**:
>
>> **Value:** `@object_key::{pattern: @number,id: 84}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `84` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ITEM**:
>
>> **Value:** `@object_key::{pattern: @item,id: 80}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `80` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@item` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **LINKED\_GROUP**:
>
>> **Value:** `@object_key::{id: 108,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `108` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **LOCK\_OBJECT\_ROTATION**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 70}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `70` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **LOCK\_TO\_PLAYER\_X**:
>
>> **Value:** `@object_key::{id: 58,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `58` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **LOCK\_TO\_PLAYER\_Y**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 59}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `59` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **MAIN\_ONLY**:
>
>> **Value:** `@object_key::{id: 65,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `65` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **MAX\_SPEED**:
>
>> **Value:** `@object_key::{pattern: @number,id: 105}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `105` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **MOVE\_X**:
>
>> **Value:** `@object_key::{pattern: @number,id: 28}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `28` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **MOVE\_Y**:
>
>> **Value:** `@object_key::{pattern: @number,id: 29}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `29` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **MULTI\_TRIGGER**:
>
>> **Value:** `@object_key::{id: 87,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `87` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **OBJ\_ID**:
>
>> **Value:** `@object_key::{pattern: @number,id: 1}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `1` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **OPACITY**:
>
>> **Value:** `@object_key::{id: 35,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `35` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **PICKUP\_MODE**:
>
>> **Value:** `@object_key::{id: 79,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `79` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **PLAYER\_COLOR\_1**:
>
>> **Value:** `@object_key::{id: 15,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `15` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **PLAYER\_COLOR\_2**:
>
>> **Value:** `@object_key::{id: 16,pattern: @bool}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `16` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **PORTAL\_CHECKED**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 13}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `13` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **PULSE\_HSV**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 48}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `48` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **RANDOMIZE\_START**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 106}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `106` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ROTATE\_DEGREES**:
>
>> **Value:** `@object_key::{pattern: @number,id: 68}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `68` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ROTATION**:
>
>> **Value:** `@object_key::{id: 6,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `6` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **ROTATION\_SPEED**:
>
>> **Value:** `@object_key::{pattern: @number,id: 97}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `97` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **SCALING**:
>
>> **Value:** `@object_key::{id: 32,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `32` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **SPAWN\_DURATION**:
>
>> **Value:** `@object_key::{pattern: @number | @epsilon,id: 63}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `63` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number | @epsilon` (`@pattern`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **SPAWN\_TRIGGERED**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 62}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `62` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **SPEED**:
>
>> **Value:** `@object_key::{id: 90,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `90` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **STRENGTH**:
>
>> **Value:** `@object_key::{id: 75,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `75` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **SUBTRACT\_COUNT**:
>
>> **Value:** `@object_key::{id: 78,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `78` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TARGET**:
>
>> **Value:** `@object_key::{pattern: @color | @group | @trigger_function,id: 51}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `51` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@color | @group | @trigger_function` (`@pattern`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TARGET\_COLOR**:
>
>> **Value:** `@object_key::{pattern: @color,id: 23}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `23` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@color` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TARGET\_POS**:
>
>> **Value:** `@object_key::{pattern: @group,id: 71}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `71` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@group` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TARGET\_POS\_AXES**:
>
>> **Value:** `@object_key::{pattern: @number,id: 101}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `101` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TARGET\_TYPE**:
>
>> **Value:** `@object_key::{pattern: @number,id: 52}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `52` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TEXT**:
>
>> **Value:** `@object_key::{id: 31,pattern: @string}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `31` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@string` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TIMES\_360**:
>
>> **Value:** `@object_key::{id: 69,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `69` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TOGGLE\_MODE**:
>
>> **Value:** `@object_key::{id: 82,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `82` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TOUCH\_TRIGGERED**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 11}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `11` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TRIGGER\_BLUE**:
>
>> **Value:** `@object_key::{id: 9,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `9` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TRIGGER\_GREEN**:
>
>> **Value:** `@object_key::{pattern: @number,id: 8}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `8` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **TRIGGER\_RED**:
>
>> **Value:** `@object_key::{id: 7,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `7` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **USE\_TARGET**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 100}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `100` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **VERTICAL\_FLIP**:
>
>> **Value:** `@object_key::{pattern: @bool,id: 5}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `5` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@bool` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **X**:
>
>> **Value:** `@object_key::{pattern: @number,id: 2}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `2` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **X\_MOD**:
>
>> **Value:** `@object_key::{pattern: @number,id: 72}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `72` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **Y**:
>
>> **Value:** `@object_key::{id: 3,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `3` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **YELLOW\_TELEPORTATION\_PORTAL\_DISTANCE**:
>
>> **Value:** `@object_key::{pattern: @number,id: 54}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `54` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **Y\_MOD**:
>
>> **Value:** `@object_key::{pattern: @number,id: 73}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `73` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **Y\_OFFSET**:
>
>> **Value:** `@object_key::{id: 92,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `92` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **Z\_LAYER**:
>
>> **Value:** `@object_key::{id: 24,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `24` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>## **Z\_ORDER**:
>
>> **Value:** `@object_key::{id: 25,pattern: @number}` (`@object_key`) 
>>
>>
>>## **id**:
>>
>>> **Value:** `25` (`@number`) 
>>>
>>>
>>>  
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@number` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>  
>>>
>>
>>  
>>
>
>  
>

  