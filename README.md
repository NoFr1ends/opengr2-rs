# opengr2
Open source parsing library for Granny2 files written in rust.

This library is an alternative rust native version of the C library [opengr2](https://github.com/arves100/opengr2).

## Features
| Functionality         | Status                                  |
|-----------------------|-----------------------------------------|
| Basic parsing         | ⚠️ (some types are still missing)       |
| Big endian files      | ✔️                                      |
| 64 bit files          | ✔️                                      |
| File format 7         | ✔️                                      |
| File format 6         | ⚠️ (should work but needs more testing) |
| Custom element types  | ❌                                       |
| Oodle-0 compression   | ❌                                       |
| Oodle-1 compression   | ❌                                       |
| Bitknit-1 compression | ❌                                       |
| Bitknit-2 compression | ❌                                       |

## Related projects
- [Granny2 Viewer](https://github.com/NoFr1ends/opengr2-viewer) an open source file viewer using egui and runs in the 
  web browser.
- [Bevy File Loader](https://github.com/NoFr1ends/opengr2-bevy) a Bevy plugin to enable loading Granny2 files as a scene.

## License
This library is licensed under the [MIT-License](LICENSE).