  

# **@item**: 
 
## **\_range\_**:

> **Value:** `(self, other: @item) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @item | | |
>  
>  
>

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
>

## **count**:

> **Value:** `(self, number: @number = 0) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the count trigger (-> returns an event)_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `number` | @number | `0` |Number to check against |
>  
>  
>

## **if\_is**:

> **Value:** `(self, comparison: @comparison, other: @number, function: @trigger_function) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Implementation of the instant count trigger_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`comparison`** | @comparison | |Comparison mode |
>  | 3 | **`other`** | @number | |Number to compare with |
>  | 4 | **`function`** | @trigger_function | |Target function if comparison is 'true' |
>  
>  
>
