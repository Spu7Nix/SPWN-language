  
# **@color**: 
 
## **\_range\_**:

> **Value:** 
>```spwn
>(self, other: @color) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the range operator (`..`) for colors_
>### Example: 
>```spwn
> for color in 1c..10c {
>    -> color.set(0,0,0, 0.5)
>}
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`other`** | @color | | |
>

## **pulse**:

> **Value:** 
>```spwn
>(self, r: @number, g: @number, b: @number, fade_in: @number = 0, hold: @number = 0, fade_out: @number = 0, exclusive: @bool = false, hsv: @bool = false, s_checked: @bool = false, b_checked: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
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

## **set**:

> **Value:** 
>```spwn
>(self, r: @number, g: @number, b: @number, duration: @number = 0, opacity: @number = 1, blending: @bool = false) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
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
>| 1 | **`r`** | @number | |Red value of the target color |
>| 2 | **`g`** | @number | |Green value of the target color |
>| 3 | **`b`** | @number | |Blue value of the target color |
>| 4 | `duration` | @number | `0` |Duration of color change |
>| 5 | `opacity` | @number | `1` |Opacity of target color |
>| 6 | `blending` | @bool | `false` |Toggle blending on target color |
>
