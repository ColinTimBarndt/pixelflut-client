# Pixelflut-Client

This is a rust-implementation of a [Pixelflut]-Client.
This client can flood the canvas at a given offset with
a specified GIF-image.

Have any suggestions for a better performance? Feel free
to make a pull request or create an issue.

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
    -a, --alpha-color <COLOR>          Color to use for transparent pixels
    -f, --file <FILE>                  Specifies the GIF file path or URL
    -x <OFFSET>                        X-Offset on the Pixelflut canvas
    -y <OFFSET>                        Y-Offset on the Pixelflut canvas
    -s, --similarity <THRESHOLD>       How similar pixels can be to be ignored (default: 0)
    -u, --url <URL>                    Specify the Pixelflut Server URL
```

## Possible improvements

- Better similarity algorhitm for better
  image quality with less performance penalty

[Pixelflut]: https://github.com/defnull/pixelflut
