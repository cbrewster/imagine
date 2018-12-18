## Imagine

An experimental Rust GUI.

I am building this GUI library to experiment with different ways to build GUI libraries in Rust.
This approach uses ECS (via specs) and webrender for rendering. Neither of these will be exposed in
the public interface so that users will not need to understand ECS or Webrender.

The layout is done via Flutter's box constraints and I have taken a lot of inspiration from
[druid](https://github.com/xi-editor/druid).
