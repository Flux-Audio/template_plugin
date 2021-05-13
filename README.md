\< TOC goes here. Generate one at: https://ecotrust-canada.github.io/markdown-toc/ \>

# \< PLUGIN_NAME \> \< semVer number goes here\>
***Categories:** \< categories go here, i.e. meta-plugin \>*

## Update Notice
Version \< current semVer \> is incompatible with \< last incompatible semVer \>! Presets cannot be ported. Make sure to
backup any old instances of \< last incompatible semVer \> if you don't want your projects to break.

\< short update description \>

A full changelist is at the bottom of this document.

## Installation
_**Disclaimer:** this plugin will only work on 64-bit windows machines!_ \
Download the `.dll` file in the `bin/` directory and place it into your DAW's VST folder.
Previous versions of the plugin are also available, in case you need them.

## Compiling The Source Code
_**Note:** you don't need to compile the source code if you just want to use the plugin, just download the `.dll`._ \
Make sure you have Cargo installed on your computer (the Rust compiler). Then in the root of the repository run `cargo build`. Once Cargo is done building, there should be a `HYSTERESIS_v0_3_0.dll` file in the newly created `debug/` directory. Place this file into your DAW's VST folder.

# What is \< PLUGIN_NAME \>?
\< medium description goes here \>

# Controls Explained
+ list
+ of
+ parameter
+ names: and short descriptions for each one of them

# Extra Info for Nerds
\< long description goes here \>


# Changelist

## \< semVer number \>
- Added: new features
- Modified: breaking changes to old features
- Removed: removed old features
- Fixed: non-breaking fixes to bugs
- Tweaked: non-breaking non-functional changes to old features, i.e. improved quality

# Known Bugs
For a detailed list, see the [issues]() tab.
- list
- of
- bugs, with short descriptions
