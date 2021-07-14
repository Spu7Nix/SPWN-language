  

# **@obj_set**: 
 
## **add**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Add all the objects in the set to the game_
>
>  
>

## **copy**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Create a copy of all the objects in this set as a new set_
>
>  
>

## **is\_empty**:

> **Value:** `(self) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns true if this set contains no objects, false otherwise._
>
>  
>

## **new**:

> **Value:** `(objects: @array, group: @group = ?g) { /* code omitted */ }` (`@macro`) 
>
>### Example: 
>```spwn
> @obj_set::new([]);
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 1 | **`objects`** | @array | | |
>  | 2 | `group` | @group | `?g` |The group to use for rotation (?) |
>  
>  
>

## **push**:

> **Value:** `(self, object: @object) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Add new objects to the set_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`object`** | @object | | |
>  
>  
>

## **rotate**:

> **Value:** `(self, deg: @number) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Applies a single rotation value to all of the objects in this set_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`deg`** | @number | | |
>  
>  
>

## **rotate\_relative**:

> **Type:** `@macro` 
>
>## Description: 
> _Rotates objects in a set around a centerpoint_
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`center_group`** | @group | | |
>  | 3 | **`deg`** | @number | | |
>  | 4 | **`duration`** | @number | | |
>  | 5 | **`easing`** | @easing_type | | |
>  | 6 | **`easing_rate`** | @number | | |
>  | 7 | **`lock_object_rotation`** | @bool | | |
>  
>  
>
