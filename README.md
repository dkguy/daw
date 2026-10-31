# daw

A small, dependency-free step sequencer written in Rust. Edit a 16-step drum and bass pattern, set the tempo, and export four bars as a mono WAV file.

## Run

```sh
cargo run
```

At the `daw>` prompt, use:

- `show` to display the current pattern
- `toggle <track> <step>` to toggle a step (tracks: `kick`, `snare`, `hat`, `bass`; steps: 1-16)
- `bpm <value>` to set the tempo (40-240)
- `export <file.wav>` to render four bars to a WAV file
- `help` to show commands, or `quit` to exit

For example, `toggle bass 5` and then `export beat.wav` adds a bass note and renders the pattern. Open the exported WAV in any audio player or DAW.

## Flatpak build

GitHub Actions builds a Flatpak bundle on each push and pull request. Download the `daw-flatpak` artifact from the workflow run, then install and launch it with:

```sh
flatpak install --user ./daw.flatpak
flatpak run io.github.dkguy.daw
```
