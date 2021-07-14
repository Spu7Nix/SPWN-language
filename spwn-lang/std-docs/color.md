  

# **@color**: 
 
## **\_range\_**:

> **Value:** `(self, other: @color) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @color | | |
>  
>  
>

## **pulse**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the pulse trigger for colors_
>### Example: 
>```spwn
> BG.pulse(255, 0, 0, fade_out = 0.5) // pulses the background red
>```
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
>  | 10 | `s_checked` | @bool | `false` |HSV specific: saturation checked |
>  | 11 | `b_checked` | @bool | `false` |HSV specific: brightness checked |
>  
>  
>

## **set**:

> **Type:** `@macro` 
>
>## Description: 
> _Implementation of the color trigger_
>### Example: 
>```spwn
> BG.set(0, 0, 0, 0.5) // turns the background color black
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`r`** | @number | |Red value of the target color |
>  | 3 | **`g`** | @number | |Green value of the target color |
>  | 4 | **`b`** | @number | |Blue value of the target color |
>  | 5 | `duration` | @number | `0` |Duration of color change |
>  | 6 | `opacity` | @number | `1` |Opacity of target color |
>  | 7 | `blending` | @bool | `false` |Toggle blending on target color |
>  
>  
>
