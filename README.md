# NOEMA // Autonomous Wikipedia Webpage

**NOEMA** is an interactive, bioluminescent artificial-life ecosystem and autonomous Wikipedia node explorer written in Rust. Rather than presenting encyclopedic knowledge as static text, NOEMA turns live Wikipedia articles into a reactive, living canvas inhabited by **1,111 procedural arthropod specimens** that roam, tether, incinerate, shatter, and mutate human knowledge in real time.

Repository: https://github.com/ShamelesAbyss/Noema

---

## Core Architecture

### 1. Autonomous Procedural Specimens (src/creature.rs)
* **1,111 Deterministic Catalog Specimens:** Every specimen is procedurally generated from a deterministic seed with distinct spine chamber geometry (HollowRect, DualCell, CrossCore, NestedDiamond, HexFrame, RibbedCompartment), dorsal structures (WireframeWings, LateralFins, OrbitingSatellites, DorsalSpikes), ocular arrays, tentacles, and multi-jointed Inverse Kinematics (IK) limbs.
* **Six Autonomous Behavioral Modes:**
  * **Silk Weaver (#0002):** Tethers individual words to its articulated legs, plucking them from paragraphs and flinging them across the canvas.
  * **Laser Incinerator (#0004):** Targets and vaporizes words across the page with thermal beams, leaving glowing embers that regenerate over time.
  * **Berserker Breaker (#0005):** Charges through dense text blocks, shattering paragraphs into kinetic word-chips and shockwaves.
  * **Harmonic Architect (#0006):** Emits cymatic footstep shockwaves and crystallizes words into illuminated floating highlights.
  * **Tesla Arc Corruptor (#0007):** Discharges jagged electrical arcs into body text, mutating words in-place using contextual Markov chains.
  * **Gravity Singularity (#0015):** Pulls surrounding words into a rotating orbital vortex around its spine core.

### 2. Live MediaWiki Bridge & Taxonomic UI (src/web.rs)
* **Real-Time Wikipedia Routing:** Powered by a multithreaded ureq MediaWiki API bridge with automatic search fallback. Clicking any highlighted word, hyperlink, plucked word-chip, cladogram branch, or infobox taxon routes the canvas directly to that Wikipedia instance.
* **Scientific Classification Infobox & Cladogram:** Renders a dark-mode taxonomic sidebar featuring an animated wireframe arthropod schematic, classification hierarchy, diversity metrics, and an interactive phylogenetic cladogram widget.
* **Organic Shadow Carving:** As specimens move across the article, their segments cast bioluminescent shadows that carve through the underlying typography.

### 3. Tactile 48-Voice FM Synthesizer (src/audio.rs)
* **Procedural Audio Engine:** Every sound is synthesized in real time at 44.1 kHz via a custom rodio Source (TactileSynthSource) with zero external audio files.
* **Microtonal Scales & Stereo Delay:** Specimens are tuned to 8 harmonic and just-intonation scales with cross-feedback stereo delay and a continuous LFO drone that shifts with specimen vitality and curiosity.
* **Tactile Mouse Brush:** Brushing the cursor across a specimen's spine chambers, leg joints, ocular core, or tentacle beads triggers velocity-sensitive FM voices panned dynamically across the stereo field.

### 4. Markov Lexicon Engine (src/lexicon.rs)
* **Live Corpus Ingestion:** Every loaded Wikipedia article is parsed into single-word and trigram Markov chains.
* **Generative Synthesis Stream:** Words plucked or vaporized by specimens are absorbed into the lexicon engine, continuously composing an evolving sentence inside the on-page synthesis pipeline box.

---

## Controls

| Input | Action |
| :--- | :--- |
| **Left Click (Empty Space)** | Set movement waypoint for the active specimen |
| **Left Click (Word / Link / Taxon)** | Route canvas to the clicked Wikipedia article |
| **Ctrl + Left Click (Link)** | Route canvas AND open the article in your system web browser |
| **Mouse Brush (Over Specimen)** | Trigger tactile FM synthesis across spine segments, joints, and tentacles |
| **Scroll Wheel / W S / Up Down** | Scroll vertically through the Wikipedia article |
| **Right / Middle Click + Drag** | Pan camera freely across the canvas |
| **A / D or Left / Right** | Cycle through the 1,111 catalog specimens (Shift to jump by 17) |
| **1 - 6** | Quick-select archetype specimens (#0002, #0007, #0015, #0004, #0005, #0006) |
| **R** | Spawn a random specimen from the 1,111 catalog |
| **TAB** | Open on-canvas search modal to load any Wikipedia topic |
| **P** | Cycle through featured biological and cybernetic Wikipedia articles |
| **O** | Open the current Wikipedia article in your default system browser |
| **M** | Mute / unmute the procedural FM audio engine |

---

## Building & Running

Ensure you have the Rust toolchain (cargo) installed:

    git clone https://github.com/ShamelesAbyss/Noema.git
    cd Noema
    cargo run --release
