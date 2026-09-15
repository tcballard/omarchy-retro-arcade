# FreeSki asset provenance

All FreeSki visuals are original project work by Omarchy Arcade contributors,
created on 12–14 September 2026 under GPL-3.0-or-later.

| Source | Contents | Creation method |
| --- | --- | --- |
| `shelf.png` | Actual fast-mode Free Ski chase, with the yeti approaching through snowy terrain | Unretouched 1280×900 native app screenshot; shelf UV frames the chase and approaching terrain |
| `../src/render.rs` | World projection, engraved boundary marks, finish stripe, tracks and wind marks | Original native egui vector drawing |
| `../src/artwork.rs` | Layered skier and movement poses, snowy evergreens, faceted rocks, raised ramps, cloth gates, finish flags, shadows, powder and landing puffs | Original hand-authored egui vector geometry and bounded cosmetic animation |
| `../src/geometry.rs` | Shared polygon triangulation and ellipse drawing | Native Rust geometry helpers, extracted unchanged from the yeti renderer |
| `../src/yeti.rs` | Shaggy white yeti, claws, snarling face and running poses | Original hand-authored egui vector geometry; movement-driven animation |
| `../src/world.rs` | Six-section practice slope and three ramp/rock pairs | Original authored world coordinates |
| `../src/course.rs` | Five Slalom courses and medal targets | Original authored coordinates, calibrated against production reference runs |
| `../src/audio.rs` | Carve, jump, crash, gate, miss, warning, catch and finish cues | Original mono PCM synthesis from oscillators and deterministic noise; no samples |
| `../../../shared/presentation/assets/README.md` | Reused Arcade cabinet material | Existing approved shared asset and provenance |

The game is inspired by the downhill skiing genre. No SkiFree images, character
designs, course data or sounds have been imported. Existing games' artwork remains unchanged.

For future assets, add path, creator/source, creation method, licence,
modifications and attribution needs. For generated assets also record the tool
and creation brief. Include notices with the installed package.

## Shelf screenshot, 14 September 2026

Captured from the native release application using `preview-evidence` and
`scripts/native-freeski-close.py`, with a disposable state directory. The example
uses ordinary Session inputs: seed 0, fast mode toggled as pursuit closes, fixture
at tick 1845, then native Enter/resume. The chosen frame shows 1,619 m and 224 km/h
just before a catch. It is a real framebuffer capture, with no compositing,
regenerated objects or edits to actor positions. `arcade/src/shelf.rs` crops
pixels (316,206)–(964,656) from this capture to emphasize the action. The screenshot
contains only the project's original art and existing approved cabinet material;
licence and attribution follow those assets (GPL-3.0-or-later). The superseded
illustrated shelf SVG remains in Git history. No user save was reset.
