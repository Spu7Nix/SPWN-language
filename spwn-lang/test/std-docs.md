# Documentation for `std` 
_This file was generated using `spwn doc [file name]`_
## Exports:
**Type:** `dictionary` 

**Literal:** 

 ```

{
EASE_IN_OUT: 1,
collision: (a, b) { /* code omitted */ },
enable_trail: () { /* code omitted */ },
hide_player: () { /* code omitted */ },
ELASTIC_IN_OUT: 4,
BOUNCE_IN: 8,
ELASTIC_IN: 5,
BOUNCE_IN_OUT: 7,
BACK_IN: 17,
EQUAL_TO: 0,
... (30 more)
}

``` 

<details>
<summary> View members </summary>


## Macros:


**`call_with_delay`**:

>**Type:** `macro` 
>
>**Literal:** ```(time, function) { /* code omitted */ }``` 
>
>## Description: 
> _Call a function after a delay_
>## Arguments:
>> **`time`** _(obligatory)_: _Delay time in seconds_
>
>
>
>
>> **`function`** _(obligatory)_: _Function to call after the delay_
>
>
>
>
>
>

**`collision`**:

>**Type:** `macro` 
>
>**Literal:** ```(a, b) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the collision trigger (returns an event)_
>## Arguments:
>> **`a`** _(obligatory)_: _Block A ID_
>
>
>
>
>> **`b`** _(obligatory)_: _Block B ID_
>
>
>
>
>
>

**`collision_exit`**:

>**Type:** `macro` 
>
>**Literal:** ```(a, b) { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when a collision exits_
>## Arguments:
>> **`a`** _(obligatory)_: _Block A ID_
>
>
>
>
>> **`b`** _(obligatory)_: _Block B ID_
>
>
>
>
>
>

**`death`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when the player dies_
>
>

**`disable_trail`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Disables the players trail_
>
>

**`enable_trail`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Enables the players trail_
>
>

**`hide_player`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Hides the player_
>
>

**`loop`**:

>**Type:** `macro` 
>
>**Literal:** ```(start_val, end_val, code, delay: 0.05, reset: true, reset_speed: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of a spawn loop_
>## Arguments:
>> **`start_val`** _(obligatory)_: _Start value for the iterator_
>
>
>
>
>> **`end_val`** _(obligatory)_: _End value for the iterator_
>
>
>
>
>> **`code`** _(obligatory)_: _Macro of the code that gets looped, should take the iterator (a mut) as the first argument_
>
>
>
>
>> _`delay` (optional)_ : _Delay between loops (less than 0.05 may be unstable)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0.05``` 
>>
>>
>>
>>
>
>
>
>
>> _`reset` (optional)_ : _Weather to reset the iterator after looping (only disable if the loop is only triggered once)_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```true``` 
>>
>>
>>
>>
>
>
>
>
>> _`reset_speed` (optional)_ : _Operation speed of the reset of the iterator, if enabled_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`mut`**:

>**Type:** `macro` 
>
>**Literal:** ```(source: 0) { /* code omitted */ }``` 
>
>## Description: 
> _Creates a new mut_
>## Arguments:
>> _`source` (optional)_ : _Source (can be a number, item ID or boolean)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`on`**:

>**Type:** `macro` 
>
>**Literal:** ```(event, function) { /* code omitted */ }``` 
>
>## Description: 
> _Triggers a function every time an event fires_
>## Arguments:
>> **`event`** _(obligatory)_: _Event to trigger on_
>
>
>
>
>> **`function`** _(obligatory)_: _Function to trigger_
>
>
>
>
>
>

**`shake`**:

>**Type:** `macro` 
>
>**Literal:** ```(strength: 1, interval: 0, duration: 0.5) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the shake trigger_
>## Arguments:
>> _`strength` (optional)_ : _Strength value_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`interval` (optional)_ : _Interval value_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`duration` (optional)_ : _Duration of shake_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0.5``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`show_player`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Shows the player_
>
>

**`supress_signal`**:

>**Type:** `macro` 
>
>**Literal:** ```(delay) { /* code omitted */ }``` 
>
>## Description: 
> _Stops signal from coming past for some time_
>## Arguments:
>> **`delay`** _(obligatory)_: _Time to supress signal_
>
>
>
>
>
>

**`toggle_bg_effect`**:

>**Type:** `macro` 
>
>**Literal:** ```(on: false) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the bg effect on/off triggers_
>## Arguments:
>> _`on` (optional)_ : _Weather to toggle bg effect on or off_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`touch`**:

>**Type:** `macro` 
>
>**Literal:** ```(dual_side: false) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the touch trigger (returns an event)_
>## Arguments:
>> _`dual_side` (optional)_ : _Dual mode (only check for touch on the dual side)_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`touch_end`**:

>**Type:** `macro` 
>
>**Literal:** ```(dual_side: false) { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when a touch ends_
>## Arguments:
>> _`dual_side` (optional)_ : _Dual mode (only check for touch on the dual side)_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`wait`**:

>**Type:** `macro` 
>
>**Literal:** ```(time) { /* code omitted */ }``` 
>
>## Description: 
> _Adds a delay before the next triggers_
>## Arguments:
>> **`time`** _(obligatory)_: _Delay time in seconds_
>
>
>
>
>
>
## Other values:

<details>
<summary> View </summary>

**`BACK_IN`**:

>**Type:** `number` 
>
>**Literal:** ```17``` 
>
>
>

**`BACK_IN_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```16``` 
>
>
>

**`BACK_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```18``` 
>
>
>

**`BG`**:

>**Type:** `color` 
>
>**Literal:** ```1000c``` 
>
>
>

**`BOUNCE_IN`**:

>**Type:** `number` 
>
>**Literal:** ```8``` 
>
>
>

**`BOUNCE_IN_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```7``` 
>
>
>

**`BOUNCE_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```9``` 
>
>
>

**`EASE_IN`**:

>**Type:** `number` 
>
>**Literal:** ```2``` 
>
>
>

**`EASE_IN_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```1``` 
>
>
>

**`EASE_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```3``` 
>
>
>

**`ELASTIC_IN`**:

>**Type:** `number` 
>
>**Literal:** ```5``` 
>
>
>

**`ELASTIC_IN_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```4``` 
>
>
>

**`ELASTIC_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```6``` 
>
>
>

**`EQUAL_TO`**:

>**Type:** `number` 
>
>**Literal:** ```0``` 
>
>
>

**`EXPONENTIAL_IN`**:

>**Type:** `number` 
>
>**Literal:** ```11``` 
>
>
>

**`EXPONENTIAL_IN_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```10``` 
>
>
>

**`EXPONENTIAL_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```12``` 
>
>
>

**`LARGER_THAN`**:

>**Type:** `number` 
>
>**Literal:** ```1``` 
>
>
>

**`NONE`**:

>**Type:** `number` 
>
>**Literal:** ```0``` 
>
>
>

**`SINE_IN`**:

>**Type:** `number` 
>
>**Literal:** ```14``` 
>
>
>

**`SINE_IN_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```13``` 
>
>
>

**`SINE_OUT`**:

>**Type:** `number` 
>
>**Literal:** ```15``` 
>
>
>

**`SMALLER_THAN`**:

>**Type:** `number` 
>
>**Literal:** ```2``` 
>
>
>

**`obj_props`**:

>**Type:** `dictionary` 
>
>**Literal:** 
>
> ```
>
>{
>TARGET_POS: 71,
>COLOR_2: 22,
>X_MOD: 72,
>DISABLE_ROTATION: 98,
>VERTICAL_FLIP: 5,
>PULSE_MODE: 48,
>HOLD: 46,
>COPIED_COLOR_HVS: 49,
>TRIGGER_GREEN: 8,
>COLOR: 21,
>... (83 more)
>}
>
>``` 
>
><details>
><summary> View members </summary>
>
>**`ACTIVATE_GROUP`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```56``` 
>>
>>
>>
>
>**`ACTIVATE_ON_EXIT`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```93``` 
>>
>>
>>
>
>**`ANIMATION_ID`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```76``` 
>>
>>
>>
>
>**`ANIMATION_SPEED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```107``` 
>>
>>
>>
>
>**`BLENDING`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```17``` 
>>
>>
>>
>
>**`BLOCK_A`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```80``` 
>>
>>
>>
>
>**`BLOCK_B`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```95``` 
>>
>>
>>
>
>**`CENTER`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```71``` 
>>
>>
>>
>
>**`COLOR`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```21``` 
>>
>>
>>
>
>**`COLOR_2`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```22``` 
>>
>>
>>
>
>**`COLOR_2_HVS`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```44``` 
>>
>>
>>
>
>**`COLOR_2_HVS_ENABLED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```42``` 
>>
>>
>>
>
>**`COMPARISON`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```88``` 
>>
>>
>>
>
>**`COPIED_COLOR_HVS`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```49``` 
>>
>>
>>
>
>**`COPIED_COLOR_ID`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```50``` 
>>
>>
>>
>
>**`COPY_OPACTITY`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```60``` 
>>
>>
>>
>
>**`COUNT`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```77``` 
>>
>>
>>
>
>**`COUNT_MULTI_ACTIVATE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```104``` 
>>
>>
>>
>
>**`DELAY`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```91``` 
>>
>>
>>
>
>**`DETAIL_ONLY`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```66``` 
>>
>>
>>
>
>**`DISABLE_ROTATION`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```98``` 
>>
>>
>>
>
>**`DONT_ENTER`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```67``` 
>>
>>
>>
>
>**`DONT_FADE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```64``` 
>>
>>
>>
>
>**`DUAL_MODE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```89``` 
>>
>>
>>
>
>**`DURATION`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```10``` 
>>
>>
>>
>
>**`DYNAMIC_BLOCK`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```94``` 
>>
>>
>>
>
>**`EASING`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```30``` 
>>
>>
>>
>
>**`EASING_RATE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```85``` 
>>
>>
>>
>
>**`EDITOR_DISABLE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```102``` 
>>
>>
>>
>
>**`EDITOR_LAYER_1`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```20``` 
>>
>>
>>
>
>**`EDITOR_LAYER_2`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```61``` 
>>
>>
>>
>
>**`EXCLUSIVE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```86``` 
>>
>>
>>
>
>**`FADE_IN`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```45``` 
>>
>>
>>
>
>**`FADE_OUT`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```47``` 
>>
>>
>>
>
>**`FOLLOW`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```71``` 
>>
>>
>>
>
>**`GLOW_DISABLED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```96``` 
>>
>>
>>
>
>**`GROUPS`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```57``` 
>>
>>
>>
>
>**`GROUP_PARENT`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```34``` 
>>
>>
>>
>
>**`HIGH_DETAIL`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```103``` 
>>
>>
>>
>
>**`HOLD`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```46``` 
>>
>>
>>
>
>**`HOLD_MODE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```81``` 
>>
>>
>>
>
>**`HVS`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```43``` 
>>
>>
>>
>
>**`HVS_ENABLED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```41``` 
>>
>>
>>
>
>**`INTERVAL`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```84``` 
>>
>>
>>
>
>**`ITEM`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```80``` 
>>
>>
>>
>
>**`LINKED_GROUP`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```108``` 
>>
>>
>>
>
>**`LOCK_OBJECT_ROTATION`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```70``` 
>>
>>
>>
>
>**`LOCK_TO_PLAYER_X`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```58``` 
>>
>>
>>
>
>**`LOCK_TO_PLAYER_Y`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```59``` 
>>
>>
>>
>
>**`MAIN_ONLY`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```65``` 
>>
>>
>>
>
>**`MAX_SPEED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```105``` 
>>
>>
>>
>
>**`MOVE_X`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```28``` 
>>
>>
>>
>
>**`MOVE_Y`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```29``` 
>>
>>
>>
>
>**`MULTI_TRIGGER`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```87``` 
>>
>>
>>
>
>**`OBJ_ID`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>
>**`OPACITY`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```35``` 
>>
>>
>>
>
>**`PICKUP_MODE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```79``` 
>>
>>
>>
>
>**`PLAYER_COLOR_1`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```15``` 
>>
>>
>>
>
>**`PLAYER_COLOR_2`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```16``` 
>>
>>
>>
>
>**`PORTAL_CHECKED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```13``` 
>>
>>
>>
>
>**`PULSE_MODE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```48``` 
>>
>>
>>
>
>**`RANDOMIZE_START`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```106``` 
>>
>>
>>
>
>**`ROTATE_DEGREES`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```68``` 
>>
>>
>>
>
>**`ROTATION`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```6``` 
>>
>>
>>
>
>**`ROTATION_SPEED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```97``` 
>>
>>
>>
>
>**`SCALING`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```32``` 
>>
>>
>>
>
>**`SPAWN_DURATION`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```63``` 
>>
>>
>>
>
>**`SPAWN_TRIGGERED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```62``` 
>>
>>
>>
>
>**`SPEED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```90``` 
>>
>>
>>
>
>**`STRENGTH`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```75``` 
>>
>>
>>
>
>**`SUBTRACT_COUNT`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```78``` 
>>
>>
>>
>
>**`TARGET`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```51``` 
>>
>>
>>
>
>**`TARGET_COLOR`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```23``` 
>>
>>
>>
>
>**`TARGET_POS`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```71``` 
>>
>>
>>
>
>**`TARGET_POS_AXES`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```101``` 
>>
>>
>>
>
>**`TARGET_TYPE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```52``` 
>>
>>
>>
>
>**`TEXT`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```31``` 
>>
>>
>>
>
>**`TIMES_360`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```69``` 
>>
>>
>>
>
>**`TOGGLE_MODE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```82``` 
>>
>>
>>
>
>**`TOUCH_TRIGGERED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```11``` 
>>
>>
>>
>
>**`TRIGGER_BLUE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```9``` 
>>
>>
>>
>
>**`TRIGGER_GREEN`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```8``` 
>>
>>
>>
>
>**`TRIGGER_RED`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```7``` 
>>
>>
>>
>
>**`USE_TARGET`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```100``` 
>>
>>
>>
>
>**`VERTICAL_FLIP`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```5``` 
>>
>>
>>
>
>**`VORIZONTAL_FLIP`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```4``` 
>>
>>
>>
>
>**`X`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```2``` 
>>
>>
>>
>
>**`X_MOD`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```72``` 
>>
>>
>>
>
>**`Y`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```3``` 
>>
>>
>>
>
>**`YELLOW_TELEPORTATION_PORTAL_DISTANCE`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```54``` 
>>
>>
>>
>
>**`Y_MOD`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```73``` 
>>
>>
>>
>
>**`Y_OFFSET`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```92``` 
>>
>>
>>
>
>**`Z_LAYER`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```24``` 
>>
>>
>>
>
>**`Z_ORDER`**:
>
>>**Type:** `number` 
>>
>>**Literal:** ```25``` 
>>
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

**`alpha`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, opacity: 1, duration: 0) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the alpha trigger_
>## Arguments:
>> _`opacity` (optional)_ 
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`duration` (optional)_ 
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`follow`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other, x_mod: 1, y_mod: 1, duration: 999) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the follow trigger_
>## Arguments:
>> **`other`** _(obligatory)_: _Group of object to follow_
>
>
>
>
>> _`x_mod` (optional)_ : _Multiplier for the movement on the X-axis_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`y_mod` (optional)_ : _Multiplier for the movement on the Y-axis_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`duration` (optional)_ : _Duration of following_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```999``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`follow_player_y`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, speed: 1, delay: 0, offset: 0, max_speed: 0, duration: 999) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the follow player Y trigger_
>## Arguments:
>> _`speed` (optional)_ : _Interpolation factor (?)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`delay` (optional)_ : _Delay of movement_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`offset` (optional)_ : _Offset on the Y-axis_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`max_speed` (optional)_ : _Maximum speed_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`duration` (optional)_ : _Duration of following_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```999``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`move`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, x, y, duration: 0, easing: 0, easing_rate: 2) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the move trigger_
>## Arguments:
>> **`x`** _(obligatory)_: _Units to move on the X axis_
>
>
>
>
>> **`y`** _(obligatory)_: _Units to move on the Y axis_
>
>
>
>
>> _`duration` (optional)_ : _Duration of movement_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`easing` (optional)_ 
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`easing_rate` (optional)_ 
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```2``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`move_to`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, target, duration: 0, x_only: false, y_only: false, easing: 0, easing_rate: 2) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the Move target feature of the move trigger_
>## Arguments:
>> **`target`** _(obligatory)_: _Group of the object to move to_
>
>
>
>
>> _`duration` (optional)_ : _Duration of movement_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`x_only` (optional)_ : _Will move to the object only on the X-axis_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>> _`y_only` (optional)_ : _Will move to the object only on the y-axis_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>> _`easing` (optional)_ : _Easing type_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`easing_rate` (optional)_ : _Easing rate_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```2``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`pulse`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, r, g, b, fade_in: 0, hold: 0, fade_out: 0, exclusive: false, hsv: false) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the pulse trigger for groups_
>## Arguments:
>> **`r`** _(obligatory)_: _Red value of pulse color (or hue if HSV is enabled)_
>
>
>
>
>> **`g`** _(obligatory)_: _Green value of pulse color (or saturation if HSV is enabled)_
>
>
>
>
>> **`b`** _(obligatory)_: _Blue value of pulse color (or brightness/value if HSV is enabled)_
>
>
>
>
>> _`fade_in` (optional)_ : _Fade-in duration_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`hold` (optional)_ : _Duration to hold the color_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`fade_out` (optional)_ : _Fade-out duration_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`exclusive` (optional)_ : _Weather to prioritize this pulse over simultaneous pulses_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>> _`hsv` (optional)_ : _Toggle HSV mode_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`rotate`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, center, degrees, duration: 0, easing: 0, easing_rate: 0, lock_object_rotation: false) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the rotate trigger_
>## Arguments:
>> **`center`** _(obligatory)_: _Group of object to rotate around_
>
>
>
>
>> **`degrees`** _(obligatory)_: _Rotation in degrees_
>
>
>
>
>> _`duration` (optional)_ : _Duration of rotation_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`easing` (optional)_ : _Easing type_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`easing_rate` (optional)_ : _Easing rate_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`lock_object_rotation` (optional)_ : _Only rotate positions of the objects, not the textures_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`stop`**:

>**Type:** `macro` 
>
>**Literal:** ```(self) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the stop trigger_
>
>

**`toggle_off`**:

>**Type:** `macro` 
>
>**Literal:** ```(self) { /* code omitted */ }``` 
>
>## Description: 
> _Toggles the group off_
>
>

**`toggle_on`**:

>**Type:** `macro` 
>
>**Literal:** ```(self) { /* code omitted */ }``` 
>
>## Description: 
> _Toggles the group on_
>
>
</details>

### **@color**: 
 <details>
<summary> View members </summary>

**`pulse`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, r, g, b, fade_in: 0, hold: 0, fade_out: 0, exclusive: false, hsv: false) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the pulse trigger for colors_
>## Arguments:
>> **`r`** _(obligatory)_: _Red value of pulse color (or hue if HSV is enabled)_
>
>
>
>
>> **`g`** _(obligatory)_: _Green value of pulse color (or saturation if HSV is enabled)_
>
>
>
>
>> **`b`** _(obligatory)_: _Blue value of pulse color (or brightness/value if HSV is enabled)_
>
>
>
>
>> _`fade_in` (optional)_ : _Fade-in duration_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`hold` (optional)_ : _Duration to hold the color_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`fade_out` (optional)_ : _Fade-out duration_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`exclusive` (optional)_ : _Weather to prioritize this pulse over simultaneous pulses_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>> _`hsv` (optional)_ : _Toggle HSV mode_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`set`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, r, g, b, duration: 0, opacity: 1, blending: false) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the color trigger_
>## Arguments:
>> **`r`** _(obligatory)_: _Red value of the target color_
>
>
>
>
>> **`g`** _(obligatory)_: _Green value of the target color_
>
>
>
>
>> **`b`** _(obligatory)_: _Blue value of the target color_
>
>
>
>
>> _`duration` (optional)_ : _Duration of color change_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>> _`opacity` (optional)_ : _Opacity of target color_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`blending` (optional)_ : _Toggle blending on target color_
>>
>>_Default value:_
>>
>>**Type:** `bool` 
>>
>>**Literal:** ```false``` 
>>
>>
>>
>>
>
>
>
>
>
>
</details>

### **@block**: 
 <details>
<summary> View members </summary>

**`create_tracker_item`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Description: 
> _Returns an item ID that is 1 when the blocks are colliding and 0 when they are not_
>## Arguments:
>> **`other`** _(obligatory)_: _Block ID to check against_
>
>
>
>
>
>
</details>

### **@item**: 
 <details>
<summary> View members </summary>

**`add`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, amount) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the pickup trigger_
>## Arguments:
>> **`amount`** _(obligatory)_: _Amount to add_
>
>
>
>
>
>

**`count`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, number: 0) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the count trigger (returns an event)_
>## Arguments:
>> _`number` (optional)_ : _Number to check against_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```0``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`if_is`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, comparison, other, function) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the instant count trigger_
>## Arguments:
>> **`comparison`** _(obligatory)_: _Comparison mode_
>
>
>
>
>> **`other`** _(obligatory)_: _Number to compare with_
>
>
>
>
>> **`function`** _(obligatory)_: _Target function if comparison is true_
>
>
>
>
>
>
</details>

### **@mut**: 
 <details>
<summary> View members </summary>

**`_as_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, _type) { /* code omitted */ }``` 
>
>## Arguments:
>> **`_type`** _(obligatory)_
>
>
>
>
>
>

**`_equal_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_less_or_equal_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_less_than_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_minus_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_more_or_equal_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_more_than_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_not_equal_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_plus_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`add`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num) { /* code omitted */ }``` 
>
>## Description: 
> _Implementation of the pickup trigger_
>## Arguments:
>> **`num`** _(obligatory)_: _Amount to add_
>
>
>
>
>
>

**`add_to`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, items, speed: 1, factor: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Adds the muts value to all item IDs in a list, and resets the mut to 0 in the process_
>## Arguments:
>> **`items`** _(obligatory)_: _Item IDs to add to_
>
>
>
>
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`factor` (optional)_ : _Multiplyer for the value added_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`clone`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, speed: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Copies the mut and returns the copy_
>## Arguments:
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`copy_to`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, item, speed: 1, factor: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Copies the value of the mut to another item ID, without consuming the original_
>## Arguments:
>> **`item`** _(obligatory)_: _Item ID to copy to_
>
>
>
>
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`factor` (optional)_ : _Factor of to multiply the copy by_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`divide`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, divisor, speed: 1, rest_item: 1i) { /* code omitted */ }``` 
>
>## Description: 
> _Devides the value of the mut by some divisor_
>## Arguments:
>> **`divisor`** _(obligatory)_: _Divisor to divide by, either another mut (very expensive) or a normal number_
>
>
>
>
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>> _`rest_item` (optional)_ : _Item ID to add the remainder of the division to_
>>
>>_Default value:_
>>
>>**Type:** `item` 
>>
>>**Literal:** ```1i``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`multiply`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, factor, speed: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Multiplies the value of the mut by some factor_
>## Arguments:
>> **`factor`** _(obligatory)_: _Factor to multiply by, either another mut (very expensive) or a normal number_
>
>
>
>
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`reset`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, speed: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Resets mut to 0_
>## Arguments:
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`subtract_from`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, items, speed: 1) { /* code omitted */ }``` 
>
>## Description: 
> _Subtracts the muts value from all item IDs in a list, and resets the mut to 0 in the process_
>## Arguments:
>> **`items`** _(obligatory)_: _Item IDs to add to_
>
>
>
>
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```1``` 
>>
>>
>>
>>
>
>
>
>
>
>

**`to_const`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, range) { /* code omitted */ }``` 
>
>## Description: 
> _Converts the mut into a normal number (very context-splitting, be careful)_
>## Arguments:
>> **`range`** _(obligatory)_: _Array of possible output values_
>
>
>
>
>
>
</details>

