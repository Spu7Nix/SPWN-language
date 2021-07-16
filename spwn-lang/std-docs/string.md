  
# **@string**: 
 
## **contains**:

> **Value:** 
>```spwn
>(self, substr: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks if the string contains a string._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`substr`** | @string | | |
>

## **ends\_with**:

> **Value:** 
>```spwn
>(self, substr: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks does the string starts with a string._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`substr`** | @string | | |
>

## **index**:

> **Value:** 
>```spwn
>(self, substr: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Gets the index of a string, if it doesn't exists returns null._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`substr`** | @string | | |
>

## **is\_empty**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns true if the string has a length of 0, false otherwise_
>

## **is\_lower**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks if whole string is lowercase, ignores characters that is not in the alphabet._
>

## **is\_upper**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks if whole string is uppercase, ignores characters that is not in the alphabet._
>

## **join**:

> **Value:** 
>```spwn
>(self, list: @array) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Joins a list using the string._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`list`** | @array | | |
>

## **lowercase**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Makes whole string lowercase._
>

## **reverse**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Reverses the string._
>

## **split**:

> **Value:** 
>```spwn
>(self, spstr: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Splits the string by the specified seperator._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`spstr`** | @string | | |
>

## **starts\_with**:

> **Value:** 
>```spwn
>(self, substr: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks does the string starts with a string._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`substr`** | @string | | |
>

## **substr**:

> **Value:** 
>```spwn
>(self, start: @number, end: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Gets a substring beginning at the specified start and ending at the specified end._
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`start`** | @number | | |
>| 2 | **`end`** | @number | | |
>

## **uppercase**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Makes whole string uppercase._
>
