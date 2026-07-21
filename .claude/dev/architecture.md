# プロジェクト構造

伺か（Ukagaka）デスクトップマスコット向けの「Ghost」。Rust 製。

## Core Structure
- `ghost/master/src/lib.rs`: Main entry point with C FFI functions (`load`, `unload`, `request`)
- `ghost/master/src/events/`: Event handling system organized by event type
- `ghost/master/src/variables.rs`: Global state management
- `shell/master/`: Visual assets and surface definitions

## Event System
Events are organized into specialized modules:
- `events/aitalk.rs`: AI talk functionality
- `events/bootend.rs`: Boot/shutdown events
- `events/common.rs`: Common utilities (response generation, blink animation, icons)
- `events/input.rs`: Input handling
- `events/key.rs`: Keyboard input events
- `events/mouse.rs`: Mouse interaction
- `events/mouse_core.rs`: Core mouse event processing (wheel, double-click, move)
- `events/talk.rs`: Talk system main module (Talk, TalkType, TalkingPlace)
- `events/talk/`: Talk-related events (first boot, random talk, anchors)
- `events/periodic.rs`: Periodic/timer events
- `events/menu.rs`: Menu system
- `events/tooltip.rs`: Tooltip display
- `events/translate.rs`: Translation features
- `events/update.rs`: Update events
- `events/webclap.rs`: Web clap functionality

## Build Process
The build script performs multiple tasks:
1. Builds Rust DLL (`haine.dll`) with release optimizations
  - must process with cargo.exe (windows)
2. Extracts 7-digit surface numbers from Rust source files
3. Generates surface definitions using `surfaces-mixer`
4. Processes candle images with ImageMagick to create animated surfaces
5. Sends SSTP notifications to running Ghost instances

## Key Features
- Morphological analysis with MeCab (vibrato)
- Audio playback with rodio
- SSTP communication for Ghost interaction
- Surface animation system with collision detection
- Logging system with file output
- Debug mode support (controlled by `./debug` file existence)

## Testing

No specific test commands found in the codebase. The project appears to rely on integration testing through the Ghost runtime environment.
