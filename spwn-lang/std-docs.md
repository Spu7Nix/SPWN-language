# Documentation for `std` 
_This file was generated using `spwn doc [file name]`_
## Info:

- Uses 0 groups
- Uses 0 colors
- Uses 0 block IDs
- Uses 1 item IDs

- Adds 0 objects
## Exports:
 **Type:** `@dictionary` 

<details>
<summary> View members </summary>


## Macros:


## **call_with_delay**:

> **Value:** `(time: @number | @epsilon = @epsilon::{
>}, function: @function) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Call a function after a delay_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `time` | @number or @epsilon | `@epsilon::{}` |Delay time in seconds (leave empty for minimum delay) |
>| 2 | **`function`** | @function | |Function to call after the delay |
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
>| 2 | **`b`** | @block | |Block B ID |
>
>

## **collision_exit**:

> **Value:** `(a: @block, b: @block) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an event for when a collision exits_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`a`** | @block | |Block A ID |
>| 2 | **`b`** | @block | |Block B ID |
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
>| 2 | `delay` | @bool | `true` |Adds a delay if a value gets added to the new item (to avoid confusing behavior) |
>
>

## **death**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an event for when the player dies_
>
>

## **disable_trail**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Disables the player's trail_
>
>

## **enable_trail**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Enables the player's trail_
>
>

## **for_loop**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of a spawn loop with a counter_
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
>

## **hide_player**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Hides the player_
>
>

## **on**:

> **Value:** `(event: @event, function: @function) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Triggers a function every time an event fires_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`event`** | @event | |Event to trigger on |
>| 2 | **`function`** | @function | |Function to trigger |
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
>| 2 | `interval` | @number | `0` |Interval value |
>| 3 | `duration` | @number | `0.5` |Duration of shake |
>
>

## **show_player**:

> **Value:** `() { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Shows the player_
>
>

## **supress_signal**:

> **Value:** `(delay: @number) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Stops signal from coming past for some time_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`delay`** | @number | |Time to supress signal |
>
>

## **toggle_bg_effect**:

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

## **touch_end**:

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

## **wait**:

> **Value:** `(time: @number | @epsilon = @epsilon::{
>}) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Adds a delay before the next triggers_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `time` | @number or @epsilon | `@epsilon::{}` |Delay time in seconds (leave empty for minimum delay) |
>
>

## **while_loop**:

> **Value:** `(expr: @macro, code: @macro, delay: @number | @epsilon = @epsilon::{
>}) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of a conditional spawn loop_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`expr`** | @macro | |While loop condition, should return a boolean |
>| 2 | **`code`** | @macro | |Macro of the code that gets looped |
>| 3 | `delay` | @number or @epsilon | `@epsilon::{}` |Delay between loops (less than 0.05 may be unstable) |
>
>
## Other values:

<details>
<summary> View </summary>

## **BACK_IN**:

> **Value:** `@easing_type::{
>id: 17
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `17` (`@number`) 
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
>
>

## **BACK_IN_OUT**:

> **Value:** `@easing_type::{
>id: 16
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `16` (`@number`) 
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
>
>

## **BACK_OUT**:

> **Value:** `@easing_type::{
>id: 18
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `18` (`@number`) 
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
>
>

## **BG**:

> **Value:** `1000c` (`@color`) 
>
>
>

## **BOUNCE_IN**:

> **Value:** `@easing_type::{
>id: 8
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `8` (`@number`) 
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
>
>

## **BOUNCE_IN_OUT**:

> **Value:** `@easing_type::{
>id: 7
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `7` (`@number`) 
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
>
>

## **BOUNCE_OUT**:

> **Value:** `@easing_type::{
>id: 9
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `9` (`@number`) 
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
>
>

## **EASE_IN**:

> **Value:** `@easing_type::{
>id: 2
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `2` (`@number`) 
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
>
>

## **EASE_IN_OUT**:

> **Value:** `@easing_type::{
>id: 1
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `1` (`@number`) 
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
>
>

## **EASE_OUT**:

> **Value:** `@easing_type::{
>id: 3
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `3` (`@number`) 
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
>
>

## **ELASTIC_IN**:

> **Value:** `@easing_type::{
>id: 5
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `5` (`@number`) 
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
>
>

## **ELASTIC_IN_OUT**:

> **Value:** `@easing_type::{
>id: 4
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `4` (`@number`) 
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
>
>

## **ELASTIC_OUT**:

> **Value:** `@easing_type::{
>id: 6
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `6` (`@number`) 
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
>
>

## **EQUAL_TO**:

> **Value:** `@comparison::{
>id: 0
>}` (`@comparison`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `0` (`@number`) 
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
>
>

## **EXPONENTIAL_IN**:

> **Value:** `@easing_type::{
>id: 11
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `11` (`@number`) 
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
>
>

## **EXPONENTIAL_IN_OUT**:

> **Value:** `@easing_type::{
>id: 10
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `10` (`@number`) 
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
>
>

## **EXPONENTIAL_OUT**:

> **Value:** `@easing_type::{
>id: 12
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `12` (`@number`) 
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
>
>

## **LARGER_THAN**:

> **Value:** `@comparison::{
>id: 1
>}` (`@comparison`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `1` (`@number`) 
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
>
>

## **NONE**:

> **Value:** `@easing_type::{
>id: 0
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `0` (`@number`) 
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
>
>

## **SINE_IN**:

> **Value:** `@easing_type::{
>id: 14
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `14` (`@number`) 
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
>
>

## **SINE_IN_OUT**:

> **Value:** `@easing_type::{
>id: 13
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `13` (`@number`) 
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
>
>

## **SINE_OUT**:

> **Value:** `@easing_type::{
>id: 15
>}` (`@easing_type`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `15` (`@number`) 
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
>
>

## **SMALLER_THAN**:

> **Value:** `@comparison::{
>id: 2
>}` (`@comparison`) 
>
><details>
><summary> View members </summary>
>
>## **id**:
>
>> **Value:** `2` (`@number`) 
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
>
>

## **obj_props**:

> **Type:** `@dictionary` 
>
><details>
><summary> View members </summary>
>
>## **ACTIVATE_GROUP**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 56
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `56` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ACTIVATE_ON_EXIT**:
>
>> **Value:** `@object_key::{
>>id: 93,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `93` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ANIMATION_ID**:
>
>> **Value:** `@object_key::{
>>id: 76,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `76` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ANIMATION_SPEED**:
>
>> **Value:** `@object_key::{
>>id: 107,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `107` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **BLENDING**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 17
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `17` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **BLOCK_A**:
>
>> **Value:** `@object_key::{
>>id: 80,
>>pattern: @block
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `80` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **BLOCK_B**:
>
>> **Value:** `@object_key::{
>>id: 95,
>>pattern: @block
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `95` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **CENTER**:
>
>> **Value:** `@object_key::{
>>id: 71,
>>pattern: @group
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `71` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COLOR**:
>
>> **Value:** `@object_key::{
>>id: 21,
>>pattern: @color
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `21` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COLOR_2**:
>
>> **Value:** `@object_key::{
>>id: 22,
>>pattern: @color
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `22` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COLOR_2_HVS**:
>
>> **Value:** `@object_key::{
>>pattern: @string,
>>id: 44
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `44` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COLOR_2_HVS_ENABLED**:
>
>> **Value:** `@object_key::{
>>id: 42,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `42` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COMPARISON**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 88
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `88` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COPIED_COLOR_HVS**:
>
>> **Value:** `@object_key::{
>>pattern: @string,
>>id: 49
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `49` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COPIED_COLOR_ID**:
>
>> **Value:** `@object_key::{
>>pattern: @color,
>>id: 50
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `50` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COPY_OPACTITY**:
>
>> **Value:** `@object_key::{
>>id: 60,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `60` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COUNT**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 77
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `77` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **COUNT_MULTI_ACTIVATE**:
>
>> **Value:** `@object_key::{
>>id: 104,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `104` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DELAY**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 91
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `91` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DETAIL_ONLY**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 66
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `66` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DISABLE_ROTATION**:
>
>> **Value:** `@object_key::{
>>id: 98,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `98` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DONT_ENTER**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 67
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `67` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DONT_FADE**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 64
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `64` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DUAL_MODE**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 89
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `89` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DURATION**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 10
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `10` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **DYNAMIC_BLOCK**:
>
>> **Value:** `@object_key::{
>>id: 94,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `94` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **EASING**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 30
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `30` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **EASING_RATE**:
>
>> **Value:** `@object_key::{
>>id: 85,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `85` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **EDITOR_DISABLE**:
>
>> **Value:** `@object_key::{
>>id: 102,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `102` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **EDITOR_LAYER_1**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 20
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `20` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **EDITOR_LAYER_2**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 61
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `61` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **EXCLUSIVE**:
>
>> **Value:** `@object_key::{
>>id: 86,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `86` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **FADE_IN**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 45
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `45` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **FADE_OUT**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 47
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `47` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **FOLLOW**:
>
>> **Value:** `@object_key::{
>>id: 71,
>>pattern: @group
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `71` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **GLOW_DISABLED**:
>
>> **Value:** `@object_key::{
>>id: 96,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `96` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **GROUPS**:
>
>> **Value:** `@object_key::{
>>id: 57,
>>pattern: [@group] | @group
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `57` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **GROUP_PARENT**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 34
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `34` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **HIGH_DETAIL**:
>
>> **Value:** `@object_key::{
>>id: 103,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `103` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **HOLD**:
>
>> **Value:** `@object_key::{
>>id: 46,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `46` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **HOLD_MODE**:
>
>> **Value:** `@object_key::{
>>id: 81,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `81` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **HORIZONTAL_FLIP**:
>
>> **Value:** `@object_key::{
>>id: 4,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `4` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **HVS**:
>
>> **Value:** `@object_key::{
>>id: 43,
>>pattern: @string
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `43` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **HVS_ENABLED**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 41
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `41` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **INTERVAL**:
>
>> **Value:** `@object_key::{
>>id: 84,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `84` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ITEM**:
>
>> **Value:** `@object_key::{
>>pattern: @item,
>>id: 80
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `80` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **LINKED_GROUP**:
>
>> **Value:** `@object_key::{
>>id: 108,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `108` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **LOCK_OBJECT_ROTATION**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 70
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `70` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **LOCK_TO_PLAYER_X**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 58
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `58` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **LOCK_TO_PLAYER_Y**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 59
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `59` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **MAIN_ONLY**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 65
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `65` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **MAX_SPEED**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 105
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `105` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **MOVE_X**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 28
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `28` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **MOVE_Y**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 29
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `29` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **MULTI_TRIGGER**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 87
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `87` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **OBJ_ID**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 1
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `1` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **OPACITY**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 35
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `35` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **PICKUP_MODE**:
>
>> **Value:** `@object_key::{
>>id: 79,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `79` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **PLAYER_COLOR_1**:
>
>> **Value:** `@object_key::{
>>id: 15,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `15` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **PLAYER_COLOR_2**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 16
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `16` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **PORTAL_CHECKED**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 13
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `13` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **PULSE_HSV**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 48
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `48` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **RANDOMIZE_START**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 106
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `106` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ROTATE_DEGREES**:
>
>> **Value:** `@object_key::{
>>id: 68,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `68` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ROTATION**:
>
>> **Value:** `@object_key::{
>>id: 6,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `6` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **ROTATION_SPEED**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 97
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `97` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **SCALING**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 32
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `32` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **SPAWN_DURATION**:
>
>> **Value:** `@object_key::{
>>id: 63,
>>pattern: @number | @epsilon
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `63` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **SPAWN_TRIGGERED**:
>
>> **Value:** `@object_key::{
>>id: 62,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `62` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **SPEED**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 90
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `90` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **STRENGTH**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 75
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `75` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **SUBTRACT_COUNT**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 78
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `78` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TARGET**:
>
>> **Value:** `@object_key::{
>>pattern: @color | @group | @function,
>>id: 51
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `51` (`@number`) 
>>>
>>>
>>>
>>
>>## **pattern**:
>>
>>> **Value:** `@color | @group | @function` (`@pattern`) 
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
>>
>>
>
>## **TARGET_COLOR**:
>
>> **Value:** `@object_key::{
>>pattern: @color,
>>id: 23
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `23` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TARGET_POS**:
>
>> **Value:** `@object_key::{
>>pattern: @group,
>>id: 71
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `71` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TARGET_POS_AXES**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 101
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `101` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TARGET_TYPE**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 52
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `52` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TEXT**:
>
>> **Value:** `@object_key::{
>>pattern: @string,
>>id: 31
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `31` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TIMES_360**:
>
>> **Value:** `@object_key::{
>>id: 69,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `69` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TOGGLE_MODE**:
>
>> **Value:** `@object_key::{
>>id: 82,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `82` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TOUCH_TRIGGERED**:
>
>> **Value:** `@object_key::{
>>id: 11,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `11` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TRIGGER_BLUE**:
>
>> **Value:** `@object_key::{
>>id: 9,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `9` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TRIGGER_GREEN**:
>
>> **Value:** `@object_key::{
>>id: 8,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `8` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **TRIGGER_RED**:
>
>> **Value:** `@object_key::{
>>id: 7,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `7` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **USE_TARGET**:
>
>> **Value:** `@object_key::{
>>pattern: @bool,
>>id: 100
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `100` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **VERTICAL_FLIP**:
>
>> **Value:** `@object_key::{
>>id: 5,
>>pattern: @bool
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `5` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **X**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 2
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `2` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **X_MOD**:
>
>> **Value:** `@object_key::{
>>id: 72,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `72` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **Y**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 3
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `3` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **YELLOW_TELEPORTATION_PORTAL_DISTANCE**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 54
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `54` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **Y_MOD**:
>
>> **Value:** `@object_key::{
>>id: 73,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `73` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **Y_OFFSET**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 92
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `92` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **Z_LAYER**:
>
>> **Value:** `@object_key::{
>>pattern: @number,
>>id: 24
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `24` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>## **Z_ORDER**:
>
>> **Value:** `@object_key::{
>>id: 25,
>>pattern: @number
>>}` (`@object_key`) 
>>
>><details>
>><summary> View members </summary>
>>
>>## **id**:
>>
>>> **Value:** `25` (`@number`) 
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
>>
>>## **type**:
>>
>>> **Value:** `@object_key` (`@type_indicator`) 
>>>
>>>
>>>
>>
>>
>
>


</details>

</details>


## Type Implementations:
### **@group**: 
 <details>
<summary> View members </summary>

## **alpha**:

> **Value:** `(self, opacity: @number = 1, duration: @number = 0) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the alpha trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `opacity` | @number | `1` | |
>| 3 | `duration` | @number | `0` | |
>
>

## **follow**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the follow trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @group | |Group of object to follow |
>| 3 | `x_mod` | @number | `1` |Multiplier for the movement on the X-axis |
>| 4 | `y_mod` | @number | `1` |Multiplier for the movement on the Y-axis |
>| 5 | `duration` | @number | `999` |Duration of following |
>
>

## **follow_player_y**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the follow player Y trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `speed` | @number | `1` |Interpolation factor (?) |
>| 3 | `delay` | @number | `0` |Delay of movement |
>| 4 | `offset` | @number | `0` |Offset on the Y-axis |
>| 5 | `max_speed` | @number | `0` |Maximum speed |
>| 6 | `duration` | @number | `999` |Duration of following |
>
>

## **lock_to_player**:

> **Value:** `(self, lock_x: @bool = true, lock_y: @bool = true, duration: @number = 999) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Lock group to player position_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `lock_x` | @bool | `true` |Lock to player X |
>| 3 | `lock_y` | @bool | `true` |Lock to player Y |
>| 4 | `duration` | @number | `999` |Duration of lock |
>
>

## **move**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the move trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`x`** | @number | |Units to move on the X axis |
>| 3 | **`y`** | @number | |Units to move on the Y axis |
>| 4 | `duration` | @number | `0` |Duration of movement |
>| 5 | `easing` | @easing_type | `@easing_type::{id: 0}` | |
>| 6 | `easing_rate` | @number | `2` | |
>
>

## **move_to**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the 'Move target' feature of the move trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`target`** | @group | |Group of the object to move to |
>| 3 | `duration` | @number | `0` |Duration of movement |
>| 4 | `x_only` | @bool | `false` |Will move to the object only on the X-axis |
>| 5 | `y_only` | @bool | `false` |Will move to the object only on the y-axis |
>| 6 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>| 7 | `easing_rate` | @number | `2` |Easing rate |
>
>

## **pulse**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the pulse trigger for groups_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`r`** | @number | |Red value of pulse color (or hue if HSV is enabled) |
>| 3 | **`g`** | @number | |Green value of pulse color (or saturation if HSV is enabled) |
>| 4 | **`b`** | @number | |Blue value of pulse color (or brightness/value if HSV is enabled) |
>| 5 | `fade_in` | @number | `0` |Fade-in duration |
>| 6 | `hold` | @number | `0` |Duration to hold the color |
>| 7 | `fade_out` | @number | `0` |Fade-out duration |
>| 8 | `exclusive` | @bool | `false` |Weather to prioritize this pulse over simultaneous pulses |
>| 9 | `hsv` | @bool | `false` |Toggle HSV mode |
>
>

## **rotate**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the rotate trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`center`** | @group | |Group of object to rotate around |
>| 3 | **`degrees`** | @number | |Rotation in degrees |
>| 4 | `duration` | @number | `0` |Duration of rotation |
>| 5 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>| 6 | `easing_rate` | @number | `2` |Easing rate |
>| 7 | `lock_object_rotation` | @bool | `false` |Only rotate positions of the objects, not the textures |
>
>

## **stop**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the stop trigger_
>
>

## **toggle_off**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Toggles the group off_
>
>

## **toggle_on**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Toggles the group on_
>
>
</details>

### **@color**: 
 <details>
<summary> View members </summary>

## **pulse**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the pulse trigger for colors_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`r`** | @number | |Red value of pulse color (or hue if HSV is enabled) |
>| 3 | **`g`** | @number | |Green value of pulse color (or saturation if HSV is enabled) |
>| 4 | **`b`** | @number | |Blue value of pulse color (or brightness/value if HSV is enabled) |
>| 5 | `fade_in` | @number | `0` |Fade-in duration |
>| 6 | `hold` | @number | `0` |Duration to hold the color |
>| 7 | `fade_out` | @number | `0` |Fade-out duration |
>| 8 | `exclusive` | @bool | `false` |Weather to prioritize this pulse over simultaneous pulses |
>| 9 | `hsv` | @bool | `false` |Toggle HSV mode |
>
>

## **set**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the color trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`r`** | @number | |Red value of the target color |
>| 3 | **`g`** | @number | |Green value of the target color |
>| 4 | **`b`** | @number | |Blue value of the target color |
>| 5 | `duration` | @number | `0` |Duration of color change |
>| 6 | `opacity` | @number | `1` |Opacity of target color |
>| 7 | `blending` | @bool | `false` |Toggle blending on target color |
>
>
</details>

### **@block**: 
 <details>
<summary> View members </summary>

## **create_tracker_item**:

> **Value:** `(self, other: @block) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an item ID that is 1 when the blocks are colliding and 0 when they are not_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @block | |Block ID to check against |
>
>
</details>

### **@item**: 
 <details>
<summary> View members </summary>

## **add**:

> **Value:** `(self, amount: @number) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the pickup trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`amount`** | @number | |Amount to add |
>
>

## **count**:

> **Value:** `(self, number: @number = 0) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the count trigger (returns an event)_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `number` | @number | `0` |Number to check against |
>
>

## **if_is**:

> **Value:** `(self, comparison: @comparison, other: @number, function: @function) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the instant count trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`comparison`** | @comparison | |Comparison mode |
>| 3 | **`other`** | @number | |Number to compare with |
>| 4 | **`function`** | @function | |Target function if comparison is 'true' |
>
>
</details>

### **@array**: 
 <details>
<summary> View members </summary>

## **contains**:

> **Value:** `(self, el) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`el`** |any | | |
>
>

## **max**:

> **Value:** `(self, minval = 0) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `minval` |any | `0` | |
>
>

## **min**:

> **Value:** `(self, max_val = 999999999999) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `max_val` |any | `999999999999` | |
>
>

## **push**:

> **Value:** `(self, value) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`value`** |any | | |
>
>
</details>

### **@event**: 
 <details>
<summary> View members </summary>

## **on**:

> **Value:** `(event: @event, function: @function) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Triggers a function every time an event fires_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`event`** | @event | |Event to trigger on |
>| 2 | **`function`** | @function | |Function to trigger |
>
>
</details>

### **@counter**: 
 <details>
<summary> View members </summary>

## **_add_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_as_**:

> **Value:** `(self, _type: @type_indicator) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`_type`** | @type_indicator | | |
>
>

## **_assign_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_divide_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_divided_by_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_equal_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_less_or_equal_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_less_than_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_minus_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_mod_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_more_or_equal_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_more_than_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_multiply_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_not_equal_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_plus_**:

> **Value:** `(self, other: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @number or @counter | | |
>
>

## **_subtract_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **_times_**:

> **Value:** `(self, num: @number | @counter) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number or @counter | | |
>
>

## **add**:

> **Value:** `(self, num: @number) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the pickup trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`num`** | @number | |Amount to add |
>
>

## **add_to**:

> **Value:** `(self, items: @array, speed: @number = 3, factor: @number = 1) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Adds the counter's value to all item IDs in a list, and resets the counter to 0 in the process_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`items`** | @array | |Item IDs to add to |
>| 3 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>| 4 | `factor` | @number | `1` |Multiplyer for the value added |
>
>

## **clone**:

> **Value:** `(self, speed: @number = 3) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Copies the counter and returns the copy_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>
>

## **compare**:

> **Value:** `(self, other: @counter, speed: @number = 3) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @counter | | |
>| 3 | `speed` | @number | `3` | |
>
>

## **copy_to**:

> **Value:** `(self, items: [@item | @counter], speed: @number = 3, factor: @number = 1) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Copies the value of the counter to another item ID, without consuming the original_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`items`** | [@item or @counter] | |Items to copy to |
>| 3 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>| 4 | `factor` | @number | `1` |Factor of to multiply the copy by |
>
>

## **divide**:

> **Type:** `@macro` 
>
>## Description: 
> _Devides the value of the counter by some divisor_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`divisor`** | @counter or @number | |Divisor to divide by, either another counter (very expensive) or a normal number |
>| 3 | `remainder` | @counter or @item | `@counter::{item: ?i}` |Counter or item to set to the remainder value |
>| 4 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>
>

## **multiply**:

> **Value:** `(self, factor: @counter | @number, speed: @number = 3) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Multiplies the value of the counter by some factor (does not consume the factor)_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`factor`** | @counter or @number | |Factor to multiply by, either another counter (very expensive) or a normal number |
>| 3 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>
>

## **new**:

> **Value:** `(source: @number | @item | @bool = 0, delay: @bool = true) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Creates a new counter_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `source` | @number or @item or @bool | `0` |Source (can be a number, item ID or boolean) |
>| 2 | `delay` | @bool | `true` |Adds a delay if a value gets added to the new item (to avoid confusing behavior) |
>
>

## **reset**:

> **Value:** `(self, speed: @number = 3) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Resets counter to 0_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>
>

## **subtract_from**:

> **Value:** `(self, items: @array, speed: @number = 3, factor: @number = 1) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Subtracts the counter's value from all item IDs in a list, and resets the counter to 0 in the process_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`items`** | @array | |Item IDs to add to |
>| 3 | `speed` | @number | `3` |Speed of operation (higher number increases group usage) |
>| 4 | `factor` | @number | `1` |Multiplyer for the value subtracted |
>
>

## **to_const**:

> **Value:** `(self, range: [@number] | @range) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Converts the counter into a normal number (very context-splitting, be careful)_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`range`** | [@number] or @range | |Array or range of possible output values |
>
>
</details>

