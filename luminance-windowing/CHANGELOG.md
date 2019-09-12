# 0.5.1

> Thur Sep 12th 2019

  - Support of `luminance-0.35`.

# 0.5

> Wed Sep 11th 2019

## Major changes

  - The `Surface` trait has a new method to implement: `Surface::back_buffer`. That method provides
    the `Framebuffer::back_buffer` in a much more convenient way and is automatically implemented
    by default.

# 0.4

> Fri Sep 6th 2019

  - Support of `luminance-0.33`.

# 0.3.1

> Tue Sep 3rd 2019

  - Support of `luminance-0.32`.

# 0.3

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

# 0.2.4

> Thursday, 20th of July, 2018

  - Add support for `luminance-0.30`.

# 0.2.3

> Tuesday, 13th of July, 2018

  - Add support for `luminance-0.29`.

# 0.2.2

> Tuesday, 3rd of July, 2018

  - Add support for `luminance-0.28`.

# 0.2.1

> Friday, 29th of June, 2018

  - Add support for `luminance-0.27`.

# 0.2

> Sunday, 17th of June, 2018

  - Re-export the new `luminance` backend interface.
  - Remove the concept of `Device` and the concept of `Surface`.

# 0.1.1

> Sunday, 1st of October, 2017

  - Fix a small nit in the documentation.

# 0.1.0

> Saturday, 30th of September, 2017

  - Initial revision.
