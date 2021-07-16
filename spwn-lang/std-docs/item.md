  
# **@item**: 
 
## **\_range\_**:

> **Value:** 
>```spwn
>(self, other: @item) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the range operator (`..`) for item IDs_
>### Example: 
>```spwn
> for item in 1i..10i {
>    item.add(10)
>}
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`other`** | @item | | |
>

## **add**:

> **Value:** 
>```spwn
>(self, amount: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the pickup trigger_
>### Example: 
>```spwn
> 10i.add(5)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`amount`** | @number | |Amount to add |
>

## **count**:

> **Value:** 
>```spwn
>(self, number: @number = 0) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the count trigger (-> returns an event for when an item reaches a certain value)_
>### Example: 
>```spwn
> on(10i.count(100), !{
>    BG.pulse(0, 255, 0, fade_out = 0.5) // will pulse each time item ID 10 becomes 100
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `number` | @number | `0` |Number to check against |
>

## **if\_is**:

> **Value:** 
>```spwn
>(self, comparison: @comparison, other: @number, function: @trigger_function) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Implementation of the instant count trigger_
>### Example: 
>```spwn
> 10i.if_is(EQUAL_TO, 5, !{
>    BG.pulse(255, 0, 0, fade_out = 0.5)
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`comparison`** | @comparison | |Comparison mode |
>| 2 | **`other`** | @number | |Number to compare with |
>| 3 | **`function`** | @trigger_function | |Target function if comparison is 'true' |
>
