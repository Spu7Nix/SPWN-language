  
# **@obj_set**: 
 
## **add**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Add all the objects in the set to the game_
>

## **copy**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Create a copy of all the objects in this set as a new set_
>

## **is\_empty**:

> **Value:** 
>```spwn
>(self) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Returns true if this set contains no objects, false otherwise._
>### Example: 
>```spwn
> $.assert(@obj_set::new().is_empty())
>```
>

## **new**:

> **Value:** 
>```spwn
>(objects: [@object] = [], group: @group = ?g) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Creates a new object set_
>### Example: 
>```spwn
> my_objects = @obj_set::new()
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | `objects` | [@object] | `[]` | |
>| 2 | `group` | @group | `?g` |The center group to use for rotation |
>

## **push**:

> **Value:** 
>```spwn
>(self, object: @object) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Add new objects to the set_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`object`** | @object | | |
>

## **rotate**:

> **Value:** 
>```spwn
>(self, deg: @number) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Applies a single rotation value to all of the objects in this set_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`deg`** | @number | | |
>

## **rotate\_relative**:

> **Value:** 
>```spwn
>(self, center_group: @group, deg: @number, duration: @number, easing: @easing_type, easing_rate: @number, lock_object_rotation: @bool) { /* code omitted */ }
>``` 
>**Type:** `@macro` 
>## Description: 
> _Rotates objects in a set around a centerpoint_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`center_group`** | @group | | |
>| 2 | **`deg`** | @number | | |
>| 3 | **`duration`** | @number | | |
>| 4 | **`easing`** | @easing_type | | |
>| 5 | **`easing_rate`** | @number | | |
>| 6 | **`lock_object_rotation`** | @bool | | |
>
