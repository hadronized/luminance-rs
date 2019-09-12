# 0.8.1

> Thur Sep 12th 2019

  - Support of `luminance-0.35`.

# 0.8

> Wed Sep 11th 2019

  - Support of `luminance-0.34`.

# 0.7

> Fri Sep 6th 2019

  - Support of `luminance-0.33`.

# 0.6.1

> Tue Sep 3rd 2019

  - Support of `luminance-0.32`.

# 0.6

> Fri Aug 23th 2019

## Major changes

  - Move `swap_buffers` from `GraphicsContext` to `Surface` in [luminance-windowing].

## Minor changes

  - The `WindowOpt` now has support for multisampling. See the `WindowOpt::set_num_samples` for
    further details.
  - Migrate to Rust Edition 2018.
  - Implement dynamic edition of windowing types properties. That allows to change data on-the-fly,
    such as the cursor mode.

## Patch & misc changes

  - Add more CI testing.
  - Massive documentation rewrite (among the use of `#![deny(missing_docs)]`. The situation is still
    not perfect and patch versions will be released to fix and update the documentation. Step by
    step.
  - Massive dependencies update. Special thanks to @eijebong for his help!

# 0.5.4

> Thursday, 29th of July, 2018

- Add support for `luminance-0.30`.

# 0.5.3

> Tuesday, 13th of July, 2018

- Add support for `luminance-0.29`.

# 0.5.2

> Tuesday, 3rd of July, 2018

- Add support for `luminance-0.28`.

# 0.5.1

> Friday, 29th of June, 2018

- Add support for `luminance-0.27`.

# 0.5

> Monday, 18th of June, 2018

- Implement the `luminance` backend interface.
- Support for the new `luminance-windowing` system.
- Remove the concept of `Device` and introduce the concept of `Surface`.

# 0.4.3

> Tuesday, 13th of February, 2018

- Support for `gl-0.10`.

# 0.4.2

> Sunday, 28th of January, 2018

- Support for `gl-0.9`.
- Support for `glfw-0.20`.

# 0.4.1

> Monday, 2nd of October, 2017

- Implement `Display` and `Error` for `GLFWDeviceError`.

# 0.4

> Sunday, 1st of October, 2017

- Use `luminance-windowing` to benefit from the types and functions defined in there.

# 0.3.3

> Saturday, 30th of September, 2017

- Remove the `luminance` dependency as it’s not needed anymore.

# 0.3.2

- All events are now polled, thus all kinds of events are now inspectable.

# 0.3.1

- Support for `glfw-0.16`.

# 0.3

- Removed `open_window` and moved its code into `Device::new`.
- Enhanced the documentation.
- Implemented Hi-DPI (tested on a Macbook Pro).
- Removed `Device::width` and `Device::height` and replaced them with `Device::size`.

# 0.2

- Changed the way events are handled. It doesn’t create an events thread anymore but instead exposes
  a polling interface. It’ll enhance performance and make mono-thread systems work, like Mac OSX.

# 0.1.5

- Internal fix for Mac OSX.

# 0.1.4

- Updated `luminance` dependencies.

# 0.1.3

- Changed the trait bound from `Fn` to `FnOnce` on `Device::draw`.

# 0.1.2

- Made `glfw::InitError` visible.

# 0.1.1

- `Device::draw` added.

# 0.1

- Initial revision.
