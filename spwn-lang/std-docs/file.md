  
# **@file**: 
 
## **new**:

> **Value:** 
>```spwn
>(path: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Creates a new file IO object_
>### Example: 
>```spwn
> @file::new('C:/path/to/file.txt')
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`path`** | @string | | |
>

## **read**:

> **Value:** 
>```spwn
>(self, s = -1) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Reads the data in the file from the seek position to the end (or for a specified amount of characters)_
>### Example: 
>```spwn
> data = @file::new('data.txt').read()
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `s` |any | `-1` | |
>

## **seek**:

> **Value:** 
>```spwn
>(self, s: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Sets a position in the file to read from_
>### Example: 
>```spwn
> f = @file::new('data.txt')
>f.seek(10)
>data = f.read(5) // reads characters 10 to 15
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`s`** | @number | | |
>
