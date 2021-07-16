  
# **@dictionary**: 
 
## **clear**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Clears the dictionary._
>### Example: 
>```spwn
> let wares = {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>}
>wares.clear()
>
>$.assert(wares.is_empty())
>```
>

## **contains\_value**:

> **Value:** 
>```spwn
>(self, value) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks if the dictionary contains a value._
>### Example: 
>```spwn
> let wares = {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>}
>
>$.assert(wares.contains(5))
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`value`** |any | | |
>

## **get**:

> **Value:** 
>```spwn
>(self, key: @string, default = @dict_not_found::{}) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Gets an item from the dictionary._
>### Example: 
>```spwn
> let wares = {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>}
>
>$.assert(wares.get('peanuts') == 5)
>$.assert(wares.get('silver', default = 42) == 42)
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`key`** | @string | | |
>| 2 | `default` |any | `@dict_not_found::{}` | |
>

## **is\_empty**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns true if there are no entries in the dictionary, false otherwise._
>### Example: 
>```spwn
> dict = {}
>$.assert(dict.is_empty())
>```
>

## **items**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Gets the dictionary's items._
>### Example: 
>```spwn
> wares = {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>}
>$.assert(wares.items() == [
>	['apple', 10],
>	['gold', 1000],
>	['peanuts', 5],
>])
>```
>

## **keys**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Gets the dictionary's keys._
>### Example: 
>```spwn
> wares = {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>}
>$.assert(wares.keys() == ['apple', 'gold', 'peanuts'])
>```
>

## **set**:

> **Value:** 
>```spwn
>(self, key: @string, val) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Sets an item in the dictionary._
>### Example: 
>```spwn
> let wares = {
>	apple: 10,
>	gold: 1000,
>}
>wares.set('peanuts', 5)
>$.assert(wares == {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`key`** | @string | | |
>| 2 | **`val`** |any | | |
>

## **values**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Gets the dictionary's values._
>### Example: 
>```spwn
> wares = {
>	apple: 10,
>	gold: 1000,
>	peanuts: 5,
>}
>$.assert(wares.values() == [10, 1000, 5])
>```
>
