## Architectural Decision Record (ADR) 

* ADR-0012: function in effects.rs **apply_shake** owns camera rotation. The constraint exists for adding a second rotation source. 

  *Proposed solution:*
    ```
    The properly correct pattern is to keep the shake offset out of Transform entirely: 
    have camera_follow write the base position/rotation, 
    and apply_shake compute base + shake each frame from a stored base.
    ```

* ADR-0012: function spawn_dust has a hard wired value of the copter half size (16.0). Need to get the height from player's sprite.
* ADR-0013: the frames should be the same size, otherwise spawn_player will create a wrong mesh, possible solution is to find the maximal frame.
* ADR-0014: 
  * The game has the following render layers: 
    * 0 - world rendering
    * 1 - post process overlay: vignette, scan, VHS effect
    * 2 - lights
* ADR-0015: two quads of postprocessing (post and light)= could be merged into one, by updating the shader:
  ```lang=wgsl
      let light = textureSample(light_map, light_sampler, mesh.uv).rgb;
      let vignette = 1.0 - smoothstep(0.35, 0.85, distance(mesh.uv, vec2(0.5))) * params.x;
      // combine both darkenings into one output
  ```
* ADR-0016: need to add night fares, update fare_between (multiply factor derived from DayTime.0.fraction())
* ADR-0017: currently the morning->day->evening->night has the same time length, it's worth to make day and nigh bigger.
* ADR-0018: in CameraPlugin, camera_follow - in case no player, center the camera at (0,0) position (level center).
