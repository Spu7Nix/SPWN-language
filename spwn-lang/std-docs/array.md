  

# **@array**: 
 
## **all**:

> **Value:** `(self, map: @macro = (a) { /* code omitted */ }) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Determines whether all the members of an array satisfy the specified callback._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `map` | @macro | `(a) { /* code omitted */ }` | |
>  
>  
>

## **any**:

> **Value:** `(self, map: @macro = (a) { /* code omitted */ }) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Determines whether the specified callback function returns true for any element of an array._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `map` | @macro | `(a) { /* code omitted */ }` | |
>  
>  
>

## **clear**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Clears the array._
>
>  
>

## **contains**:

> **Value:** `(self, el) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _See if array contains an element._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`el`** |any | | |
>  
>  
>

## **filter**:

> **Value:** `(self, cb: @macro) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns the elements of an array that meet the condition specified in the callback function._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`cb`** | @macro | | |
>  
>  
>

## **index**:

> **Value:** `(self, el) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Gets the index of an element, if it doesn't exists returns null._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`el`** |any | | |
>  
>  
>

## **map**:

> **Value:** `(self, cb: @macro) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Calls a defined callback function on each element of an array, and returns an array that contains the results._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`cb`** | @macro | | |
>  
>  
>

## **max**:

> **Value:** `(self, minval = -999999999999) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Gets the highest number in the array._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `minval` |any | `-999999999999` | |
>  
>  
>

## **min**:

> **Value:** `(self, max_val = 999999999999) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Gets the lowest number in the array._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `max_val` |any | `999999999999` | |
>  
>  
>

## **pop**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Removes the last value from the array and returns it._
>
>  
>

## **push**:

> **Value:** `(self, value) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Pushes a value to the end of the array._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`value`** |any | | |
>  
>  
>

## **reduce**:

> **Value:** `(self, cb: @macro) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Calls the specified callback function for all the elements in an array. The return value of the callback function is the accumulated result, and is provided as an argument in the next call to the callback function._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`cb`** | @macro | | |
>  
>  
>

## **remove**:

> **Value:** `(self, index: @number) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Removes a specific index from the array and returns it._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`index`** | @number | | |
>  
>  
>

## **reverse**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Reverses the array._
>
>  
>

## **shift**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Removes the first index from the array and returns it._
>
>  
>

## **sort**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Sorts the array._
>
>  
>

## **sum**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Gets the sum of the value in the array._
>
>  
>

## **unshift**:

> **Value:** `(self, value) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Pushes a value to the start of the array and returns it._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`value`** |any | | |
>  
>  
>
