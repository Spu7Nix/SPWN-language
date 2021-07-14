  

# **@array**: 
 
## **all**:

> **Value:** `(self, map: @macro = (a) { /* code omitted */ }) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Determines whether all the members of an array satisfy the specified callback._
>### Example: 
>```spwn
> arr = [true, true, true]
>$.assert(arr.all())
>arr2 = [1, 2, 3, 1, 4, 7]
>$.assert(arr2.all(el => el > 0)) // checks if the array contains only positive elements
>```
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
>### Example: 
>```spwn
> arr = [false, false, true, false]
>$.assert(arr.any())
>arr2 = [1, 2, 3, 1, 4, -1, 7]
>$.assert(arr2.any(el => el < 0)) // checks if the array contains any negative elements
>```
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
>### Example: 
>```spwn
> let arr = [1, 2, 3]
>arr.clear()
>$.assert(arr.is_empty())
>```
>
>  
>

## **contains**:

> **Value:** `(self, el) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _See if array contains an element._
>### Example: 
>```spwn
> fruit = ['apple', 'banana', 'mango']
>$.assert(arr.contains('banana'))
>```
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
>### Example: 
>```spwn
> arr = [1, 2, 3, 4, 5]
>$.assert(arr.filter(el => el > 3) == [4, 5])
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`cb`** | @macro | | |
>  
>  
>

## **flat\_map**:

> **Value:** `(self, cb: @macro = () { /* code omitted */ }) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Works the same way as map but flattens any sub-arrays into one big array._
>### Example: 
>```spwn
> arr = [1, 2, [3, 4], 5, [6, 7, [8]]]
>$.assert(arr.flat_map(el => el > 4) == [5, 6, 7, 8])
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | `cb` | @macro | `() { /* code omitted */ }` | |
>  
>  
>

## **index**:

> **Value:** `(self, el) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Gets the index of an element (if it doesn't exist, `null` is returned)_
>### Example: 
>```spwn
> fruit = ['apple', 'banana', 'mango']
>$.assert(fruit.index('apple') == 0)
>$.assert(fruit.index('carrot') == null)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`el`** |any | | |
>  
>  
>

## **is\_empty**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns true if the array has a length of 0, false otherwise._
>### Example: 
>```spwn
> arr = []
>arr2 = [1, 2, 3]
>$.assert(arr.is_empty())
>$.assert(!arr2.is_empty())
>```
>
>  
>

## **map**:

> **Value:** `(self, cb: @macro) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Calls a defined callback function on each element of an array, and returns an array that contains the results._
>### Example: 
>```spwn
> arr = [1, 2, 3, 4, 5]
>$.assert(arr.map(el => el * 2) == [2, 4, 6, 8, 10])
>```
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
>### Example: 
>```spwn
> arr = [3, 1, 4, 1]
>$.assert(arr.max() == 4)
>```
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
>### Example: 
>```spwn
> arr = [3, 1, 4, 1]
>$.assert(arr.min() == 1)
>```
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
>### Example: 
>```spwn
> let arr = [1, 2, 3, 4]
>arr.pop()
>$.assert(arr == [1, 2, 3])
>```
>
>  
>

## **push**:

> **Value:** `(self, value) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Pushes a value to the end of the array._
>### Example: 
>```spwn
> let arr = [1, 2, 3]
>arr.push(4)
>$.assert(arr == [1, 2, 3, 4])
>```
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
>### Example: 
>```spwn
> arr = [1, 2, 3, 4, 5]
>sum = arr.reduce((acum, el) => acum + el)
>$.assert(sum == 15)
>```
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
>### Example: 
>```spwn
> let arr = [1, 2, 3, 4, 5]
>arr.remove(3)
>$.assert(arr == [1, 2, 3, 5])
>```
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
>### Example: 
>```spwn
> let arr = [1, 2, 3]
>arr.reverse()
>$.assert(arr == [3, 2, 1])
>```
>
>  
>

## **shift**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Removes the first index from the array and returns it._
>### Example: 
>```spwn
> let arr = [5, 1, 5, 3, 2]
>$.assert(arr.shift() == 5)
>$.assert(arr == [1, 5, 3, 2])
>```
>
>  
>

## **sort**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns a sorted version of the array._
>### Example: 
>```spwn
> arr = [5, 1, 5, 3, 2]
>$.assert(arr.sort() == [1, 2, 3, 5, 5])
>```
>
>  
>

## **sum**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Gets the sum of the value in the array._
>### Example: 
>```spwn
> arr = [1, 2, 3, 4, 5]
>$.assert(arr.sum() == 15)
>```
>
>  
>

## **unshift**:

> **Value:** `(self, value) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Pushes a value to the start of the array and returns it._
>### Example: 
>```spwn
> let arr = [1, 5, 3, 2]
>$.assert(arr.unshift(5) == 5)
>$.assert(arr == [5, 1, 5, 3, 2])
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`value`** |any | | |
>  
>  
>
