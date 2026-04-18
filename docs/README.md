# nocaprs [WIP]

Cross platform screen recording tool with built in support for camera and
microphone input, along with features like mouse tracking and live zoom,
all built from the ground up in Rust.


## Directory structure
```
.
├── app                  # Contains user applications
│   ├── nocaprs          # Main gui user application
│   └── nocaprs-cli      # Future planned nocaprs cli application
└── crates               # Internal crates for nocaprs application
    ├── platform         # Platform interfaces without any implementation
    └── platform-*       # Platform interfaces implementations for a specific platform
```

## Architecture

```mermaid
flowchart TD
    A[Applications] --> B[Post processing]
    B --> D[Unified Platform API]

    D --> E[Screen Capture / Audio / Camera]

    E --> F1[Linux]
    E --> F2[Windows]
    E --> F3[macOS]

    F1 --> G1[Wayland, PipeWire]
    F2 --> G2[Windows.Graphics.Capture]
    F3 --> G3[ScreenCaptureKit]
```
## UI

This is a very early prototype design idea for the main application

![ui](./assets/ui.png)
