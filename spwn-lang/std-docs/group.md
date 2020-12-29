  

# **@group**: 
 
## **\_range\_**:

> **Value:** `(self, other: @group) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @group | | |
>  
>  
>

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
>  | 3 | `duration` | @number | `0` | |
>  
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
>  | 3 | `x_mod` | @number | `1` |Multiplier for the movement on the X-axis |
>  | 4 | `y_mod` | @number | `1` |Multiplier for the movement on the Y-axis |
>  | 5 | `duration` | @number | `999` |Duration of following |
>  
>  
>

## **follow\_player\_y**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the follow player Y trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `speed` | @number | `1` |Interpolation factor (?) |
>  | 3 | `delay` | @number | `0` |Delay of movement |
>  | 4 | `offset` | @number | `0` |Offset on the Y-axis |
>  | 5 | `max_speed` | @number | `0` |Maximum speed |
>  | 6 | `duration` | @number | `999` |Duration of following |
>  
>  
>

## **lock\_to\_player**:

> **Value:** `(self, lock_x: @bool = true, lock_y: @bool = true, duration: @number = 999) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Lock group to player position_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `lock_x` | @bool | `true` |Lock to player X |
>  | 3 | `lock_y` | @bool | `true` |Lock to player Y |
>  | 4 | `duration` | @number | `999` |Duration of lock |
>  
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
>  | 3 | **`y`** | @number | |Units to move on the Y axis |
>  | 4 | `duration` | @number | `0` |Duration of movement |
>  | 5 | `easing` | @easing_type | `@easing_type::{id: 0}` | |
>  | 6 | `easing_rate` | @number | `2` | |
>  
>  
>

## **move\_to**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the 'Move target' feature of the move trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`target`** | @group | |Group of the object to move to |
>  | 3 | `duration` | @number | `0` |Duration of movement |
>  | 4 | `x_only` | @bool | `false` |Will move to the object only on the X-axis |
>  | 5 | `y_only` | @bool | `false` |Will move to the object only on the y-axis |
>  | 6 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>  | 7 | `easing_rate` | @number | `2` |Easing rate |
>  
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
>  | 3 | **`g`** | @number | |Green value of pulse color (or saturation if HSV is enabled) |
>  | 4 | **`b`** | @number | |Blue value of pulse color (or brightness/value if HSV is enabled) |
>  | 5 | `fade_in` | @number | `0` |Fade-in duration |
>  | 6 | `hold` | @number | `0` |Duration to hold the color |
>  | 7 | `fade_out` | @number | `0` |Fade-out duration |
>  | 8 | `exclusive` | @bool | `false` |Weather to prioritize this pulse over simultaneous pulses |
>  | 9 | `hsv` | @bool | `false` |Toggle HSV mode |
>  
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
>  | 3 | **`degrees`** | @number | |Rotation in degrees |
>  | 4 | `duration` | @number | `0` |Duration of rotation |
>  | 5 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>  | 6 | `easing_rate` | @number | `2` |Easing rate |
>  | 7 | `lock_object_rotation` | @bool | `false` |Only rotate positions of the objects, not the textures |
>  
>  
>

## **stop**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the stop trigger_
>
>  
>

## **toggle\_off**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Toggles the group off_
>
>  
>

## **toggle\_on**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Toggles the group on_
>
>  
>
