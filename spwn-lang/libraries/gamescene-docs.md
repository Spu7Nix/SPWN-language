# Documentation for `gamescene` 
_This file was generated using `spwn doc [file name]`_
## Info:

- Uses 3 groups
- Uses 0 colors
- Uses 2 block IDs
- Uses 1 item IDs

- Adds 2 objects
## Exports:
**Type:** `dictionary` 

**Literal:** 

 ```

{
button_a: () { /* code omitted */ },
button_a_end: () { /* code omitted */ },
button_b: () { /* code omitted */ },
button_b_end: () { /* code omitted */ },
hidden_group: ?g
}

``` 

<details>
<summary> View members </summary>


## Macros:


**`button_a`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when button A is pressed (the right side by default)_
>
>

**`button_a_end`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when button A is released (the right side by default)_
>
>

**`button_b`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when button B is pressed (the left side by default)_
>
>

**`button_b_end`**:

>**Type:** `macro` 
>
>**Literal:** ```() { /* code omitted */ }``` 
>
>## Description: 
> _Returns an event for when button B is released (the left side by default)_
>
>
## Other values:

<details>
<summary> View </summary>

**`hidden_group`**:

>**Type:** `group` 
>
>**Literal:** ```?g``` 
>
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
>**Literal:** ```(self, opacity: @number = 1, duration: @number = 0) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @group, x_mod: @number = 1, y_mod: @number = 1, duration: @number = 999) { /* code omitted */ }``` 
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
>**Literal:** ```(self, speed: @number = 1, delay: @number = 0, offset: @number = 0, max_speed: @number = 0, duration: @number = 999) { /* code omitted */ }``` 
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

**`lock_to_player`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, lock_x: @bool = true, lock_y: @bool = true, duration: @number = 999) { /* code omitted */ }``` 
>
>## Description: 
> _Lock group to player position_
>## Arguments:
>> _`lock_x` (optional)_ : _Lock to player X_
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
>> _`lock_y` (optional)_ : _Lock to player Y_
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
>> _`duration` (optional)_ : _Duration of lock_
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
>**Literal:** 
>
> ```
>
>(self, x: @number, y: @number, duration: @number = 0, easing: @easing_type = {
>type: @easing_type,
>id: 0
>}, easing_rate: @number = 2) { /* code omitted */ }
>
>``` 
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
>>**Type:** `easing_type` 
>>
>>**Literal:** 
>>
>> ```
>>
>>{
>>type: @easing_type,
>>id: 0
>>}
>>
>>``` 
>>
>><details>
>><summary> View members </summary>
>>
>>**`id`**:
>>
>>>**Type:** `number` 
>>>
>>>**Literal:** ```0``` 
>>>
>>>
>>>
>>
>>**`type`**:
>>
>>>**Type:** `type_indicator` 
>>>
>>>**Literal:** ```@easing_type``` 
>>>
>>>
>>>
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
>**Literal:** 
>
> ```
>
>(self, target: @group, duration: @number = 0, x_only: @bool = false, y_only: @bool = false, easing: @easing_type = {
>type: @easing_type,
>id: 0
>}, easing_rate: @number = 2) { /* code omitted */ }
>
>``` 
>
>## Description: 
> _Implementation of the 'Move target' feature of the move trigger_
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
>>**Type:** `easing_type` 
>>
>>**Literal:** 
>>
>> ```
>>
>>{
>>type: @easing_type,
>>id: 0
>>}
>>
>>``` 
>>
>><details>
>><summary> View members </summary>
>>
>>**`id`**:
>>
>>>**Type:** `number` 
>>>
>>>**Literal:** ```0``` 
>>>
>>>
>>>
>>
>>**`type`**:
>>
>>>**Type:** `type_indicator` 
>>>
>>>**Literal:** ```@easing_type``` 
>>>
>>>
>>>
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
>**Literal:** ```(self, r: @number, g: @number, b: @number, fade_in: @number = 0, hold: @number = 0, fade_out: @number = 0, exclusive: @bool = false, hsv: @bool = false) { /* code omitted */ }``` 
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
>**Literal:** 
>
> ```
>
>(self, center: @group, degrees: @number, duration: @number = 0, easing: @easing_type = {
>id: 0,
>type: @easing_type
>}, easing_rate: @number = 2, lock_object_rotation: @bool = false) { /* code omitted */ }
>
>``` 
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
>>**Type:** `easing_type` 
>>
>>**Literal:** 
>>
>> ```
>>
>>{
>>id: 0,
>>type: @easing_type
>>}
>>
>>``` 
>>
>><details>
>><summary> View members </summary>
>>
>>**`id`**:
>>
>>>**Type:** `number` 
>>>
>>>**Literal:** ```0``` 
>>>
>>>
>>>
>>
>>**`type`**:
>>
>>>**Type:** `type_indicator` 
>>>
>>>**Literal:** ```@easing_type``` 
>>>
>>>
>>>
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
>**Literal:** ```(self, r: @number, g: @number, b: @number, fade_in: @number = 0, hold: @number = 0, fade_out: @number = 0, exclusive: @bool = false, hsv: @bool = false) { /* code omitted */ }``` 
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
>**Literal:** ```(self, r: @number, g: @number, b: @number, duration: @number = 0, opacity: @number = 1, blending: @bool = false) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @block) { /* code omitted */ }``` 
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
>**Literal:** ```(self, amount: @number) { /* code omitted */ }``` 
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
>**Literal:** ```(self, number: @number = 0) { /* code omitted */ }``` 
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
>**Literal:** ```(self, comparison: @comparison, other: @number, function: @function) { /* code omitted */ }``` 
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
>> **`function`** _(obligatory)_: _Target function if comparison is 'true'_
>
>
>
>
>
>
</details>

### **@array**: 
 <details>
<summary> View members </summary>

**`contains`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, el) { /* code omitted */ }``` 
>
>## Arguments:
>> **`el`** _(obligatory)_
>
>
>
>
>
>

**`max`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, minval = 0) { /* code omitted */ }``` 
>
>## Arguments:
>> _`minval` (optional)_ 
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

**`min`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, max_val = 999999999999) { /* code omitted */ }``` 
>
>## Arguments:
>> _`max_val` (optional)_ 
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```999999999999``` 
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

**`push`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, value) { /* code omitted */ }``` 
>
>## Arguments:
>> **`value`** _(obligatory)_
>
>
>
>
>
>
</details>

### **@counter**: 
 <details>
<summary> View members </summary>

**`_add_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`_as_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, _type: @type_indicator) { /* code omitted */ }``` 
>
>## Arguments:
>> **`_type`** _(obligatory)_
>
>
>
>
>
>

**`_assign_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`_divide_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`_divided_by_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`_equal_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_multiply_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`_not_equal_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
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
>**Literal:** ```(self, other: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>
>

**`_subtract_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`_times_`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number | @counter) { /* code omitted */ }``` 
>
>## Arguments:
>> **`num`** _(obligatory)_
>
>
>
>
>
>

**`add`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, num: @number) { /* code omitted */ }``` 
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
>**Literal:** ```(self, items: @array, speed: @number = 3, factor: @number = 1) { /* code omitted */ }``` 
>
>## Description: 
> _Adds the counter's value to all item IDs in a list, and resets the counter to 0 in the process_
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
>>**Literal:** ```3``` 
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
>**Literal:** ```(self, speed: @number = 3) { /* code omitted */ }``` 
>
>## Description: 
> _Copies the counter and returns the copy_
>## Arguments:
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```3``` 
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

**`compare`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, other: @counter, speed: @number = 3) { /* code omitted */ }``` 
>
>## Arguments:
>> **`other`** _(obligatory)_
>
>
>
>
>> _`speed` (optional)_ 
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```3``` 
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
>**Literal:** ```(self, items: [@item | @counter], speed: @number = 3, factor: @number = 1) { /* code omitted */ }``` 
>
>## Description: 
> _Copies the value of the counter to another item ID, without consuming the original_
>## Arguments:
>> **`items`** _(obligatory)_: _Items to copy to_
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
>>**Literal:** ```3``` 
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
>**Literal:** 
>
> ```
>
>(self, divisor: @counter | @number, remainder: @counter | @item = {
>type: @counter,
>item: ?i
>}, speed: @number = 3) { /* code omitted */ }
>
>``` 
>
>## Description: 
> _Devides the value of the counter by some divisor_
>## Arguments:
>> **`divisor`** _(obligatory)_: _Divisor to divide by, either another counter (very expensive) or a normal number_
>
>
>
>
>> _`remainder` (optional)_ : _Counter or item to set to the remainder value_
>>
>>_Default value:_
>>
>>**Type:** `counter` 
>>
>>**Literal:** 
>>
>> ```
>>
>>{
>>type: @counter,
>>item: ?i
>>}
>>
>>``` 
>>
>><details>
>><summary> View members </summary>
>>
>>**`item`**:
>>
>>>**Type:** `item` 
>>>
>>>**Literal:** ```?i``` 
>>>
>>>
>>>
>>
>>**`type`**:
>>
>>>**Type:** `type_indicator` 
>>>
>>>**Literal:** ```@counter``` 
>>>
>>>
>>>
>>
>>
>>
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
>>**Literal:** ```3``` 
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
>**Literal:** ```(self, factor: @counter | @number, speed: @number = 3) { /* code omitted */ }``` 
>
>## Description: 
> _Multiplies the value of the counter by some factor (does not consume the factor)_
>## Arguments:
>> **`factor`** _(obligatory)_: _Factor to multiply by, either another counter (very expensive) or a normal number_
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
>>**Literal:** ```3``` 
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

**`new`**:

>**Type:** `macro` 
>
>**Literal:** ```(source: @number | @item | @bool = 0, delay: @bool = true) { /* code omitted */ }``` 
>
>## Description: 
> _Creates a new counter_
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
>> _`delay` (optional)_ : _Adds a delay if a value gets added to the new item (to avoid confusing behavior)_
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
>
>

**`reset`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, speed: @number = 3) { /* code omitted */ }``` 
>
>## Description: 
> _Resets counter to 0_
>## Arguments:
>> _`speed` (optional)_ : _Speed of operation (higher number increases group usage)_
>>
>>_Default value:_
>>
>>**Type:** `number` 
>>
>>**Literal:** ```3``` 
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
>**Literal:** ```(self, items: @array, speed: @number = 3, factor: @number = 1) { /* code omitted */ }``` 
>
>## Description: 
> _Subtracts the counter's value from all item IDs in a list, and resets the counter to 0 in the process_
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
>>**Literal:** ```3``` 
>>
>>
>>
>>
>
>
>
>
>> _`factor` (optional)_ : _Multiplyer for the value subtracted_
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
>**Literal:** ```(self, range: [@number] | @range) { /* code omitted */ }``` 
>
>## Description: 
> _Converts the counter into a normal number (very context-splitting, be careful)_
>## Arguments:
>> **`range`** _(obligatory)_: _Array or range of possible output values_
>
>
>
>
>
>
</details>

