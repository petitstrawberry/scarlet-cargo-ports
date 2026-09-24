# Scarlet Cargo dependency ports

This repository versions the Scarlet-specific source ports used by native Cargo. Each package retains its upstream package name and version so Cargo can select it through a pinned Git source. Package licenses and upstream attribution remain in each source directory.

`scarlet-abi` and `scarlet-sys` are consumed from the pinned Scarlet repository. The source here contains no build artifacts or local patch files.
