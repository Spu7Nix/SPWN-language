  
# **@regex**: 
 
## **match**:

> **Value:** 
>```spwn
>(self, match: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Checks if the regex matches a string argument_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`match`** | @string | | |
>

## **new**:

> **Value:** 
>```spwn
>(re: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Create a new instance of regex_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`re`** | @string | |A regex string. Make sure to use two backslashes to escape selectors instead of one or it will error |
>

## **replace**:

> **Value:** 
>```spwn
>(self, to_replace: @string, replacer: @string) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Regex replace the contents of a string_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`to_replace`** | @string | | |
>| 2 | **`replacer`** | @string | | |
>
