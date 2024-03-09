# RMenu

RMenu is like dmenu but in rust

![Demo](assets/preview.gif)

# Installation
Installation can be done with the following commands
```
$ chmod +x build.sh
$ sudo ./build.sh
```

# Usage
```
Usage: rmenu [options]
options:
    -config <path>      custom config path
    -help               show help
```

# Configuration
In order to start configuring you first need to create the config file
at `~/.config/rmenu/config.toml`.

This can be done with the following shell commands
```
$ mkdir .config/rmenu
$ touch .config/rmenu/config.toml
```

Next you want to insert the following template into `config.toml`
```
foreground = "254-128-25"
background = "40-40-40"
font = "DejaVu Sans Mono:size=11:antialias=true"
```

## Colors
The colors in RMenu are formated using rgb but it doesnt follow any standard,
as seen in the configuration guide, rgb values are seperated by a `-` for readability
In a context where `<r/g/b>` is the color values the format is the following `<r>-<g>-<b>`

## Fonts
RMenu has by default support for scalable fonts using Xft to draw these fonts,
the format in which these fonts are specified is with a Fontconfig pattern as the
string is passed straight to XftOpenFontName (See https://www.x.org/archive/X11R7.5/doc/man/man3/Xft.3.html)


