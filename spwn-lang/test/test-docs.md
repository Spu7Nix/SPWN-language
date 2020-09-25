# Documentation for `test` 
_This file was generated using `spwn doc [file name]`_
## Info:

- Uses 0 groups
- Uses 1 colors
- Uses 0 block IDs
- Uses 0 item IDs

- Adds 0 objects
## Exports:
**Type:** `dictionary` 

**Literal:** 

 ```

{
wait: (time: @number) { /* code omitted */ },
hello: hello
}

``` 

<details>
<summary> View members </summary>


## Macros:


**`wait`**:

>**Type:** `macro` 
>
>**Literal:** ```(time: @number) { /* code omitted */ }``` 
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

**`hello`**:

>**Type:** `string` 
>
>**Literal:** ```hello``` 
>
>
>


</details>

</details>


## Type Implementations:
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
>**Literal:** ```(self, comparison: @number, other: @number, function: @function) { /* code omitted */ }``` 
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

### **@counter**: 
 <details>
<summary> View members </summary>

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

**`multiply`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, factor, speed: @number = 3) { /* code omitted */ }``` 
>
>## Description: 
> _Multiplies the value of the counter by some factor_
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

**`subtract_from`**:

>**Type:** `macro` 
>
>**Literal:** ```(self, items: @array, speed: @number = 3) { /* code omitted */ }``` 
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
>
>
</details>

