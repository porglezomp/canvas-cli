# Canvas CLI

This is a small tool for interacting with Canvas from the command line.

In order to run it, you'll need to go into your canvas settings and generate an API key, and then fill out a config file.
At `~/.config/canvas-cli/config.toml`, make a config file containing:

```toml

[api]
url = " umich.instructure.com "   # whichever canvas domain that you use
key = " asdfasdfasdf "              # an API key that you generate in your settings

```

You can find documentation by running `canvas --help`
