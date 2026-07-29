## Architectural Decision Record (ADR) 

* ADR-0012: function in effects.rs **apply_shake** owns camera rotation. The constraint exists for adding a second rotation source. 

  *Proposed solution:*
    ```
    The properly correct pattern is to keep the shake offset out of Transform entirely: 
    have camera_follow write the base position/rotation, 
    and apply_shake compute base + shake each frame from a stored base.
    ```

* ADR-0012: function spawn_dust has a hard wired value of the copter half size (16.0). Need to get the height from player's sprite.
