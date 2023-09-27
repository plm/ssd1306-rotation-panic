SSD1306 Rotation Panic
===

Reproduces a panic when using [ssd1306 version `0.8.1`](https://docs.rs/ssd1306/0.8.1/ssd1306/index.html)'s `DisplaySize128x64` with `DisplayRotation::Rotate90` or `DisplayRotation::Rotate270` in `TerminalMode`.

Using a [Seeeduino XIAO RP2040](https://github.com/rp-rs/rp-hal-boards/tree/main/boards/seeeduino-xiao-rp2040) with [expansion board](https://wiki.seeedstudio.com/Seeeduino-XIAO-Expansion-Board/), this example will panic when writing the ninth test line ("7TEST"). Panic can be observed when the blue LED (GPIO 25) turns on and no additional test lines are written to the display. When `DisplayRotation::Rotate0` or `DisplayRotation::Rotate180` are used, no panic is produced and the display prints test lines continuously.

Testing shows that the first character of each of the test lines is written to the first/top line of the display rather than subsequent lines with the rest of the test string. The first character is also offset within the first line based on the row it should have been printed within. The unexpected offset and attempts to write the ninth line's first character ("7") offscreen is likely the cause of the panic.

Push this firmware to your Seeeduino XIAO RP2040 by holding down the `B` button and either powering on the board or pressing the `R` button. Release all buttons and then run:

```
cargo run --release
```

During execution, character positions are written to `UART0` at 115200 bps and will produce the following output:

```
Iter#0>(0, 1)
Iter#1>(0, 2)
Iter#2>(0, 3)
Iter#3>(0, 4)
Iter#4>(0, 5)
Iter#5>(0, 6)
Iter#6>(0, 7)
```
