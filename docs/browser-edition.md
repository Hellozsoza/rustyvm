# Experimental browser frontend

The standalone frontend in `browser/` runs compatible 16/32-bit
x86 guests inside a browser using the v86 WebAssembly emulator. It provides a
VirtualBox-style manager, local boot media, saved states, snapshots, and raw disk
export. It is an alternative frontend and execution engine; it does not run the
native VirtualBox hypervisor or Qt application.

See [the frontend documentation](../browser/README.md)
for build, startup, supported formats, storage behavior, tests, and licenses.
Native VirtualBox guest compatibility, saved states, networking, and hardware
features do not carry over to this experimental browser edition.
