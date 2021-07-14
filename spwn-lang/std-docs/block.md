  

# **@block**: 
 
## **\_range\_**:

> **Value:** `(self, other: @block) { /* code omitted */ }` (`@macro`) 
>
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @block | | |
>  
>  
>

## **create\_tracker\_item**:

> **Value:** `(self, other: @block) { /* code omitted */ }` (`@macro`) 
>
>## Description: 
> _Returns an item ID that is 1 when the blocks are colliding and 0 when they are not_
>### Example: 
>```spwn
> // in some minigame
>player = @player::{ block: 1b, group: 1g}
>ground = 2b
>on_ground = counter(player.block.create_tracker_item(ground))
>on(touch(), !{
>    //jump
>    if on_ground == 1 { // player can only jump while touching the ground
>        player.jump()
>    }
>})
>```
>## Arguments:
>
>| # | name | type | default value | description |
>| - | ---- | ---- | ------------- | ----------- |
>| 2 | **`other`** | @block | |Block ID to check against |
>  
>  
>
