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
- [**@number**](std-docs/number.md)
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

## **alpha\_trigger**:

> **Value:** 
>```spwn
>(group: @group, opacity: @number = 1, duration: @number = 0) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns an alpha trigger as an object_
>### Example: 
>```spwn
> $.add( alpha_trigger(1g,0.5,duration = 2).with(X,600) ) // Creates an alpha trigger at X 600 that fades group 1 to half opacity over 2 seconds
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | | |
>| 2 | `opacity` | @number | `1` | |
>| 3 | `duration` | @number | `0` | |
>

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

## **color\_trigger**:

> **Value:** 
>```spwn
>(channel: @color, r: @number, g: @number, b: @number, duration: @number = 0, opacity: @number = 1, blending: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a color trigger as an object_
>### Example: 
>```spwn
> $.add( color_trigger(BG,0,0,0,0.5).with(X,600) )
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`channel`** | @color | |Color channel to change |
>| 2 | **`r`** | @number | |Red value of the target color |
>| 3 | **`g`** | @number | |Green value of the target color |
>| 4 | **`b`** | @number | |Blue value of the target color |
>| 5 | `duration` | @number | `0` |Duration of color change |
>| 6 | `opacity` | @number | `1` |Opacity of target color |
>| 7 | `blending` | @bool | `false` |Toggle blending on target color |
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

## **follow\_player\_y\_trigger**:

> **Value:** 
>```spwn
>(group: @group, speed: @number = 1, delay: @number = 0, offset: @number = 0, max_speed: @number = 0, duration: @number = 999) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a follow player Y trigger as an object_
>### Example: 
>```spwn
> $.add( follow_player_y_trigger(10g,delay = 0.5).with(X,600) )
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group that will follow |
>| 2 | `speed` | @number | `1` |Interpolation factor (?) |
>| 3 | `delay` | @number | `0` |Delay of movement |
>| 4 | `offset` | @number | `0` |Offset on the Y-axis |
>| 5 | `max_speed` | @number | `0` |Maximum speed |
>| 6 | `duration` | @number | `999` |Duration of following |
>

## **follow\_trigger**:

> **Value:** 
>```spwn
>(group: @group, other: @group, x_mod: @number = 1, y_mod: @number = 1, duration: @number = 999) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a follow trigger as an object_
>### Example: 
>```spwn
> $.add( follow_trigger(10g,3g).with(X,600) ) // Creates a follow trigger at X 600 that makes group 10 follow group 3
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group that will follow |
>| 2 | **`other`** | @group | |Group of object to follow |
>| 3 | `x_mod` | @number | `1` |Multiplier for the movement on the X-axis |
>| 4 | `y_mod` | @number | `1` |Multiplier for the movement on the Y-axis |
>| 5 | `duration` | @number | `999` |Duration of following |
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

## **lock\_to\_player\_trigger**:

> **Value:** 
>```spwn
>(group: @group, lock_x: @bool = true, lock_y: @bool = true, duration: @number = 999) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a move trigger that locks the group's position as an object_
>### Example: 
>```spwn
> $.add( lock_to_player_trigger(1g,lock_x = true,lock_y = false).with(X,600) ) // Creates a move trigger at X 600 that locks group 1 to the player's X
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to lock |
>| 2 | `lock_x` | @bool | `true` |Lock to player X |
>| 3 | `lock_y` | @bool | `true` |Lock to player Y |
>| 4 | `duration` | @number | `999` |Duration of lock |
>

## **move\_to\_trigger**:

> **Value:** 
>```spwn
>(group: @group, target: @group, duration: @number = 0, x_only: @bool = false, y_only: @bool = false, easing: @easing_type = @easing_type::{id: 0}, easing_rate: @number = 2) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a move trigger that uses "move to" as an object_
>### Example: 
>```spwn
> $.add( move_to_trigger(10g,3g).with(X,600) ) // Creates a move trigger at X 600 that moves group 10 to group 3
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to move |
>| 2 | **`target`** | @group | |Group of the object to move to |
>| 3 | `duration` | @number | `0` |Duration of movement |
>| 4 | `x_only` | @bool | `false` |Will move to the object only on the X-axis |
>| 5 | `y_only` | @bool | `false` |Will move to the object only on the y-axis |
>| 6 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>| 7 | `easing_rate` | @number | `2` |Easing rate |
>

## **move\_trigger**:

> **Value:** 
>```spwn
>(group: @group, x: @number, y: @number, duration: @number = 0, easing: @easing_type = @easing_type::{id: 0}, easing_rate: @number = 2) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a move trigger as an object_
>### Example: 
>```spwn
> $.add( move_trigger(1g,10,0).with(X,600) ) // Creates a move trigger at X 600 that moves group 1 a block to the right
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to move |
>| 2 | **`x`** | @number | |Units to move on the X axis |
>| 3 | **`y`** | @number | |Units to move on the Y axis |
>| 4 | `duration` | @number | `0` |Duration of movement |
>| 5 | `easing` | @easing_type | `@easing_type::{id: 0}` | |
>| 6 | `easing_rate` | @number | `2` | |
>

## **obj\_set**:

> **Value:** 
>```spwn
>(objects: [@object] = [], group: @group = ?g) { /* code omitted */ }
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
>| 1 | `objects` | [@object] | `[]` | |
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

## **pickup\_trigger**:

> **Value:** 
>```spwn
>(item_id: @item, amount: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a pickup trigger as an object_
>### Example: 
>```spwn
> $.add( pickup_trigger(1i,3).with(X,600) )
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`item_id`** | @item | |Item ID to modify |
>| 2 | **`amount`** | @number | |Amount to add |
>

## **pulse\_trigger**:

> **Value:** 
>```spwn
>(target: @group | @color, r: @number, g: @number, b: @number, fade_in: @number = 0, hold: @number = 0, fade_out: @number = 0, exclusive: @bool = false, hsv: @bool = false, s_checked: @bool = false, b_checked: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a pulse trigger as an object_
>### Example: 
>```spwn
> $.add( pulse_trigger(10g,255,0,0,fade_out = 0.5).with(X,600) )
>    $.add( pulse_trigger(10c,255,0,0,fade_out = 0.5).with(X,600) )
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`target`** | @group or @color | |Target to pulse (group or color) |
>| 2 | **`r`** | @number | |Red value of pulse color (or hue if HSV is enabled) |
>| 3 | **`g`** | @number | |Green value of pulse color (or saturation if HSV is enabled) |
>| 4 | **`b`** | @number | |Blue value of pulse color (or brightness/value if HSV is enabled) |
>| 5 | `fade_in` | @number | `0` |Fade-in duration |
>| 6 | `hold` | @number | `0` |Duration to hold the color |
>| 7 | `fade_out` | @number | `0` |Fade-out duration |
>| 8 | `exclusive` | @bool | `false` |Weather to prioritize this pulse over simultaneous pulses |
>| 9 | `hsv` | @bool | `false` |Toggle HSV mode |
>| 10 | `s_checked` | @bool | `false` |HSV specific: saturation checked |
>| 11 | `b_checked` | @bool | `false` |HSV specific: brightness checked |
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

## **rotate\_trigger**:

> **Value:** 
>```spwn
>(group: @group, center: @group, degrees: @number, duration: @number = 0, easing: @easing_type = @easing_type::{id: 0}, easing_rate: @number = 2, lock_object_rotation: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a rotate trigger as an object_
>### Example: 
>```spwn
> $.add( rotate_trigger(10g,3g,90,duration = 5).with(X,600) ) // Creates a rotate trigger at X 600 that rotates group 10 90 degrees around group 3 over 5 seconds
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to rotate |
>| 2 | **`center`** | @group | |Group of object to rotate around |
>| 3 | **`degrees`** | @number | |Rotation in degrees |
>| 4 | `duration` | @number | `0` |Duration of rotation |
>| 5 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>| 6 | `easing_rate` | @number | `2` |Easing rate |
>| 7 | `lock_object_rotation` | @bool | `false` |Only rotate positions of the objects, not the textures |
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

## **spawn\_trigger**:

> **Value:** 
>```spwn
>(group: @group, delay: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a spawn trigger as an object_
>### Example: 
>```spwn
> $.add( spawn_trigger(5g,0.5).with(X,600) )
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to spawn |
>| 2 | **`delay`** | @number | |Delay |
>

## **stop\_trigger**:

> **Value:** 
>```spwn
>(group: @group) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a stop trigger as an object_
>### Example: 
>```spwn
> $.add( stop_trigger(10g).with(X,600) ) // Creates a stop trigger at X 600 that stops group 10
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to stop |
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

## **toggle\_off\_trigger**:

> **Value:** 
>```spwn
>(group: @group) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a toggle off trigger as an object_
>### Example: 
>```spwn
> $.add( toggle_off_trigger(5g).with(X,600) ) // Creates a toggle trigger at X 600 that turns off group 5
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to toggle |
>

## **toggle\_on\_trigger**:

> **Value:** 
>```spwn
>(group: @group) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns a toggle on trigger as an object_
>### Example: 
>```spwn
> $.add( toggle_on_trigger(5g).with(X,600) ) // Creates a toggle trigger at X 600 that turns on group 5
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`group`** | @group | |Group to toggle |
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

## **obj\_ids**:

> **Type:** `@dictionary` 
>
>## **portals**:
>
>> **Value:** 
>>```spwn
>>{WAVE: 660,SIZE_NORMAL: 99,GRAVITY_UP: 11,SPEED_GREEN: 202,BALL: 47,SPIDER: 1331,DUAL_OFF: 287,GRAVITY_DOWN: 10,DUAL_ON: 286,CUBE: 12,SPEED_YELLOW: 200,... (9 more) }
>>``` 
>>**Type:** `@dictionary` 
>>
>>## **BALL**:
>>
>>> **Value:** 
>>>```spwn
>>>47
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **CUBE**:
>>
>>> **Value:** 
>>>```spwn
>>>12
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **DUAL\_OFF**:
>>
>>> **Value:** 
>>>```spwn
>>>287
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **DUAL\_ON**:
>>
>>> **Value:** 
>>>```spwn
>>>286
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **GRAVITY\_DOWN**:
>>
>>> **Value:** 
>>>```spwn
>>>10
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **GRAVITY\_UP**:
>>
>>> **Value:** 
>>>```spwn
>>>11
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **MIRROR\_OFF**:
>>
>>> **Value:** 
>>>```spwn
>>>46
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **MIRROR\_ON**:
>>
>>> **Value:** 
>>>```spwn
>>>45
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **ROBOT**:
>>
>>> **Value:** 
>>>```spwn
>>>745
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SHIP**:
>>
>>> **Value:** 
>>>```spwn
>>>13
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SIZE\_MINI**:
>>
>>> **Value:** 
>>>```spwn
>>>101
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SIZE\_NORMAL**:
>>
>>> **Value:** 
>>>```spwn
>>>99
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPEED\_BLUE**:
>>
>>> **Value:** 
>>>```spwn
>>>201
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPEED\_GREEN**:
>>
>>> **Value:** 
>>>```spwn
>>>202
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPEED\_PINK**:
>>
>>> **Value:** 
>>>```spwn
>>>203
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPEED\_RED**:
>>
>>> **Value:** 
>>>```spwn
>>>1334
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPEED\_YELLOW**:
>>
>>> **Value:** 
>>>```spwn
>>>200
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPIDER**:
>>
>>> **Value:** 
>>>```spwn
>>>1331
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **TELEPORT**:
>>
>>> **Value:** 
>>>```spwn
>>>747
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **UFO**:
>>
>>> **Value:** 
>>>```spwn
>>>111
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **WAVE**:
>>
>>> **Value:** 
>>>```spwn
>>>660
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>
>## **triggers**:
>
>> **Value:** 
>>```spwn
>>{DISABLE_TRAIL: 33,STOP: 1616,TOGGLE: 1049,COUNT: 1611,COLOR: 899,ROTATE: 1346,ON_DEATH: 1812,ALPHA: 1007,MOVE: 901,HIDE: 1612,BG_EFFECT_ON: 1818,... (12 more) }
>>``` 
>>**Type:** `@dictionary` 
>>
>>## **ALPHA**:
>>
>>> **Value:** 
>>>```spwn
>>>1007
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **ANIMATE**:
>>
>>> **Value:** 
>>>```spwn
>>>1585
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **BG\_EFFECT\_OFF**:
>>
>>> **Value:** 
>>>```spwn
>>>1819
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **BG\_EFFECT\_ON**:
>>
>>> **Value:** 
>>>```spwn
>>>1818
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **COLLISION**:
>>
>>> **Value:** 
>>>```spwn
>>>1815
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **COLOR**:
>>
>>> **Value:** 
>>>```spwn
>>>899
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **COUNT**:
>>
>>> **Value:** 
>>>```spwn
>>>1611
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **DISABLE\_TRAIL**:
>>
>>> **Value:** 
>>>```spwn
>>>33
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **ENABLE\_TRAIL**:
>>
>>> **Value:** 
>>>```spwn
>>>32
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **FOLLOW**:
>>
>>> **Value:** 
>>>```spwn
>>>1347
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **FOLLOW\_PLAYER\_Y**:
>>
>>> **Value:** 
>>>```spwn
>>>1814
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **HIDE**:
>>
>>> **Value:** 
>>>```spwn
>>>1612
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **INSTANT\_COUNT**:
>>
>>> **Value:** 
>>>```spwn
>>>1811
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **MOVE**:
>>
>>> **Value:** 
>>>```spwn
>>>901
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **ON\_DEATH**:
>>
>>> **Value:** 
>>>```spwn
>>>1812
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **PICKUP**:
>>
>>> **Value:** 
>>>```spwn
>>>1817
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **PULSE**:
>>
>>> **Value:** 
>>>```spwn
>>>1006
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **ROTATE**:
>>
>>> **Value:** 
>>>```spwn
>>>1346
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SHAKE**:
>>
>>> **Value:** 
>>>```spwn
>>>1520
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SHOW**:
>>
>>> **Value:** 
>>>```spwn
>>>1613
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **SPAWN**:
>>
>>> **Value:** 
>>>```spwn
>>>1268
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **STOP**:
>>
>>> **Value:** 
>>>```spwn
>>>1616
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **TOGGLE**:
>>
>>> **Value:** 
>>>```spwn
>>>1049
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>>## **TOUCH**:
>>
>>> **Value:** 
>>>```spwn
>>>1595
>>>``` 
>>>**Type:** `@number` 
>>>
>>
>

## **obj\_props**:

> **Type:** `@dictionary` 
>
>## **ACTIVATE\_GROUP**:
>
>> **Value:** 
>>```spwn
>>@object_key::{id: 56,pattern: @bool}
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
>>@object_key::{pattern: @number,id: 76}
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
>>@object_key::{pattern: @block,id: 95}
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
>## **COLOR**:
>
>> **Value:** 
>>```spwn
>>@object_key::{pattern: @color,id: 21}
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
>>@object_key::{id: 42,pattern: @bool}
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
>>@object_key::{pattern: @string,id: 49}
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
>>@object_key::{id: 104,pattern: @bool}
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
>>@object_key::{id: 98,pattern: @bool}
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
>>@object_key::{pattern: @bool,id: 67}
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
>>@object_key::{id: 64,pattern: @bool}
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
>>@object_key::{id: 94,pattern: @bool}
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
>>@object_key::{pattern: @number,id: 20}
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
>>@object_key::{pattern: @bool,id: 86}
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
>>@object_key::{id: 45,pattern: @number}
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
>>@object_key::{pattern: [@group] | @group,id: 57}
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
>>@object_key::{id: 103,pattern: @bool}
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
>>@object_key::{pattern: @bool,id: 81}
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
>>@object_key::{id: 43,pattern: @string}
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
>>@object_key::{pattern: @bool,id: 41}
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
>>@object_key::{id: 58,pattern: @bool}
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
>>@object_key::{id: 65,pattern: @bool}
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
>>@object_key::{pattern: @number,id: 28}
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
>>@object_key::{id: 1,pattern: @number}
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
>>@object_key::{id: 79,pattern: @number}
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
>>@object_key::{id: 15,pattern: @bool}
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
>>@object_key::{pattern: @bool,id: 16}
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
>>@object_key::{pattern: @bool,id: 48}
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
>>@object_key::{id: 106,pattern: @bool}
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
>>@object_key::{id: 68,pattern: @number}
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
>>@object_key::{pattern: @number,id: 32}
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
>>@object_key::{pattern: @number | @epsilon,id: 63}
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
>>@object_key::{pattern: @bool,id: 62}
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
>>@object_key::{id: 75,pattern: @number}
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
>>@object_key::{id: 51,pattern: @color | @group | @trigger_function}
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
>>@object_key::{pattern: @color,id: 23}
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
>>@object_key::{pattern: @number,id: 52}
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
>>@object_key::{pattern: @string,id: 31}
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
>>@object_key::{pattern: @number,id: 9}
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
>>@object_key::{pattern: @bool,id: 5}
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
>>@object_key::{pattern: @number,id: 72}
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
>>@object_key::{id: 3,pattern: @number}
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
>>@object_key::{pattern: @number,id: 54}
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
>>@object_key::{id: 92,pattern: @number}
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
>>@object_key::{pattern: @number,id: 24}
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
>>@object_key::{id: 25,pattern: @number}
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
