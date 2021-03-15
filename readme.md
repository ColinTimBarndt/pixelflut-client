# Pixelflut-Client

This is a rust-implementation of a [Pixelflut]-Client.
This client can flood the canvas at a given offset with
a specified GIF-image.

Have any suggestions for a better performance? Feel free
to make a pull request or create an issue.

## How does it work?

The client first loads the GIF by either downloading it into
the RAM or streaming it from storage. The frames are then
processed with optimizations, so that changed pixels are
drawn with priority when a frame changes for a smoother
animation. After that, all commands needed for displaying
the frames are pre-generated and cached.

The complete image is redrawn as
fast as possible in a loop until the next frame begins. This
is to prevent griefing from other evil Fluter clients.

## CLI

For help, run `pixelflut-client -h`:

```txt
Pixelflut Client 1.0
Colin Tim Barndt <colin.barndt@gmail.com>
Stream a gif to a server using Pixelflut

USAGE:
    pixelflut-client.exe [OPTIONS] --file <FILE> --url <URL>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --file <FILE>               Specifies the GIF file path or URL
    -x <OFFSET>                     X-Offset on the Pixelflut canvas
    -y <OFFSET>                     Y-Offset on the Pixelflut canvas
        --shuffle <SHUFFLE>         If the instruction should be shuffled for a better image quality if griefed
                                    (default: yes) [possible values: yes, no]
    -s, --similarity <THRESHOLD>    How similar pixels can be to be treated as equal, smaller values mean more
                                    similarity (default: 0)
        --time-factor <FACTOR>      Factor by which to scale the time between frames from the original GIF, a higher
                                    value means slower animation but more resistant against grief (default: 10)
    -u, --url <URL>                 Specify the Pixelflut Server URL
```

## Possible improvements

- Better similarity algorhitm for better
  image quality with less performance penalty
- More parallel processing

[Pixelflut]: https://github.com/defnull/pixelflut
