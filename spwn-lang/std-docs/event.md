  
# **@event**: 
 
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
