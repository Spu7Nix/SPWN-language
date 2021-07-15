  
# **@group**: 
 
## **\_range\_**:

> **Value:** 
>```spwn
>(self, other: @group) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the range operator (`..`) for groups_
>### Example: 
>```spwn
> for group in 1g..10g {
>    -> group.move(10, 0, 0.5)
>}
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`other`** | @group | | |
>

## **alpha**:

> **Value:** 
>```spwn
>(self, opacity: @number = 1, duration: @number = 0) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the alpha trigger_
>### Example: 
>```spwn
> 1g.alpha(0)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `opacity` | @number | `1` | |
>| 2 | `duration` | @number | `0` | |
>

## **follow**:

> **Value:** 
>```spwn
>(self, other: @group, x_mod: @number = 1, y_mod: @number = 1, duration: @number = 999) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the follow trigger_
>### Example: 
>```spwn
> 10g.follow(11g)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`other`** | @group | |Group of object to follow |
>| 2 | `x_mod` | @number | `1` |Multiplier for the movement on the X-axis |
>| 3 | `y_mod` | @number | `1` |Multiplier for the movement on the Y-axis |
>| 4 | `duration` | @number | `999` |Duration of following |
>

## **follow\_player\_y**:

> **Value:** 
>```spwn
>(self, speed: @number = 1, delay: @number = 0, offset: @number = 0, max_speed: @number = 0, duration: @number = 999) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the follow player Y trigger_
>### Example: 
>```spwn
> 10g.follow_player_y(delay = 0.5)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `speed` | @number | `1` |Interpolation factor (?) |
>| 2 | `delay` | @number | `0` |Delay of movement |
>| 3 | `offset` | @number | `0` |Offset on the Y-axis |
>| 4 | `max_speed` | @number | `0` |Maximum speed |
>| 5 | `duration` | @number | `999` |Duration of following |
>

## **lock\_to\_player**:

> **Value:** 
>```spwn
>(self, lock_x: @bool = true, lock_y: @bool = true, duration: @number = 999) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Lock group to player position_
>### Example: 
>```spwn
> 10g.lock_to_player(lock_x = true, duration = 20)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `lock_x` | @bool | `true` |Lock to player X |
>| 2 | `lock_y` | @bool | `true` |Lock to player Y |
>| 3 | `duration` | @number | `999` |Duration of lock |
>

## **move**:

> **Value:** 
>```spwn
>(self, x: @number, y: @number, duration: @number = 0, easing: @easing_type = @easing_type::{id: 0}, easing_rate: @number = 2) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the move trigger_
>### Example: 
>```spwn
> 10g.move(100, 0, 0.5, easing = EASE_IN_OUT)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`x`** | @number | |Units to move on the X axis |
>| 2 | **`y`** | @number | |Units to move on the Y axis |
>| 3 | `duration` | @number | `0` |Duration of movement |
>| 4 | `easing` | @easing_type | `@easing_type::{id: 0}` | |
>| 5 | `easing_rate` | @number | `2` | |
>

## **move\_to**:

> **Value:** 
>```spwn
>(self, target: @group, duration: @number = 0, x_only: @bool = false, y_only: @bool = false, easing: @easing_type = @easing_type::{id: 0}, easing_rate: @number = 2) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the 'Move target' feature of the move trigger. Remember that both groups can only contain one object._
>### Example: 
>```spwn
> 10g.move_to(20g)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`target`** | @group | |Group of the object to move to |
>| 2 | `duration` | @number | `0` |Duration of movement |
>| 3 | `x_only` | @bool | `false` |Will move to the object only on the X-axis |
>| 4 | `y_only` | @bool | `false` |Will move to the object only on the y-axis |
>| 5 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>| 6 | `easing_rate` | @number | `2` |Easing rate |
>

## **pulse**:

> **Value:** 
>```spwn
>(self, r: @number, g: @number, b: @number, fade_in: @number = 0, hold: @number = 0, fade_out: @number = 0, exclusive: @bool = false, hsv: @bool = false, s_checked: @bool = false, b_checked: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the pulse trigger for groups_
>### Example: 
>```spwn
> 10g.pulse(255, 0, 0, fade_out = 0.5)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`r`** | @number | |Red value of pulse color (or hue if HSV is enabled) |
>| 2 | **`g`** | @number | |Green value of pulse color (or saturation if HSV is enabled) |
>| 3 | **`b`** | @number | |Blue value of pulse color (or brightness/value if HSV is enabled) |
>| 4 | `fade_in` | @number | `0` |Fade-in duration |
>| 5 | `hold` | @number | `0` |Duration to hold the color |
>| 6 | `fade_out` | @number | `0` |Fade-out duration |
>| 7 | `exclusive` | @bool | `false` |Weather to prioritize this pulse over simultaneous pulses |
>| 8 | `hsv` | @bool | `false` |Toggle HSV mode |
>| 9 | `s_checked` | @bool | `false` |HSV specific: saturation checked |
>| 10 | `b_checked` | @bool | `false` |HSV specific: brightness checked |
>

## **rotate**:

> **Value:** 
>```spwn
>(self, center: @group, degrees: @number, duration: @number = 0, easing: @easing_type = @easing_type::{id: 0}, easing_rate: @number = 2, lock_object_rotation: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the rotate trigger_
>### Example: 
>```spwn
> center = 3g
>10g.rotate(center, 360, 2, easing = EASE_IN_OUT)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`center`** | @group | |Group of object to rotate around |
>| 2 | **`degrees`** | @number | |Rotation in degrees |
>| 3 | `duration` | @number | `0` |Duration of rotation |
>| 4 | `easing` | @easing_type | `@easing_type::{id: 0}` |Easing type |
>| 5 | `easing_rate` | @number | `2` |Easing rate |
>| 6 | `lock_object_rotation` | @bool | `false` |Only rotate positions of the objects, not the textures |
>

## **stop**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the stop trigger_
>### Example: 
>```spwn
> move = !{
>    10g.move(1000, 0, 10)
>}
>move!
>wait(2)
>move.start_group.stop()
>```
>

## **toggle\_off**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Toggles the group off_
>### Example: 
>```spwn
> 10g.toggle_off()
>```
>

## **toggle\_on**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Toggles the group on_
>### Example: 
>```spwn
> 10g.toggle_on()
>```
>
