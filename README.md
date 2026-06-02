# vatromet
Interoperability software allowing FOR-A HANABI video switcher control boards to be used to control OBS Studio.

_Vatromet = Fireworks in Serbian, the same way Hanabi = Fireworks in Japanese._ 
## Support
This software mainly focuses to support the FOR-A HVS-100OU control board.
## Building
Run `cargo build` in the main directory.
## Configuration
You should either have a `vatromet.toml` file in your pwd or manually define the path to your
configuration file with the `VATROMET_CONFIG` environment variable.

An example configuration file is provided in `vatromet.toml.example`.